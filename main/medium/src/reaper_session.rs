use c_str_macro::c_str;

use std::ptr::NonNull;

use reaper_low::{
    add_cpp_control_surface, raw, remove_cpp_control_surface, IReaperControlSurface,
    ReaperPluginContext,
};

use crate::infostruct_keeper::InfostructKeeper;

use crate::{
    concat_reaper_strs, delegating_hook_command, delegating_hook_post_command,
    delegating_toggle_action, CommandId, DelegatingControlSurface, MainThreadScope,
    MediumAudioHookRegister, MediumGaccelRegister, MediumHookCommand, MediumHookPostCommand,
    MediumOnAudioBuffer, MediumReaperControlSurface, MediumToggleAction, RealTimeAudioThreadScope,
    Reaper, ReaperFunctionError, ReaperFunctionResult, ReaperStringArg, RegistrationObject,
};
use reaper_low::raw::audio_hook_register_t;
use std::collections::{HashMap, HashSet};

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
// TODO-medium Add some doc from https://www.reaper.fm/sdk/vst/vst_ext.php
// TODO-medium Lift low-level ReaperPluginContext functions to medium-level style. Especially the
//  VST host context stuff: https://www.reaper.fm/sdk/vst/vst_ext.php.
#[derive(Debug, Default)]
pub struct ReaperSession {
    reaper: Reaper<MainThreadScope>,
    gaccel_registers: InfostructKeeper<MediumGaccelRegister, raw::gaccel_register_t>,
    audio_hook_registers: InfostructKeeper<MediumAudioHookRegister, raw::audio_hook_register_t>,
    csurf_insts: HashMap<NonNull<raw::IReaperControlSurface>, Box<Box<dyn IReaperControlSurface>>>,
    plugin_registrations: HashSet<RegistrationObject<'static>>,
    audio_hook_registrations: HashSet<NonNull<raw::audio_hook_register_t>>,
}

impl ReaperSession {
    /// Creates a new instance by getting hold of a [low-level `Reaper`] instance.
    ///
    /// [low-level `Reaper`]: https://docs.rs/reaper-low
    pub fn new(low: reaper_low::Reaper) -> ReaperSession {
        ReaperSession {
            reaper: Reaper::new(low),
            gaccel_registers: Default::default(),
            audio_hook_registers: Default::default(),
            csurf_insts: Default::default(),
            plugin_registrations: Default::default(),
            audio_hook_registrations: Default::default(),
        }
    }

    /// Loads all available REAPER functions from the given plug-in context.
    ///
    /// Returns a medium-level `ReaperSession` instance which allows you to call these functions.
    pub fn load(context: ReaperPluginContext) -> ReaperSession {
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
    /// most cases it just returns 1. In any case it's not 0, *reaper-rs* translates this into an
    /// error.
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
        self.plugin_registrations
            .insert(object.clone().into_owned());
        let infostruct = object.ptr_to_raw();
        let result = self
            .reaper
            .low()
            .plugin_register(object.key_into_raw().as_ptr(), infostruct);
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
        let infostruct = object.ptr_to_raw();
        let name_with_minus =
            concat_reaper_strs(reaper_str!("-"), object.clone().key_into_raw().as_ref());
        let result = self
            .reaper
            .low()
            .plugin_register(name_with_minus.as_ptr(), infostruct);
        self.plugin_registrations.remove(&object.into_owned());
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
    /// use reaper_medium::{MediumHookCommand, CommandId};
    ///
    /// // Usually you would use a dynamic command ID that you have obtained via
    /// // `plugin_register_add_command_id()`. Unfortunately that command ID must be exposed as
    /// // a static variable. The high-level API provides a solution for that.
    /// const MY_COMMAND_ID: CommandId = unsafe { CommandId::new_unchecked(42000) };
    ///
    /// struct MyHookCommand;
    ///
    /// impl MediumHookCommand for MyHookCommand {
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
    pub fn plugin_register_add_hook_command<T: MediumHookCommand>(
        &mut self,
    ) -> ReaperFunctionResult<()> {
        unsafe {
            self.plugin_register_add(RegistrationObject::HookCommand(
                delegating_hook_command::<T>,
            ))?;
        }
        Ok(())
    }

    /// Unregisters a hook command.
    pub fn plugin_register_remove_hook_command<T: MediumHookCommand>(&mut self) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::HookCommand(
                delegating_hook_command::<T>,
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
    pub fn plugin_register_add_toggle_action<T: MediumToggleAction>(
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
    pub fn plugin_register_remove_toggle_action<T: MediumToggleAction>(&mut self) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::ToggleAction(
                delegating_toggle_action::<T>,
            ));
        }
    }

