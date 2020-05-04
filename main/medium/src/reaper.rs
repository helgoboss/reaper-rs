use c_str_macro::c_str;
use std::borrow::Cow;

use std::ptr::NonNull;

use reaper_rs_low::{
    add_cpp_control_surface, raw, remove_cpp_control_surface, IReaperControlSurface,
};

use crate::infostruct_keeper::InfostructKeeper;

use crate::{
    concat_c_strs, delegating_hook_command, delegating_hook_post_command, delegating_toggle_action,
    require_non_null, require_non_null_panic, ActionValueChange, AddFxBehavior, AudioHookRegister,
    AudioThread, AutomationMode, Bpm, ChunkCacheHint, CommandId, CreateTrackSendFailed, Db,
    DelegatingControlSurface, EnvChunkName, FxAddByNameBehavior, FxPresetRef, FxShowInstruction,
    GangBehavior, GlobalAutomationModeOverride, Hwnd, InputMonitoringMode, KbdSectionInfo,
    MainThread, MasterTrackBehavior, MediaTrack, MediumAudioHookRegister, MediumGaccelRegister,
    MediumHookCommand, MediumHookPostCommand, MediumOnAudioBuffer, MediumReaperControlSurface,
    MediumToggleAction, MessageBoxResult, MessageBoxType, MidiInput, MidiInputDeviceId,
    MidiOutputDeviceId, NotRegistered, NotificationBehavior, PlaybackSpeedFactor,
    PluginRegistration, ProjectContext, ProjectPart, ProjectRef, ReaProject, ReaperFunctions,
    ReaperNormalizedFxParamValue, ReaperPanValue, ReaperPointer, ReaperStringArg, ReaperVersion,
    ReaperVolumeValue, RecordArmState, RecordingInput, RegistrationFailed, SectionContext,
    SectionId, SendTarget, StuffMidiMessageTarget, TrackDefaultsBehavior, TrackEnvelope,
    TrackFxChainType, TrackFxLocation, TrackInfoKey, TrackRef, TrackSendCategory,
    TrackSendDirection, TrackSendInfoKey, TransferBehavior, UndoBehavior, UndoScope, ValueChange,
    VolumeSliderValue, WindowContext,
};

use reaper_rs_low;
use reaper_rs_low::raw::audio_hook_register_t;
use std::collections::{HashMap, HashSet};

/// This is the main hub for accessing medium-level API functions.
///
/// In order to use this struct, you first must obtain an instance of it by invoking [`new()`].
/// This struct itself is limited to REAPER functions for registering/unregistering certain things.
/// You can access all the other functions by calling [`functions()`].
///
/// Please note that this struct will take care of unregistering everything (also audio hooks)
/// automatically when it gets dropped (good RAII manners).
///
/// # Design
///
/// ## Why is there a separation into `Reaper` and `ReaperFunctions`?
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
/// with `Arc<Mutex<Reaper>>`. However, why going through all that trouble and put up with possible
/// performance issues if we can avoid it?
///
/// [`new()`]: #method.new
/// [`functions()`]: #method.functions
#[derive(Debug)]
pub struct Reaper {
    functions: ReaperFunctions<dyn MainThread>,
    gaccel_registers: InfostructKeeper<MediumGaccelRegister, raw::gaccel_register_t>,
    audio_hook_registers: InfostructKeeper<MediumAudioHookRegister, raw::audio_hook_register_t>,
    csurf_insts: HashMap<NonNull<raw::IReaperControlSurface>, Box<Box<dyn IReaperControlSurface>>>,
    plugin_registrations: HashSet<PluginRegistration<'static>>,
    audio_hook_registrations: HashSet<NonNull<raw::audio_hook_register_t>>,
}

impl Reaper {
    /// Creates a new instance by getting hold of a [low-level `Reaper`] instance.
    ///
    /// [low-level `Reaper`]: /reaper_rs_low/struct.Reaper.html
    pub fn new(low: reaper_rs_low::Reaper) -> Reaper {
        Reaper {
            functions: ReaperFunctions::new(low),
            gaccel_registers: Default::default(),
            audio_hook_registers: Default::default(),
            csurf_insts: Default::default(),
            plugin_registrations: Default::default(),
            audio_hook_registrations: Default::default(),
        }
    }

