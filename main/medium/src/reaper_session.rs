use std::ptr::NonNull;

use reaper_low::{
    create_cpp_to_rust_control_surface, delete_cpp_control_surface, raw, IReaperControlSurface,
    PluginContext,
};

use crate::keeper::{Keeper, SharedKeeper, SimpleKeeper};

use crate::{
    concat_reaper_strs, delegating_hook_command, delegating_hook_command_2,
    delegating_hook_post_command, delegating_hook_post_command_2, delegating_toggle_action,
    AcceleratorPosition, BufferingBehavior, CommandId, ControlSurface, ControlSurfaceAdapter,
    FileInProjectCallback, GenericRegistrationHandle, Handle, HookCommand, HookCommand2,
    HookCustomMenu, HookPostCommand, HookPostCommand2, MainThreadScope, MeasureAlignment,
    OnAudioBuffer, OwnedAcceleratorRegister, OwnedAudioHookRegister, OwnedGaccelRegister,
    OwnedPreviewRegister, PluginRegistration, ProjectContext, ReaProject, RealTimeAudioThreadScope,
    Reaper, ReaperFunctionError, ReaperFunctionResult, ReaperMutex, ReaperString, ReaperStringArg,
    RegistrationHandle, RegistrationObject, ToggleAction, ToolbarIconMap, TranslateAccel,
};
use reaper_low::raw::audio_hook_register_t;

use crate::file_in_project_hook::OwnedFileInProjectHook;
use crate::fn_traits::{delegating_hook_custom_menu, delegating_toolbar_icon_map};
use enumflags2::BitFlags;
use std::collections::{HashMap, HashSet};
use std::os::raw::{c_char, c_void};
use std::sync::Arc;

/// This is the main hub for accessing medium-level API functions.
///
/// In order to use this struct, you first must obtain an instance of it by invoking [`new()`]
/// or [`load()`].
/// This struct itself is limited to REAPER functions for registering/unregistering certain things.
/// You can access all the other functions by calling [`reaper()`].
///
/// Please note that this struct will take care of unregistering everything (also audio hooks)
/// automatically when it gets dropped (good RAII manners).
///
/// # Design
///
/// ## Why is there a separation into `ReaperSession` and `Reaper`?
///
/// Functions for registering/unregistering things have been separated from the rest because they
/// require more than just access to REAPER function pointers. They also need data structures to
/// keep track of the registered things and to offer them a warm and cosy place in memory. As a
/// result, this struct gains special importance, needs to be mutable and can't just be cloned as
/// desired. But there's no reason why this restriction should also apply to all other REAPER
/// functions. After all, being able to clone and pass things around freely can simplify things a
/// lot.
///
/// ### Example
///
/// Here's an example how things can get difficult without the ability to clone: In order to be able
/// to use REAPER functions also from e.g. the audio hook register, we would need to wrap it in an
/// `Arc` (not an `Rc`, because we access it from multiple threads). That's not enough though for
/// most real-world cases. We probably want to register/unregister things (in the main thread) not
/// only in the beginning but also at a later time. That means we need mutable access. So we end up
/// with `Arc<Mutex<ReaperSession>>`. However, why going through all that trouble and put up with
/// possible performance issues if we can avoid it?
///
/// [`new()`]: #method.new
/// [`load()`]: #method.load
/// [`reaper()`]: #method.reaper
#[derive(Debug, Default)]
pub struct ReaperSession {
    reaper: Reaper<MainThreadScope>,
    /// Provides a safe place in memory for registered actions.
    gaccel_registers: Keeper<OwnedGaccelRegister, raw::gaccel_register_t>,
    /// Provides a safe place in memory for accelerator registers.
    accelerator_registers: Keeper<OwnedAcceleratorRegister, raw::accelerator_register_t>,
    /// Provides a safe place in memory for file-in-project hooks.
    file_in_project_hooks: SimpleKeeper<OwnedFileInProjectHook>,
    /// Provides a safe place in memory for currently playing preview registers.
    preview_registers: SharedKeeper<ReaperMutex<OwnedPreviewRegister>, raw::preview_register_t>,
    /// Provides a safe place in memory for command names used in command ID registrations.
    //
    // We don't need to box the string because it's content is something which is on the heap
    // already and doesn't change its address when moved.
    command_names: HashSet<ReaperString>,
    /// Provides a safe place in memory for API definition string structs.
    api_defs: Vec<Vec<c_char>>,
    /// Provides a safe place in memory for each registered audio hook.
    ///
    /// While in here, the audio hook is considered to be owned by REAPER, meaning that REAPER is
    /// supposed to have exclusive access to it.
    audio_hook_registers: Keeper<OwnedAudioHookRegister, raw::audio_hook_register_t>,
    /// Provides a safe place in memory for each registered control surface.
    ///
    /// While in here, the control surface is considered to be owned by REAPER, meaning that REAPER
    /// is supposed to have exclusive access to it.
    #[allow(clippy::redundant_allocation)]
    csurf_insts: HashMap<Handle<c_void>, Box<Box<dyn IReaperControlSurface>>>,
    /// Provides a safe place in memory for plug-in registration keys (e.g. "API_myfunction").
    ///
    /// Also used for keeping track of registrations so they can be unregistered automatically on
    /// drop.
    plugin_registrations: HashSet<PluginRegistration>,
    /// Keep track of audio hook registrations so they can be unregistered automatically on drop.
    audio_hook_registrations: HashSet<Handle<raw::audio_hook_register_t>>,
    /// Keep track of playing preview registers so they can be unregistered automatically on drop.
    playing_preview_registers: HashSet<Handle<raw::preview_register_t>>,
    /// Keep track of playing track preview registers so they can be unregistered automatically on drop.
    playing_track_preview_registers: HashSet<(ProjectContext, Handle<raw::preview_register_t>)>,
}

// The raw pointers contained in the session don't do harm when sent to another thread.
unsafe impl Send for ReaperSession {}

