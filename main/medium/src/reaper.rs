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
    DelegatingControlSurface, EnvChunkName, FxAddByNameBehavior, FxPresetRef, FxShowFlag,
    GangBehavior, GlobalAutomationModeOverride, Hwnd, InputMonitoringMode, KbdSectionInfo,
    MainThread, MasterTrackBehavior, MediaTrack, MediumAudioHookRegister, MediumGaccelRegister,
    MediumHookCommand, MediumHookPostCommand, MediumOnAudioBuffer, MediumReaperControlSurface,
    MediumToggleAction, MessageBoxResult, MessageBoxType, MidiInput, MidiInputDeviceId,
    MidiOutputDeviceId, NotificationBehavior, PlaybackSpeedFactor, PluginRegistration,
    ProjectContext, ProjectPart, ProjectRef, ReaProject, ReaperFunctions,
    ReaperNormalizedFxParamValue, ReaperPanValue, ReaperPointer, ReaperStringArg, ReaperVersion,
    ReaperVolumeValue, RecordArmState, RecordingInput, SectionContext, SectionId, SendTarget,
    StuffMidiMessageTarget, TrackDefaultsBehavior, TrackEnvelope, TrackFxChainType,
    TrackFxLocation, TrackInfoKey, TrackRef, TrackSendCategory, TrackSendDirection,
    TrackSendInfoKey, TransferBehavior, UndoBehavior, UndoScope, ValueChange, VolumeSliderValue,
    WindowContext,
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

    // Kept return value type i32 because meaning of return value depends very much on the actual
    // thing which is registered and probably is not safe to generalize.
    // Unregistering is optional! It will be done anyway on Drop via RAII.
    pub unsafe fn plugin_register_add(&mut self, reg: PluginRegistration) -> i32 {
        self.plugin_registrations.insert(reg.clone().into_owned());
        let infostruct = reg.ptr_to_raw();
        let result = self
            .functions
            .low()
            .plugin_register(reg.key_into_raw().as_ptr(), infostruct);
        result
    }

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

    pub fn plugin_register_add_hookcommand<T: MediumHookCommand>(&mut self) -> Result<(), ()> {
        let result = unsafe {
            self.plugin_register_add(PluginRegistration::HookCommand(
                delegating_hook_command::<T>,
            ))
        };
        ok_if_one(result)
    }

    pub fn plugin_register_remove_hookcommand<T: MediumHookCommand>(&mut self) {
        unsafe {
            self.plugin_register_remove(PluginRegistration::HookCommand(
                delegating_hook_command::<T>,
            ));
        }
    }

    pub fn plugin_register_add_toggleaction<T: MediumToggleAction>(&mut self) -> Result<(), ()> {
        let result = unsafe {
            self.plugin_register_add(PluginRegistration::ToggleAction(
                delegating_toggle_action::<T>,
            ))
        };
        ok_if_one(result)
    }

    pub fn plugin_register_remove_toggleaction<T: MediumToggleAction>(&mut self) {
        unsafe {
            self.plugin_register_remove(PluginRegistration::ToggleAction(
                delegating_toggle_action::<T>,
            ));
        }
    }

    pub fn plugin_register_add_hookpostcommand<T: MediumHookPostCommand>(
        &mut self,
    ) -> Result<(), ()> {
        let result = unsafe {
            self.plugin_register_add(PluginRegistration::HookPostCommand(
                delegating_hook_post_command::<T>,
            ))
        };
        ok_if_one(result)
    }

    pub fn plugin_register_remove_hookpostcommand<T: MediumHookPostCommand>(&mut self) {
        unsafe {
            self.plugin_register_remove(PluginRegistration::HookPostCommand(
                delegating_hook_post_command::<T>,
            ));
        }
    }

    // Returns the assigned command index.
    // If the command ID is already used, it just returns the index which has been assigned before.
    // Passing an empty string actually works (!). If a null pointer is passed, 0 is returned, but
    // we can't do that using this signature. If a very large string is passed, it works. If a
    // number of a built-in command is passed, it works.
    //
    ///  which is unique to the current REAPER
    //     /// session.
    pub fn plugin_register_add_command_id<'a>(
        &mut self,
        command_name: impl Into<ReaperStringArg<'a>>,
    ) -> CommandId {
        let raw_id = unsafe {
            self.plugin_register_add(PluginRegistration::CommandId(
                command_name.into().into_inner(),
            )) as u32
        };
        CommandId(raw_id)
    }

    // # Old description (not valid anymore, problem solved)
    //
    // A reference is in line here (vs. pointer) because gaccel_register_t is a struct created on
    // our (Rust) side. It doesn't necessary have to be static because we might just write a
    // script which registers something only shortly and unregisters it again later.
    //
    // gaccel_register_t and similar structs registered with plugin_register cannot be,
    // lifted to medium-level API style. Because at the end of the day
    // REAPER *needs* the correct struct here. Also, with structs we can't do any indirection as
    // with function calls. So at a maxium we can provide some optionally usable
    // factory method for creating such structs. The consumer must ensure that it lives long
    // enough!
    //
    // Unsafe because consumer must ensure proper lifetime of given reference.
    //
    // # New description
    //
    // Medium-level API takes care now of keeping the registered infostructs. The API consumer
    // doesn't need to take care of maintaining a stable address. It's also more safe because
    // the API consumer needs to give up ownership of the thing given and read or even mutated by
    // REAPER. This is why we can make this function save! No lifetime worries anymore.
    pub fn plugin_register_add_gaccel(
        &mut self,
        reg: MediumGaccelRegister,
    ) -> Result<NonNull<raw::gaccel_register_t>, ()> {
        let handle = self.gaccel_registers.keep(reg);
        let result = unsafe { self.plugin_register_add(PluginRegistration::Gaccel(handle)) };
        if result != 1 {
            return Err(());
        }
        Ok(handle)
    }

    pub fn plugin_register_remove_gaccel(
        &mut self,
        reg_handle: NonNull<raw::gaccel_register_t>,
    ) -> Result<MediumGaccelRegister, ()> {
        unsafe { self.plugin_register_remove(PluginRegistration::Gaccel(reg_handle)) };
        let original = self.gaccel_registers.release(reg_handle).ok_or(())?;
        Ok(original)
    }

    pub fn plugin_register_add_csurf_inst(
        &mut self,
        control_surface: impl MediumReaperControlSurface + 'static,
    ) -> Result<NonNull<raw::IReaperControlSurface>, ()> {
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
        let result =
            unsafe { self.plugin_register_add(PluginRegistration::CsurfInst(cpp_control_surface)) };
        if result != 1 {
            return Err(());
        }
        Ok(cpp_control_surface)
    }

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
    // The given audio_hook_register_t will be modified by REAPER. After registering it, it must
    // only be accessed from within OnAudioBuffer callback (passed as param).
    // Returns true on success
    pub unsafe fn audio_reg_hardware_hook_add_unchecked(
        &mut self,
        reg: NonNull<audio_hook_register_t>,
    ) -> Result<(), ()> {
        self.audio_hook_registrations.insert(reg);
        let result = self
            .functions
            .low()
            .Audio_RegHardwareHook(true, reg.as_ptr());
        ok_if_one(result)
    }

    pub unsafe fn audio_reg_hardware_hook_remove_unchecked(
        &mut self,
        reg: NonNull<audio_hook_register_t>,
    ) {
        self.functions
            .low()
            .Audio_RegHardwareHook(false, reg.as_ptr());
        self.audio_hook_registrations.remove(&reg);
    }

    pub fn audio_reg_hardware_hook_add<T: MediumOnAudioBuffer + 'static>(
        &mut self,
        callback: T,
    ) -> Result<NonNull<audio_hook_register_t>, ()> {
        let handle = self
            .audio_hook_registers
            .keep(MediumAudioHookRegister::new(callback));
        unsafe { self.audio_reg_hardware_hook_add_unchecked(handle)? };
        Ok(handle)
    }

    pub fn audio_reg_hardware_hook_remove(
        &mut self,
        reg_handle: NonNull<audio_hook_register_t>,
    ) -> Result<(), ()> {
        self.audio_hook_registers.release(reg_handle).ok_or(())?;
        unsafe { self.audio_reg_hardware_hook_remove_unchecked(reg_handle) };
        Ok(())
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

fn ok_if_one(result: i32) -> Result<(), ()> {
    if result == 1 { Ok(()) } else { Err(()) }
}