    /// Gives access to all REAPER functions which can be safely executed in the main thread.
    pub fn functions(&self) -> &ReaperFunctions<dyn MainThread> {
        &self.functions
    }

    /// Creates a new container of REAPER functions with only those unlocked that can be safely
    /// executed in the audio thread.
    pub fn create_real_time_functions(&self) -> ReaperFunctions<dyn AudioThread> {
        ReaperFunctions::new(self.functions.low().clone())
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
        reg: PluginRegistration,
    ) -> Result<i32, RegistrationFailed> {
        self.plugin_registrations.insert(reg.clone().into_owned());
        let infostruct = reg.ptr_to_raw();
        let result = self
            .functions
            .low()
            .plugin_register(reg.key_into_raw().as_ptr(), infostruct);
        if result == 0 {
            return Err(RegistrationFailed);
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
    pub unsafe fn plugin_register_remove(&mut self, reg: PluginRegistration) -> i32 {
        let infostruct = reg.ptr_to_raw();
        let name_with_minus = concat_c_strs(c_str!("-"), reg.clone().key_into_raw().as_ref());
        let result = self
            .functions
            .low()
            .plugin_register(name_with_minus.as_ptr(), infostruct);
        self.plugin_registrations.remove(&reg.into_owned());
        result
    }

    /// Registers a hook command.
    ///
    /// REAPER calls hook commands whenever an action is requested to be run.
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    pub fn plugin_register_add_hookcommand<T: MediumHookCommand>(
        &mut self,
    ) -> Result<(), RegistrationFailed> {
        unsafe {
            self.plugin_register_add(PluginRegistration::HookCommand(
                delegating_hook_command::<T>,
            ))?;
        }
        Ok(())
    }

    /// Unregisters a hook command.
    pub fn plugin_register_remove_hookcommand<T: MediumHookCommand>(&mut self) {
        unsafe {
            self.plugin_register_remove(PluginRegistration::HookCommand(
                delegating_hook_command::<T>,
            ));
        }
    }

    /// Registers a toggle action.
    ///
    /// REAPER calls toggle actions whenever it wants to know the on/off state of an action.
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    pub fn plugin_register_add_toggleaction<T: MediumToggleAction>(
        &mut self,
    ) -> Result<(), RegistrationFailed> {
        unsafe {
            self.plugin_register_add(PluginRegistration::ToggleAction(
                delegating_toggle_action::<T>,
            ))?
        };
        Ok(())
    }

    /// Unregisters a toggle action.
    pub fn plugin_register_remove_toggleaction<T: MediumToggleAction>(&mut self) {
        unsafe {
            self.plugin_register_remove(PluginRegistration::ToggleAction(
                delegating_toggle_action::<T>,
            ));
        }
    }

    /// Registers a hook post command.
    ///
    /// REAPER calls hook post commands whenever an action of the main section has been performed.
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    pub fn plugin_register_add_hookpostcommand<T: MediumHookPostCommand>(
        &mut self,
    ) -> Result<(), RegistrationFailed> {
        unsafe {
            self.plugin_register_add(PluginRegistration::HookPostCommand(
                delegating_hook_post_command::<T>,
            ))?
        };
        Ok(())
    }

    /// Unregisters a hook post command.
    pub fn plugin_register_remove_hookpostcommand<T: MediumHookPostCommand>(&mut self) {
        unsafe {
            self.plugin_register_remove(PluginRegistration::HookPostCommand(
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
    ) -> Result<CommandId, RegistrationFailed> {
        let raw_id = unsafe {
            self.plugin_register_add(PluginRegistration::CommandId(
                command_name.into().into_inner(),
            ))? as u32
        };
        Ok(CommandId(raw_id))
    }

    /// Registers a an action into the main section.
    ///
    /// This consists of a command ID, a description and a default binding for it. It doesn't
    /// include the actual code to be executed when the action runs (use
    /// [`plugin_register_add_hookcommand()`] for that).
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
    /// [`plugin_register_add_hookcommand()`]: #method.plugin_register_add_hookcommand
    /// [`plugin_register_remove_gaccel()`]: #method.plugin_register_remove_gaccel
    pub fn plugin_register_add_gaccel(
        &mut self,
        reg: MediumGaccelRegister,
    ) -> Result<NonNull<raw::gaccel_register_t>, RegistrationFailed> {
        let handle = self.gaccel_registers.keep(reg);
        unsafe { self.plugin_register_add(PluginRegistration::Gaccel(handle))? };
        Ok(handle)
    }

    /// Unregisters an action.
    ///
    /// This function hands the once registered action back to you.
    ///
    /// # Errors
    ///
    /// Returns an error if the action was not registered.
    pub fn plugin_register_remove_gaccel(
        &mut self,
        reg_handle: NonNull<raw::gaccel_register_t>,
    ) -> Result<MediumGaccelRegister, NotRegistered> {
        unsafe { self.plugin_register_remove(PluginRegistration::Gaccel(reg_handle)) };
        let original = self
            .gaccel_registers
            .release(reg_handle)
            .ok_or(NotRegistered)?;
        Ok(original)
    }

    /// Registers a hidden control surface.
    ///
    /// This is very useful for being notified by REAPER about all kinds of events.
    ///
    /// This function returns a handle which you can use to unregister the control surface at any
    /// time via [`plugin_register_remove_csurf_inst()`].
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    ///
    /// [`plugin_register_remove_csurf_inst()`]: #method.plugin_register_remove_csurf_inst
    pub fn plugin_register_add_csurf_inst(
        &mut self,
        control_surface: impl MediumReaperControlSurface + 'static,
    ) -> Result<NonNull<raw::IReaperControlSurface>, RegistrationFailed> {
        let rust_control_surface =
            DelegatingControlSurface::new(control_surface, &self.functions.get_app_version());
        // We need to box it twice in order to obtain a thin pointer for passing to C as callback
        // target
        let rust_control_surface: Box<Box<dyn IReaperControlSurface>> =
            Box::new(Box::new(rust_control_surface));
        let cpp_control_surface =
            unsafe { add_cpp_control_surface(rust_control_surface.as_ref().into()) };
        self.csurf_insts
            .insert(cpp_control_surface, rust_control_surface);
        unsafe { self.plugin_register_add(PluginRegistration::CsurfInst(cpp_control_surface))? };
        Ok(cpp_control_surface)
    }

    /// Unregisters a hidden control surface.
    pub fn plugin_register_remove_csurf_inst(
        &mut self,
        handle: NonNull<raw::IReaperControlSurface>,
    ) {
        unsafe {
            self.plugin_register_remove(PluginRegistration::CsurfInst(handle));
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
        reg: NonNull<audio_hook_register_t>,
    ) -> Result<(), RegistrationFailed> {
        self.audio_hook_registrations.insert(reg);
        let result = self
            .functions
            .low()
            .Audio_RegHardwareHook(true, reg.as_ptr());
        if result == 0 {
            return Err(RegistrationFailed);
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
        reg: NonNull<audio_hook_register_t>,
    ) {
        self.functions
            .low()
            .Audio_RegHardwareHook(false, reg.as_ptr());
        self.audio_hook_registrations.remove(&reg);
    }

    /// Registers an audio hook register.
    ///
    /// This allows you to get called back in the audio thread before and after REAPER's processing.
    /// You should be careful with this because you are entering real-time world.
    ///
    /// This function returns a handle which you can use to unregister the audio hook register at
    /// any time via [`audio_reg_hardware_hook_remove()`] (from the main thread).
    ///
    /// # Errors
    ///
    /// Returns an error if the registration failed.
    ///
    /// [`audio_reg_hardware_hook_remove()`]: #method.audio_reg_hardware_hook_remove
    pub fn audio_reg_hardware_hook_add<T: MediumOnAudioBuffer + 'static>(
        &mut self,
        callback: T,
    ) -> Result<NonNull<audio_hook_register_t>, RegistrationFailed> {
        let handle = self
            .audio_hook_registers
            .keep(MediumAudioHookRegister::new(callback));
        unsafe { self.audio_reg_hardware_hook_add_unchecked(handle)? };
        Ok(handle)
    }

    /// Unregisters an audio hook register.
    pub fn audio_reg_hardware_hook_remove(&mut self, reg_handle: NonNull<audio_hook_register_t>) {
        unsafe { self.audio_reg_hardware_hook_remove_unchecked(reg_handle) };
        let _ = self.audio_hook_registers.release(reg_handle);
    }
}

impl Drop for Reaper {
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