impl ReaperSession {
    /// Creates a new instance by getting hold of a [low-level `Reaper`] instance.
    ///
    /// [low-level `Reaper`]: https://docs.rs/reaper-low
    pub fn new(low: reaper_low::Reaper) -> ReaperSession {
        ReaperSession {
            reaper: Reaper::new(low),
            gaccel_registers: Default::default(),
            accelerator_registers: Default::default(),
            file_in_project_hooks: Default::default(),
            preview_registers: Default::default(),
            command_names: Default::default(),
            api_defs: Default::default(),
            audio_hook_registers: Default::default(),
            csurf_insts: Default::default(),
            plugin_registrations: Default::default(),
            audio_hook_registrations: Default::default(),
            playing_preview_registers: Default::default(),
            playing_track_preview_registers: Default::default(),
        }
    }

    /// Loads all available REAPER functions from the given plug-in context.
    ///
    /// Returns a medium-level `ReaperSession` instance which allows you to call these functions.
    pub fn load(context: PluginContext) -> ReaperSession {
        let low = reaper_low::Reaper::load(context);
        ReaperSession::new(low)
    }

    /// Gives access to all REAPER functions which can be safely executed in the main thread.
    ///
    /// # Example
    ///
    /// If the REAPER functions are needed somewhere else, just clone them:
    ///
    /// ```no_run
    /// # let mut session = reaper_medium::ReaperSession::default();
    /// let standalone_reaper = session.reaper().clone();
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn reaper(&self) -> &Reaper<MainThreadScope> {
        &self.reaper
    }

    /// Creates a new container of REAPER functions with only those unlocked that can be safely
    /// executed in the real-time audio thread.
    pub fn create_real_time_reaper(&self) -> Reaper<RealTimeAudioThreadScope> {
        Reaper::new(*self.reaper.low())
    }

    /// This is the primary function for plug-ins to register things.
    ///
    /// *Things* can be keyboard shortcuts, project importers etc. Typically you register things
    /// when the plug-in is loaded.
    ///
    /// It is not recommended to use this function directly because it's unsafe. Consider using
    /// the safe convenience functions instead. They all start with `plugin_register_add_`.
    ///
    /// The meaning of the return value depends very much on the actual thing being registered. In
    /// most cases it just returns 1. In any case, if it's not 0, *reaper-rs* translates this into
    /// an error.
    ///
    /// Also see [`plugin_register_remove()`].
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer or if it dangles during the time it
    /// is registered. So you must ensure that the registered thing lives long enough and
    /// has a stable address in memory. Additionally, mutation of the thing while it is registered
    /// can lead to subtle bugs.
    ///
    /// [`plugin_register_remove()`]: #method.plugin_register_remove
    pub unsafe fn plugin_register_add(
        &mut self,
        object: RegistrationObject,
    ) -> ReaperFunctionResult<i32> {
        let reg = object.into_raw();
        let key_ptr = reg.key.as_ptr();
        let value = reg.value;
        self.plugin_registrations.insert(reg);
        let result = self.reaper.low().plugin_register(key_ptr, value);
        if result == 0 {
            return Err(ReaperFunctionError::new("couldn't register thing"));
        }
        Ok(result)
    }

    /// Unregisters things that you have registered with [`plugin_register_add()`].
    ///
    /// Please note that unregistering things manually just for cleaning up is unnecessary in most
    /// situations because *reaper-rs* takes care of automatically unregistering everything when
    /// this struct is dropped (RAII). This happens even when using the unsafe function variants.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    ///
    /// [`plugin_register_add()`]: #method.plugin_register_add
    pub unsafe fn plugin_register_remove(&mut self, object: RegistrationObject) -> i32 {
        self.plugin_register_remove_internal(object.into_raw())
    }

    unsafe fn plugin_register_remove_internal(&mut self, reg: PluginRegistration) -> i32 {
        let name_with_minus = concat_reaper_strs(reaper_str!("-"), reg.key.as_ref());
        let result = self
            .reaper
            .low()
            .plugin_register(name_with_minus.as_ptr(), reg.value);
        self.plugin_registrations.remove(&reg);
        result
    }

    /// Registers a hook command.
    ///
    /// REAPER calls hook commands whenever an action is requested to be run.
    ///
    /// This method doesn't take a closure because REAPER expects a plain function pointer here.
    /// Unlike [`audio_reg_hardware_hook_add`](#method.audio_reg_hardware_hook_add), REAPER
    /// doesn't offer the possibiity to pass a context to the function. So we can't access any
    /// context data in the hook command. You will probably have to use a kind of static
    /// variable which contains command IDs in order to make proper use of this method. The
    /// high-level API makes that much easier (it just takes an arbitrary closure). For the
    /// medium-level API this is out of scope.
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::{HookCommand, CommandId};
    ///
    /// // Usually you would use a dynamic command ID that you have obtained via
    /// // `plugin_register_add_command_id()`. Unfortunately that command ID must be exposed as
    /// // a static variable. The high-level API provides a solution for that.
    /// const MY_COMMAND_ID: CommandId = unsafe { CommandId::new_unchecked(42000) };
    ///
    /// struct MyHookCommand;
    ///
    /// impl HookCommand for MyHookCommand {
    ///     fn call(command_id: CommandId, _flag: i32) -> bool {
    ///         if command_id != MY_COMMAND_ID {
    ///             return false;
    ///         }           
    ///         println!("Executing my command!");
    ///         true
    ///     }
    /// }
    /// session.plugin_register_add_hook_command::<MyHookCommand>();
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Design
    ///
    /// You will note that this method has a somewhat strange signature: It expects a type parameter
    /// only, not a function pointer. That allows us to lift the API to medium-level style.
    /// The alternative would have been to expect a function pointer, but then consumers would have
    /// to deal with raw types.
    pub fn plugin_register_add_hook_command<T: HookCommand>(&mut self) -> ReaperFunctionResult<()> {
        unsafe {
            self.plugin_register_add(RegistrationObject::HookCommand(
                delegating_hook_command::<T>,
            ))?;
        }
        Ok(())
    }