    /// Registers a hook post command.
    ///
    /// REAPER calls hook post commands whenever an action of the main section has been performed.
    ///
    /// See [`plugin_register_add_hook_command()`](#method.plugin_register_add_hook_command) for an
    /// example.
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    pub fn plugin_register_add_hook_post_command<T: MediumHookPostCommand>(
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
    pub fn plugin_register_remove_hook_post_command<T: MediumHookPostCommand>(&mut self) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::HookPostCommand(
                delegating_hook_post_command::<T>,
            ));
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
    pub fn plugin_register_add_command_id<'a>(
        &mut self,
        command_name: impl Into<ReaperStringArg<'a>>,
    ) -> ReaperFunctionResult<CommandId> {
        let raw_id = unsafe {
            self.plugin_register_add(RegistrationObject::CommandId(
                command_name.into().into_inner(),
            ))?
        };
        Ok(CommandId(raw_id as _))
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
        register: MediumGaccelRegister,
    ) -> ReaperFunctionResult<NonNull<raw::gaccel_register_t>> {
        let handle = self.gaccel_registers.keep(register);
        unsafe { self.plugin_register_add(RegistrationObject::Gaccel(handle))? };
        Ok(handle)
    }

    /// Unregisters an action.
    pub fn plugin_register_remove_gaccel(&mut self, handle: NonNull<raw::gaccel_register_t>) {
        unsafe { self.plugin_register_remove(RegistrationObject::Gaccel(handle)) };
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
    /// use reaper_medium::MediumReaperControlSurface;
    ///
    /// #[derive(Debug)]
    /// struct MyControlSurface;
    ///
    /// impl MediumReaperControlSurface for MyControlSurface {
    ///     fn set_track_list_change(&self) {
    ///         println!("Tracks changed");
    ///     }
    /// }
    /// session.plugin_register_add_csurf_inst(MyControlSurface);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`plugin_register_remove_csurf_inst()`]: #method.plugin_register_remove_csurf_inst
    pub fn plugin_register_add_csurf_inst(
        &mut self,
        control_surface: impl MediumReaperControlSurface + 'static,
    ) -> ReaperFunctionResult<NonNull<raw::IReaperControlSurface>> {
        let rust_control_surface =
            DelegatingControlSurface::new(control_surface, &self.reaper.get_app_version());
        // We need to box it twice in order to obtain a thin pointer for passing to C as callback
        // target
        let rust_control_surface: Box<Box<dyn IReaperControlSurface>> =
            Box::new(Box::new(rust_control_surface));
        let cpp_control_surface =
            unsafe { add_cpp_control_surface(rust_control_surface.as_ref().into()) };
        self.csurf_insts
            .insert(cpp_control_surface, rust_control_surface);
        unsafe { self.plugin_register_add(RegistrationObject::CsurfInst(cpp_control_surface))? };
        Ok(cpp_control_surface)
    }

    /// Unregisters a hidden control surface.
    pub fn plugin_register_remove_csurf_inst(
        &mut self,
        handle: NonNull<raw::IReaperControlSurface>,
    ) {
        unsafe {
            self.plugin_register_remove(RegistrationObject::CsurfInst(handle));
        }
        self.csurf_insts.remove(&handle);
        unsafe {
            remove_cpp_control_surface(handle);
        }
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
        register: NonNull<audio_hook_register_t>,
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
        register: NonNull<audio_hook_register_t>,
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
    ///     MediumReaperControlSurface, MediumOnAudioBuffer, OnAudioBufferArgs,
    ///     Reaper, RealTimeAudioThreadScope, MidiInputDeviceId
    /// };
    ///
    /// struct MyOnAudioBuffer {
    ///     counter: u64,
    ///     reaper: Reaper<RealTimeAudioThreadScope>,
    /// }
    ///
    /// impl MediumOnAudioBuffer for MyOnAudioBuffer {
    ///     fn call(&mut self, args: OnAudioBufferArgs) {
    ///         // Mutate some own state (safe because we are the owner)
    ///         if self.counter % 100 == 0 {
    ///             println!("Audio hook callback counter: {}\n", self.counter);
    ///         }
    ///         self.counter += 1;
    ///         // Read some MIDI events
    ///         self.reaper.get_midi_input(MidiInputDeviceId::new(0), |input| {
    ///             for event in input.get_read_buf().enum_items(0) {
    ///                 println!("Received MIDI event {:?}", event);
    ///             }   
    ///         });
    ///     }
    /// }
    ///
    /// session.audio_reg_hardware_hook_add(MyOnAudioBuffer {
    ///     counter: 0,
    ///     reaper: session.create_real_time_reaper()
    /// });
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// [`audio_reg_hardware_hook_remove()`]: #method.audio_reg_hardware_hook_remove
    pub fn audio_reg_hardware_hook_add<T: MediumOnAudioBuffer + 'static>(
        &mut self,
        callback: T,
    ) -> ReaperFunctionResult<NonNull<audio_hook_register_t>> {
        let handle = self
            .audio_hook_registers
            .keep(MediumAudioHookRegister::new(callback));
        unsafe { self.audio_reg_hardware_hook_add_unchecked(handle)? };
        Ok(handle)
    }

    /// Unregisters an audio hook register.
    pub fn audio_reg_hardware_hook_remove(&mut self, handle: NonNull<audio_hook_register_t>) {
        unsafe { self.audio_reg_hardware_hook_remove_unchecked(handle) };
        let _ = self.audio_hook_registers.release(handle);
    }
}

impl Drop for ReaperSession {
    fn drop(&mut self) {
        for handle in self.audio_hook_registrations.clone() {
            unsafe {
                self.audio_reg_hardware_hook_remove_unchecked(handle);
            }
        }
        for reg in self.plugin_registrations.clone() {
            unsafe {
                self.plugin_register_remove(reg);
            }
        }
    }
}