    /// Unregisters a hook command.
    pub fn plugin_register_remove_hook_command<T: HookCommand>(&mut self) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::HookCommand(
                delegating_hook_command::<T>,
            ));
        }
    }

    /// Registers a hook command that supports MIDI CC/mousewheel actions.
    ///
    /// See [`plugin_register_add_hook_command`](#method.plugin_register_add_hook_command) for
    /// understanding how to use this function (it has a very similar design).
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    pub fn plugin_register_add_hook_command_2<T: HookCommand2>(
        &mut self,
    ) -> ReaperFunctionResult<()> {
        unsafe {
            self.plugin_register_add(RegistrationObject::HookCommand2(
                delegating_hook_command_2::<T>,
            ))?;
        }
        Ok(())
    }

    /// Unregisters a hook command that supports MIDI CC/mousewheel actions.
    pub fn plugin_register_remove_hook_command_2<T: HookCommand2>(&mut self) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::HookCommand2(
                delegating_hook_command_2::<T>,
            ));
        }
    }

    /// Registers a custom menu hook.
    ///
    /// See [`plugin_register_add_hook_command`](#method.plugin_register_add_hook_command) for
    /// understanding how to use this function (it has a very similar design).
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    pub fn plugin_register_add_hook_custom_menu<T: HookCustomMenu>(
        &mut self,
    ) -> ReaperFunctionResult<()> {
        unsafe {
            self.plugin_register_add(RegistrationObject::HookCustomMenu(
                delegating_hook_custom_menu::<T>,
            ))?;
        }
        Ok(())
    }

    /// Unregisters a custom menu hook.
    pub fn plugin_register_remove_hook_custom_menu<T: HookCustomMenu>(&mut self) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::HookCustomMenu(
                delegating_hook_custom_menu::<T>,
            ));
        }
    }

    /// Registers a toolbar icon map.
    ///
    /// See [`plugin_register_add_hook_command`](#method.plugin_register_add_hook_command) for
    /// understanding how to use this function (it has a very similar design).
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    pub fn plugin_register_add_toolbar_icon_map<T: ToolbarIconMap>(
        &mut self,
    ) -> ReaperFunctionResult<()> {
        unsafe {
            self.plugin_register_add(RegistrationObject::ToolbarIconMap(
                delegating_toolbar_icon_map::<T>,
            ))?;
        }
        Ok(())
    }

    /// Unregisters a toolbar icon map.
    pub fn plugin_register_remove_toolbar_icon_map<T: ToolbarIconMap>(&mut self) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::ToolbarIconMap(
                delegating_toolbar_icon_map::<T>,
            ));
        }
    }

    /// Registers a toggle action.
    ///
    /// REAPER calls toggle actions whenever it wants to know the on/off state of an action.
    ///
    /// See [`plugin_register_add_hook_command()`](#method.plugin_register_add_hook_command) for an
    /// example.
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    pub fn plugin_register_add_toggle_action<T: ToggleAction>(
        &mut self,
    ) -> ReaperFunctionResult<()> {
        unsafe {
            self.plugin_register_add(RegistrationObject::ToggleAction(
                delegating_toggle_action::<T>,
            ))?
        };
        Ok(())
    }

    /// Unregisters a toggle action.
    pub fn plugin_register_remove_toggle_action<T: ToggleAction>(&mut self) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::ToggleAction(
                delegating_toggle_action::<T>,
            ));
        }
    }

    /// Registers a hook post command.
    ///
    /// REAPER calls hook post commands whenever a normal action of the main section has been
    /// performed.
    ///
    /// See [`plugin_register_add_hook_command()`](#method.plugin_register_add_hook_command) for an
    /// example.
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    pub fn plugin_register_add_hook_post_command<T: HookPostCommand>(
        &mut self,
    ) -> ReaperFunctionResult<()> {
        unsafe {
            self.plugin_register_add(RegistrationObject::HookPostCommand(
                delegating_hook_post_command::<T>,
            ))?
        };
        Ok(())
    }

    /// Unregisters a hook post command.
    pub fn plugin_register_remove_hook_post_command<T: HookPostCommand>(&mut self) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::HookPostCommand(
                delegating_hook_post_command::<T>,
            ));
        }
    }

    /// Registers a hook post command 2.
    ///
    /// REAPER calls hook post commands 2 whenever a MIDI CC/mousewheel action has been performed.
    ///
    /// See [`plugin_register_add_hook_command()`](#method.plugin_register_add_hook_command) for an
    /// example.
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    pub fn plugin_register_add_hook_post_command_2<T: HookPostCommand2>(
        &mut self,
    ) -> ReaperFunctionResult<()> {
        unsafe {
            self.plugin_register_add(RegistrationObject::HookPostCommand2(
                delegating_hook_post_command_2::<T>,
            ))?
        };
        Ok(())
    }

    /// Unregisters a hook post command 2.
    pub fn plugin_register_remove_hook_post_command_2<T: HookPostCommand2>(&mut self) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::HookPostCommand2(
                delegating_hook_post_command_2::<T>,
            ));
        }
    }

    /// Registers a timer.
    ///
    /// The given function will be called regularly until removal. Roughly 30 times per second,
    /// just like the `run()` method of a control surface.
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    pub fn plugin_register_add_timer(
        &mut self,
        timer_func: extern "C" fn(),
    ) -> ReaperFunctionResult<()> {
        unsafe { self.plugin_register_add(RegistrationObject::Timer(timer_func))? };
        Ok(())
    }

    /// Unregisters a timer.
    pub fn plugin_register_remove_timer(&mut self, timer_func: extern "C" fn()) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::Timer(timer_func));
        }
    }

    /// Registers a command ID for the given command name.
    ///
    /// The given command name must be a unique identifier with only A-Z and 0-9.
    ///
    /// Returns the assigned command ID, an ID which is guaranteed to be unique within the current
    /// REAPER session. If the command name is already in use, it just seems to return the ID
    /// which has been assigned before.
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed (e.g. because not supported or out of actions).
    //
    // TODO-low Add function for removing command ID
    pub fn plugin_register_add_command_id<'a>(
        &mut self,
        command_name: impl Into<ReaperStringArg<'a>>,
    ) -> ReaperFunctionResult<CommandId> {
        let owned = command_name.into().into_inner().to_reaper_string();
        let ptr = owned.as_ptr();
        self.command_names.insert(owned);
        let raw_id = unsafe { self.plugin_register_add(RegistrationObject::CommandId(ptr))? };
        Ok(CommandId(raw_id as _))
    }

    /// Unstable!!!
    ///
    /// # Safety
    ///
    /// You must ensure that the given function pointer is valid.
    // TODO-high-unstable Better API (maybe a builder) and doc. Also because current one is prone to
    //  breaking changes.
    // TODO-low Add function for removal
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn plugin_register_add_api_and_def<'a>(
        &mut self,
        function_name: impl Into<ReaperStringArg<'a>>,
        function_ptr: *mut c_void,
        vararg_function_ptr: raw::ApiVararg,
        return_type: impl Into<ReaperStringArg<'a>>,
        argument_types: impl Into<ReaperStringArg<'a>>,
        argument_names: impl Into<ReaperStringArg<'a>>,
        help: impl Into<ReaperStringArg<'a>>,
    ) -> ReaperFunctionResult<()> {
        // Register function
        let function_name = function_name.into().into_inner();
        self.plugin_register_add(RegistrationObject::Api(
            function_name.as_ref().into(),
            function_ptr,
        ))?;
        // Register function definition
        fn to_c_chars<'a>(text: &'a ReaperStringArg) -> impl Iterator<Item = c_char> + 'a {
            text.as_reaper_str()
                .as_c_str()
                .to_bytes_with_nul()
                .iter()
                .map(|c| *c as c_char)
        }
        let null_separated_fields: Vec<c_char> = to_c_chars(&return_type.into())
            .chain(to_c_chars(&argument_types.into()))
            .chain(to_c_chars(&argument_names.into()))
            .chain(to_c_chars(&help.into()))
            .collect();
        let ptr = null_separated_fields.as_ptr();
        self.api_defs.push(null_separated_fields);
        self.plugin_register_add(RegistrationObject::ApiDef(
            function_name.as_ref().into(),
            ptr,
        ))?;
        // Make available to ReaScript
        self.plugin_register_add(RegistrationObject::ApiVararg(
            function_name,
            vararg_function_ptr,
        ))?;
        Ok(())
    }

    /// Registers a an action into the main section.
    ///
    /// This consists of a command ID, a description and a default binding for it. It doesn't
    /// include the actual code to be executed when the action runs (use
    /// [`plugin_register_add_hook_command()`] for that).
    ///
    /// This function returns a handle which you can use to unregister the action at any time via
    /// [`plugin_register_remove_gaccel()`].
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    ///
    /// # Design
    ///
    /// This function takes ownership of the passed struct in order to take complete care of it.
    /// Compared to the alternative of taking a reference or pointer, that releases the API
    /// consumer from the responsibilities to guarantee a long enough lifetime and to maintain a
    /// stable address in memory. Giving up ownership also means that the consumer doesn't have
    /// access to the struct anymore - which is a good thing, because REAPER should be the new
    /// rightful owner of this struct. Thanks to this we don't need to mark this function as
    /// unsafe!
    ///
    /// [`plugin_register_add_hook_command()`]: #method.plugin_register_add_hook_command
    /// [`plugin_register_remove_gaccel()`]: #method.plugin_register_remove_gaccel
    pub fn plugin_register_add_gaccel(
        &mut self,
        register: OwnedGaccelRegister,
    ) -> ReaperFunctionResult<Handle<raw::gaccel_register_t>> {
        let handle = self.gaccel_registers.keep(register);
        unsafe { self.plugin_register_add(RegistrationObject::Gaccel(handle))? };
        Ok(handle)
    }

    /// Like [`Self::plugin_register_add_gaccel`] but registers shortcut globally, except if text field focused.
    pub fn plugin_register_add_gaccel_global(
        &mut self,
        register: OwnedGaccelRegister,
    ) -> Result<Handle<raw::gaccel_register_t>, OwnedGaccelRegister> {
        let handle = self.gaccel_registers.keep(register);
        let result = unsafe { self.plugin_register_add(RegistrationObject::GaccelGlobal(handle)) };
        result
            .map(|_| handle)
            .map_err(|_| self.gaccel_registers.release(handle).unwrap())
    }

    /// Like [`Self::plugin_register_add_gaccel`] but registers shortcut globally, even if text field focused.
    pub fn plugin_register_add_gaccel_global_text(
        &mut self,
        register: OwnedGaccelRegister,
    ) -> Result<Handle<raw::gaccel_register_t>, OwnedGaccelRegister> {
        let handle = self.gaccel_registers.keep(register);
        let result =
            unsafe { self.plugin_register_add(RegistrationObject::GaccelGlobalText(handle)) };
        result
            .map(|_| handle)
            .map_err(|_| self.gaccel_registers.release(handle).unwrap())
    }

    pub fn plugin_register_add_accelerator_register<T>(
        &mut self,
        callback: Box<T>,
        position: AcceleratorPosition,
    ) -> ReaperFunctionResult<RegistrationHandle<T>>
    where
        T: TranslateAccel + 'static,
    {
        // Create thin pointer of callback before making it a trait object (for being able to
        // restore the original callback later).
        let callback_thin_ptr: NonNull<T> = callback.as_ref().into();
        // Create accelerator register and make it own the callback (as user data)
        let register = OwnedAcceleratorRegister::new(callback);
        // Store it in memory.  Although we keep it here, conceptually it's owned by REAPER, so we
        // should not access it while being registered.
        let reaper_ptr = self.accelerator_registers.keep(register);
        // Register the low-level register at REAPER
        let reg = match position {
            AcceleratorPosition::Front => RegistrationObject::FrontAccelerator(reaper_ptr),
            AcceleratorPosition::Back => RegistrationObject::BackAccelerator(reaper_ptr),
        };
        unsafe { self.plugin_register_add(reg)? };
        // Returns a handle which the consumer can use to unregister
        let handle = RegistrationHandle::new(callback_thin_ptr, reaper_ptr.cast());
        Ok(handle)
    }

    pub fn plugin_register_add_file_in_project_callback<'a, T>(
        &mut self,
        project: ReaProject,
        file_name: impl Into<ReaperStringArg<'a>>,
        callback: Box<T>,
    ) -> ReaperFunctionResult<GenericRegistrationHandle<usize, T>>
    where
        T: FileInProjectCallback + 'static,
    {
        // Create thin pointer of callback before making it a trait object (for being able to
        // restore the original callback later).
        let callback_thin_ptr: NonNull<T> = callback.as_ref().into();
        // Create hook and make it own the callback (as user data)
        let hook = OwnedFileInProjectHook::new(
            project,
            file_name.into().into_inner().to_reaper_string(),
            callback,
        );
        // Store it in memory.  Although we keep it here, conceptually it's owned by REAPER, so we
        // should not access it while being registered.
        let (handle, hook) = self.file_in_project_hooks.keep(hook);
        // Important to call this *after* it has been added to the keeper (because only then it is at its final
        // place in memory and create the correct pointers).
        let plugin_register_arg = hook.create_plugin_register_arg();
        // Register the low-level hook at REAPER
        let reg = RegistrationObject::FileInProjectCallback(Handle::new(NonNull::from(
            &plugin_register_arg,
        )));
        let result = unsafe { self.plugin_register_add(reg)? };
        tracing::debug!(
            msg = "Added file-in-project callback",
            result,
            ?plugin_register_arg
        );
        // Returns a handle which the consumer can use to unregister
        Ok(GenericRegistrationHandle::new(callback_thin_ptr, handle))
    }

    /// Plays a preview register.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid preview pointer, if the pointer gets stale while
    /// still playing, if you don't properly handle synchronization via mutex or critical section
    /// when modifying the register while playing. Use [`play_preview_ex()`] if you want to be
    /// released from that burden.
    ///
    /// [`play_preview_ex()`]: #method.play_preview_ex
    pub unsafe fn play_preview_ex_unchecked(
        &mut self,
        preview: Handle<raw::preview_register_t>,
        buffering_behavior: BitFlags<BufferingBehavior>,
        measure_alignment: MeasureAlignment,
    ) -> ReaperFunctionResult<()> {
        self.playing_preview_registers.insert(preview);
        let result = self.reaper.low().PlayPreviewEx(
            preview.as_ptr(),
            buffering_behavior.bits() as i32,
            measure_alignment.to_raw(),
        );
        if result == 0 {
            return Err(ReaperFunctionError::new("couldn't play preview"));
        }
        Ok(())
    }

    /// Stops a preview that you have played with [`play_preview_ex_unchecked()`].
    ///
    /// Please note that stopping preview registers manually just for cleaning up is
    /// unnecessary in most situations because *reaper-rs* takes care of automatically
    /// unregistering everything when this struct is dropped (RAII). This happens even when using
    /// the unsafe function variants.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    ///
    /// [`play_preview_ex_unchecked()`]: #method.play_preview_ex_unchecked
    pub unsafe fn stop_preview_unchecked(
        &mut self,
        register: Handle<raw::preview_register_t>,
    ) -> ReaperFunctionResult<()> {
        let successful = self.reaper.low().StopPreview(register.as_ptr());
        // If not successful, it usually means the preview is stopped already. Let's remove
        // the handle now so that we don't try stopping again when dropping the session.
        self.playing_preview_registers.remove(&register);
        if successful == 0 {
            return Err(ReaperFunctionError::new("couldn't stop preview"));
        }
        Ok(())
    }

    /// Plays a preview register on a specific track.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project or preview pointer, if the pointer gets
    /// stale while still playing, if you don't properly handle synchronization via mutex or
    /// critical section when modifying the register while playing. Use
    /// [`play_track_preview_2_ex()`] if you want to be released from that burden.
    ///
    /// [`play_track_preview_2_ex()`]: #method.play_track_preview_2_ex
    pub unsafe fn play_track_preview_2_ex_unchecked(
        &mut self,
        project: ProjectContext,
        preview: Handle<raw::preview_register_t>,
        buffering_behavior: BitFlags<BufferingBehavior>,
        measure_alignment: MeasureAlignment,
    ) -> ReaperFunctionResult<()> {
        self.playing_track_preview_registers
            .insert((project, preview));
        let result = self.reaper.low().PlayTrackPreview2Ex(
            project.to_raw(),
            preview.as_ptr(),
            buffering_behavior.bits() as i32,
            measure_alignment.to_raw(),
        );
        if result == 0 {
            return Err(ReaperFunctionError::new("couldn't play track preview"));
        }
        Ok(())
    }

    /// Stops a preview that you have played with [`play_track_preview_2_ex_unchecked()`].
    ///
    /// Please note that stopping preview registers manually just for cleaning up is
    /// unnecessary in most situations because *reaper-rs* takes care of automatically
    /// unregistering everything when this struct is dropped (RAII). This happens even when using
    /// the unsafe function variants.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid register pointer.
    ///
    /// [`play_track_preview_2_ex_unchecked()`]: #method.play_track_preview_2_ex_unchecked
    pub unsafe fn stop_track_preview_2_unchecked(
        &mut self,
        project: ProjectContext,
        register: Handle<raw::preview_register_t>,
    ) -> ReaperFunctionResult<()> {
        let successful = self
            .reaper
            .low()
            .StopTrackPreview2(project.to_raw() as _, register.as_ptr());
        // If not successful, it usually means the preview is stopped already. Let's remove
        // the handle now so that we don't try stopping again when dropping the session.
        self.playing_track_preview_registers
            .remove(&(project, register));
        if successful == 0 {
            return Err(ReaperFunctionError::new("couldn't stop track preview"));
        }
        Ok(())
    }

    /// Plays a preview register.
    ///
    /// It asks for a shared mutex-protected register because it assumes you want to keep
    /// controlling the playback. With the mutex you can safely modify the register on-the-fly while
    /// it's being played by REAPER.
    ///
    /// Returns a handle which is necessary to stop the preview at a later time.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful.
    pub fn play_preview_ex(
        &mut self,
        register: Arc<ReaperMutex<OwnedPreviewRegister>>,
        buffering_behavior: BitFlags<BufferingBehavior>,
        measure_alignment: MeasureAlignment,
    ) -> ReaperFunctionResult<Handle<raw::preview_register_t>> {
        let handle = self.preview_registers.keep(register);
        unsafe { self.play_preview_ex_unchecked(handle, buffering_behavior, measure_alignment)? };
        Ok(handle)
    }

    /// Stops a preview that you have played with [`play_preview_ex()`].
    ///
    /// # Errors
    ///
    /// Returns an error if not successful (e.g. was not playing).
    ///
    /// [`play_preview_ex()`]: #method.play_preview_ex
    pub fn stop_preview(
        &mut self,
        handle: Handle<raw::preview_register_t>,
    ) -> ReaperFunctionResult<()> {
        let result = unsafe { self.stop_preview_unchecked(handle) };
        // If stopping was not successful, it usually means that the preview was not playing
        // anymore, e.g. because the track was removed already. In that case we still need to
        // clean up, otherwise we have a leak.
        self.preview_registers.release(handle);
        result
    }

    /// Plays a preview register on a specific track.
    ///
    /// It asks for a shared mutex-protected register because it assumes you want to keep
    /// controlling the playback. With the mutex you can safely modify the register on-the-fly while
    /// it's being played by REAPER.
    ///
    /// Returns a handle which is necessary to stop the preview at a later time.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful.
    pub fn play_track_preview_2_ex(
        &mut self,
        project: ProjectContext,
        register: Arc<ReaperMutex<OwnedPreviewRegister>>,
        buffering_behavior: BitFlags<BufferingBehavior>,
        measure_alignment: MeasureAlignment,
    ) -> ReaperFunctionResult<Handle<raw::preview_register_t>> {
        self.reaper.require_valid_project(project);
        let handle = self.preview_registers.keep(register);
        unsafe {
            self.play_track_preview_2_ex_unchecked(
                project,
                handle,
                buffering_behavior,
                measure_alignment,
            )?
        };
        Ok(handle)
    }

    /// Stops a preview that you have played with [`play_track_preview_2_ex()`].
    ///
    /// # Errors
    ///
    /// Returns an error if not successful (e.g. was not playing).
    ///
    /// [`play_track_preview_2_ex()`]: #method.play_track_preview_2_ex
    pub fn stop_track_preview_2(
        &mut self,
        project: ProjectContext,
        handle: Handle<raw::preview_register_t>,
    ) -> ReaperFunctionResult<()> {
        // It's important we don't just panic when the given project is invalid because that would
        // force the consumer to check the project validity before and not call this method if the
        // project is invalid. However, that would leak the preview register.
        let result = if self.reaper.project_is_valid(project) {
            unsafe { self.stop_track_preview_2_unchecked(project, handle) }
        } else {
            Err(ReaperFunctionError::new(
                "project not valid anymore, preview stopped already",
            ))
        };
        // If stopping was not successful, it usually means that the preview was not playing
        // anymore, e.g. because the track was removed already. In that case we still need to
        // clean up, otherwise we have a leak.
        self.preview_registers.release(handle);
        result
    }

    /// Unregisters an action.
    pub fn plugin_register_remove_gaccel(&mut self, handle: Handle<raw::gaccel_register_t>) {
        unsafe { self.plugin_register_remove(RegistrationObject::Gaccel(handle)) };
    }

    pub fn plugin_register_remove_gaccel_global(&mut self, handle: Handle<raw::gaccel_register_t>) {
        unsafe { self.plugin_register_remove(RegistrationObject::GaccelGlobal(handle)) };
    }

    pub fn plugin_register_remove_gaccel_global_text(
        &mut self,
        handle: Handle<raw::gaccel_register_t>,
    ) {
        unsafe { self.plugin_register_remove(RegistrationObject::GaccelGlobalText(handle)) };
    }

    pub fn plugin_register_remove_accelerator<T>(
        &mut self,
        handle: RegistrationHandle<T>,
    ) -> Option<Box<T>>
    where
        T: TranslateAccel,
    {
        // Unregister the low-level register from REAPER
        let reaper_ptr = handle.key().cast();
        unsafe { self.plugin_register_remove(RegistrationObject::BackAccelerator(reaper_ptr)) };
        // Take the owned register out of its storage
        let owned_register = self.accelerator_registers.release(handle.key().cast())?;
        // Reconstruct the initial value for handing ownership back to the consumer
        let dyn_callback = owned_register.into_callback();
        // We are not interested in the fat pointer (Box<dyn TranslateAccel>) anymore.
        // By calling leak(), we make the pointer go away but prevent Rust from
        // dropping its content.
        Box::leak(dyn_callback);
        // Here we pick up the content again and treat it as a Box - but this
        // time not a trait object box (Box<dyn TranslateAccel> = fat pointer) but a
        // normal box (Box<T> = thin pointer) ... original type restored.
        let callback = unsafe { handle.restore_original() };
        Some(callback)
    }

    /// If unregistering successful, returns the number of remaining usages of that file.
    pub fn plugin_register_remove_file_in_project_callback<T>(
        &mut self,
        handle: GenericRegistrationHandle<usize, T>,
    ) -> ReaperFunctionResult<u32>
    where
        T: FileInProjectCallback + 'static,
    {
        // Take the owned hook out of its storage
        let owned_hook = self
            .file_in_project_hooks
            .get(handle.key())
            .ok_or(ReaperFunctionError::new("hook not available anymore in reaper-rs, and therefore can't be registered anymore in REAPER as well"))?;
        let plugin_register_arg = owned_hook.create_plugin_register_arg();
        // Unregister the low-level register from REAPER
        let result = unsafe {
            self.plugin_register_remove(RegistrationObject::FileInProjectCallback(Handle::new(
                NonNull::from(&plugin_register_arg),
            )))
        };
        // Release *after* unregistering from REAPER
        let _ = self.file_in_project_hooks.release(handle.key());
        tracing::debug!(
            msg = "Removed file-in-project callback",
            result,
            ?plugin_register_arg
        );
        if result < 256 {
            return Err(ReaperFunctionError::new(
                "project file combination was not registered anymore in REAPER",
            ));
        }
        Ok(result as u32 - 256)
    }

    /// Registers a hidden control surface.
    ///
    /// This is very useful for being notified by REAPER about all kinds of events in the main
    /// thread.
    ///
    /// This function returns a handle which you can use to unregister the control surface at any
    /// time via [`plugin_register_remove_csurf_inst()`].
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::ControlSurface;
    ///
    /// #[derive(Debug)]
    /// struct MyControlSurface;
    ///
    /// impl ControlSurface for MyControlSurface {
    ///     fn set_track_list_change(&self) {
    ///         println!("Tracks changed");
    ///     }
    /// }
    /// session.plugin_register_add_csurf_inst(Box::new(MyControlSurface));
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`plugin_register_remove_csurf_inst()`]: #method.plugin_register_remove_csurf_inst
    pub fn plugin_register_add_csurf_inst<T>(
        &mut self,
        control_surface: Box<T>,
    ) -> ReaperFunctionResult<RegistrationHandle<T>>
    where
        T: ControlSurface + 'static,
    {
        // Create thin pointer of control_surface before making it a trait object (for being able to
        // restore the original control_surface later).
        let control_surface_thin_ptr: NonNull<T> = control_surface.as_ref().into();
        // Create low-level Rust control surface which delegates to the medium-level one.
        let low_cs = ControlSurfaceAdapter::new(control_surface, &self.reaper.get_app_version());
        // Create the C++ counterpart surface (we need to box the Rust side twice in order to obtain
        // a thin pointer for passing it to C++ as callback target).
        let double_boxed_low_cs: Box<Box<dyn IReaperControlSurface>> = Box::new(Box::new(low_cs));
        let cpp_cs =
            unsafe { create_cpp_to_rust_control_surface(double_boxed_low_cs.as_ref().into()) };
        let cpp_cs = Handle::new(cpp_cs);
        // Store the low-level Rust control surface in memory. Although we keep it here,
        // conceptually it's owned by REAPER, so we should not access it while being registered.
        let handle = RegistrationHandle::new(control_surface_thin_ptr, cpp_cs.cast());
        self.csurf_insts.insert(handle.key(), double_boxed_low_cs);
        // Register the C++ control surface at REAPER
        unsafe { self.plugin_register_add(RegistrationObject::CsurfInst(cpp_cs))? };
        // Return a handle which the consumer can use to unregister
        Ok(handle)
    }

    /// Unregisters a hidden control surface and hands ownership back to you.
    ///
    /// If the control surface is not registered, this function just returns `None`.
    ///
    /// This only needs to be called if you explicitly want the control surface to "stop" while
    /// your plug-in is still running. You don't need to call this for cleaning up because this
    /// struct takes care of unregistering everything safely when it gets dropped.
    ///
    /// # Safety
    ///
    /// As soon as the returned control surface goes out of scope, it is removed from memory.
    /// If you don't intend to keep the return value around longer, you should be absolutely sure
    /// that your control surface is not currently executing any function. Because both this
    /// function and any control surface function can only be called by the main thread, this
    /// effectively means you must make sure that this removal function is not called by a
    /// control surface method itself. That would be like pulling the rug out from under your
    /// feet!
    ///
    /// This scenario is not hypothetical: The control surface `run()` method is very suitable for
    /// processing arbitrary tasks which it receives via a channel. Let's say one of these arbitrary
    /// tasks calls this removal function. It's guaranteed that REAPER will not call the `run()`
    /// function anymore after that, yes. But, the `run()` function itself might not be finished
    /// yet and pull another task from the receiver ... oops. The receiver is not there anymore
    /// because it was owned by the control surface and therefore removed from memory as well.
    /// This will lead to a crash.
    ///
    /// Ideally, REAPER would _really_ own this control surface, including managing its lifetime.
    /// Then REAPER would remove it as soon as the `run()` function returns. But this is not how
    /// the REAPER API works. We must manage the control surface lifetime for REAPER.
    #[must_use]
    pub unsafe fn plugin_register_remove_csurf_inst<T>(
        &mut self,
        handle: RegistrationHandle<T>,
    ) -> Option<Box<T>>
    where
        T: ControlSurface,
    {
        // Take the low-level Rust control surface out of its storage
        let double_boxed_low_cs = self.csurf_insts.remove(&handle.key())?;
        // Unregister the C++ control surface from REAPER
        let cpp_cs_ptr = handle.key().cast();
        self.plugin_register_remove(RegistrationObject::CsurfInst(cpp_cs_ptr));
        // Remove the C++ counterpart surface
        delete_cpp_control_surface(cpp_cs_ptr.get());
        // Reconstruct the initial value for handing ownership back to the consumer
        let low_cs = double_boxed_low_cs
            .into_any()
            .downcast::<ControlSurfaceAdapter>()
            .ok()?;
        let dyn_control_surface = low_cs.into_delegate();
        // We are not interested in the fat pointer (Box<dyn ControlSurface>) anymore.
        // By calling leak(), we make the pointer go away but prevent Rust from
        // dropping its content.
        Box::leak(dyn_control_surface);
        // Here we pick up the content again and treat it as a Box - but this
        // time not a trait object box (Box<dyn ControlSurface> = fat pointer) but a
        // normal box (Box<T> = thin pointer) ... original type restored.
        let control_surface = handle.restore_original();
        Some(control_surface)
    }

    /// Like [`audio_reg_hardware_hook_add`] but doesn't manage memory for you.
    ///
    /// Also see [`audio_reg_hardware_hook_remove_unchecked()`].
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer or if it dangles during the time it
    /// is registered. So you must ensure that the audio hook register lives long enough and
    /// has a stable address in memory. Additionally, incorrectly accessing the audio hook register
    /// while it is registered can lead to horrible race conditions and other undefined
    /// behavior.
    ///
    /// [`audio_reg_hardware_hook_remove_unchecked()`]:
    /// #method.audio_reg_hardware_hook_remove_unchecked
    /// [`audio_reg_hardware_hook_add`]: #method.audio_reg_hardware_hook_add
    pub unsafe fn audio_reg_hardware_hook_add_unchecked(
        &mut self,
        register: Handle<audio_hook_register_t>,
    ) -> ReaperFunctionResult<()> {
        self.audio_hook_registrations.insert(register);
        let result = self
            .reaper
            .low()
            .Audio_RegHardwareHook(true, register.as_ptr());
        if result == 0 {
            return Err(ReaperFunctionError::new("couldn't register audio hook"));
        }
        Ok(())
    }

    /// Unregisters the audio hook register that you have registered with
    /// [`audio_reg_hardware_hook_add_unchecked()`].
    ///
    /// Please note that unregistering audio hook registers manually just for cleaning up is
    /// unnecessary in most situations because *reaper-rs* takes care of automatically
    /// unregistering everything when this struct is dropped (RAII). This happens even when using
    /// the unsafe function variants.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid pointer.
    ///
    /// [`audio_reg_hardware_hook_add_unchecked()`]: #method.audio_reg_hardware_hook_add_unchecked
    pub unsafe fn audio_reg_hardware_hook_remove_unchecked(
        &mut self,
        register: Handle<audio_hook_register_t>,
    ) {
        self.reaper
            .low()
            .Audio_RegHardwareHook(false, register.as_ptr());
        self.audio_hook_registrations.remove(&register);
    }

    /// Registers an audio hook register.
    ///
    /// This allows you to get called back in the real-time audio thread before and after REAPER's
    /// processing. You should be careful with this because you are entering real-time world.
    ///
    /// This function returns a handle which you can use to unregister the audio hook register at
    /// any time via [`audio_reg_hardware_hook_remove()`] (from the main thread).
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::{
    ///     ControlSurface, OnAudioBuffer, OnAudioBufferArgs,
    ///     Reaper, RealTimeAudioThreadScope, MidiInputDeviceId
    /// };
    ///
    /// struct MyOnAudioBuffer {
    ///     counter: u64,
    ///     reaper: Reaper<RealTimeAudioThreadScope>,
    /// }
    ///
    /// impl OnAudioBuffer for MyOnAudioBuffer {
    ///     fn call(&mut self, args: OnAudioBufferArgs) {
    ///         // Mutate some own state (safe because we are the owner)
    ///         if self.counter % 100 == 0 {
    ///             println!("Audio hook callback counter: {}\n", self.counter);
    ///         }
    ///         self.counter += 1;
    ///         // Read some MIDI events
    ///         self.reaper.get_midi_input(MidiInputDeviceId::new(0), |input| -> Option<()> {
    ///             for event in input?.get_read_buf().enum_items(0) {
    ///                 println!("Received MIDI event {:?}", event);
    ///             }
    ///             Some(())
    ///         });
    ///     }
    /// }
    ///
    /// session.audio_reg_hardware_hook_add(Box::new(MyOnAudioBuffer {
    ///     counter: 0,
    ///     reaper: session.create_real_time_reaper()
    /// }));
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`audio_reg_hardware_hook_remove()`]: #method.audio_reg_hardware_hook_remove
    pub fn audio_reg_hardware_hook_add<T>(
        &mut self,
        callback: Box<T>,
    ) -> ReaperFunctionResult<RegistrationHandle<T>>
    where
        T: OnAudioBuffer + 'static,
    {
        // Create thin pointer of callback before making it a trait object (for being able to
        // restore the original callback later).
        let callback_thin_ptr: NonNull<T> = callback.as_ref().into();
        // Create owned audio hook register and make it own the callback (as user data)
        let register = OwnedAudioHookRegister::new(callback);
        // Store it in memory.  Although we keep it here, conceptually it's owned by REAPER, so we
        // should not access it while being registered.
        let reaper_ptr = self.audio_hook_registers.keep(register);
        // Register the low-level audio hook register at REAPER
        unsafe { self.audio_reg_hardware_hook_add_unchecked(reaper_ptr)? };
        // Return a handle which the consumer can use to unregister
        let handle = RegistrationHandle::new(callback_thin_ptr, reaper_ptr.cast());
        Ok(handle)
    }

    /// Unregisters an audio hook register and hands ownership back to you.
    ///
    /// If the audio hook register is not registered, this function just returns `None`.
    ///
    /// This only needs to be called if you explicitly want the audio hook to "stop" while your
    /// plug-in is still running. You don't need to call this for cleaning up because this
    /// struct takes care of unregistering everything safely when it gets dropped.
    ///
    /// REAPER guarantees via proper synchronization that after this method returns, the callback
    /// is not in the process of being called and also will not be called anymore. However, it is
    /// *not* guaranteed that the last callback invocation has `is_post == true`.
    pub fn audio_reg_hardware_hook_remove<T>(
        &mut self,
        handle: RegistrationHandle<T>,
    ) -> Option<Box<T>>
    where
        T: OnAudioBuffer,
    {
        // Unregister the low-level audio hook register from REAPER
        let reaper_ptr = handle.key().cast();
        unsafe { self.audio_reg_hardware_hook_remove_unchecked(reaper_ptr) };
        // Take the owned audio hook register out of its storage
        let owned_audio_hook_register = self.audio_hook_registers.release(handle.key().cast())?;
        // Reconstruct the initial value for handing ownership back to the consumer
        let dyn_callback = owned_audio_hook_register.into_callback();
        // We are not interested in the fat pointer (Box<dyn OnAudioBuffer>) anymore.
        // By calling leak(), we make the pointer go away but prevent Rust from
        // dropping its content.
        Box::leak(dyn_callback);
        // Here we pick up the content again and treat it as a Box - but this
        // time not a trait object box (Box<dyn OnAudioBuffer> = fat pointer) but a
        // normal box (Box<T> = thin pointer) ... original type restored.
        let callback = unsafe { handle.restore_original() };
        Some(callback)
    }
}

impl Drop for ReaperSession {
    fn drop(&mut self) {
        for (project, handle) in self.playing_track_preview_registers.clone() {
            unsafe {
                let _ = self.stop_track_preview_2_unchecked(project, handle);
            }
        }
        for handle in self.playing_preview_registers.clone() {
            unsafe {
                let _ = self.stop_preview_unchecked(handle);
            }
        }
        for handle in self.audio_hook_registrations.clone() {
            unsafe {
                self.audio_reg_hardware_hook_remove_unchecked(handle);
            }
        }
        for reg in self.plugin_registrations.clone() {
            unsafe {
                self.plugin_register_remove_internal(reg);
            }
        }
    }
}
