use c_str_macro::c_str;
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr::{null_mut, NonNull};

use reaper_rs_low::{
    add_cpp_control_surface, firewall, raw, remove_cpp_control_surface, IReaperControlSurface,
};

use crate::infostruct_keeper::InfostructKeeper;
use crate::ProjectContext::CurrentProject;
use crate::{
    concat_c_strs, delegating_hook_command, delegating_hook_post_command, delegating_toggle_action,
    require_non_null, require_non_null_panic, ActionValueChange, AddFxBehavior, AudioHookRegister,
    AutomationMode, Bpm, ChunkCacheHint, CommandId, CreateTrackSendFailed, Db,
    DelegatingControlSurface, EnvChunkName, FxAddByNameBehavior, FxPresetRef, FxShowFlag,
    GaccelRegister, GangBehavior, GlobalAutomationOverride, Hwnd, InputMonitoringMode,
    KbdSectionInfo, MasterTrackBehavior, MediaTrack, MediumAudioHookRegister, MediumGaccelRegister,
    MediumHookCommand, MediumHookPostCommand, MediumReaperControlSurface, MediumToggleAction,
    MessageBoxResult, MessageBoxType, MidiInput, MidiInputDeviceId, MidiOutput, MidiOutputDeviceId,
    NotificationBehavior, PlaybackSpeedFactor, PluginRegistration, ProjectContext, ProjectRef,
    ReaProject, RealtimeReaper, ReaperControlSurface, ReaperNormalizedValue, ReaperPanValue,
    ReaperPointer, ReaperStringArg, ReaperVersion, ReaperVolumeValue, RecordArmState,
    RecordingInput, SectionContext, SectionId, SendTarget, StuffMidiMessageTarget,
    TrackDefaultsBehavior, TrackEnvelope, TrackFxChainType, TrackFxRef, TrackInfoKey, TrackRef,
    TrackSendCategory, TrackSendDirection, TrackSendInfoKey, TransferBehavior, UndoBehavior,
    UndoFlag, UndoScope, ValueChange, VolumeSliderValue, WindowContext,
};
use enumflags2::BitFlags;
use helgoboss_midi::ShortMessage;
use reaper_rs_low;
use reaper_rs_low::raw::{
    audio_hook_register_t, gaccel_register_t, midi_Input, GUID, UNDO_STATE_ALL,
};
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::mem::MaybeUninit;
use std::path::PathBuf;
use std::rc::Rc;

/// This is the medium-level API access point to all REAPER functions. In order to use it, you first
/// must obtain an instance of this struct by invoking [`new`](struct.Reaper.html#method.new).
///
/// It's always possible that a function from the low-level API is missing in the medium-level one.
/// That's because unlike the low-level API, the medium-level API is hand-written and a perpetual
/// work in progress. If you can't find the function that you need, you can always resort to the
/// low-level API by navigating to [`low`](struct.Reaper.html#structfield.low). Of course you are
/// welcome to contribute to bring the medium-level API on par with the low-level one.  
pub struct Reaper {
    // It's an Rc because we need `RealtimeReaper` to be completely stand-alone (without any
    // borrowed stuff) to be able to box it to be able to obtain a stable address for handing it
    // over to Reaper as userdata for audio_hook_register_t. If we wouldn't share low-level Reaper,
    // we would either need to copy it (unnecessary memory overhead, although small, ~800 * 8 byte
    // = ~7 kB) or let the consumer wrap medium-level Reaper in an Rc (which would be too
    // presumptuous and also lead to unnecessary indirection).
    low: Rc<reaper_rs_low::Reaper>,
    gaccel_registers: InfostructKeeper<MediumGaccelRegister, raw::gaccel_register_t>,
    audio_hook_registers: InfostructKeeper<MediumAudioHookRegister, raw::audio_hook_register_t>,
    csurf_insts: HashMap<NonNull<raw::IReaperControlSurface>, Box<Box<dyn IReaperControlSurface>>>,
    plugin_registrations: HashSet<PluginRegistration<'static>>,
    audio_hook_registrations: HashSet<NonNull<raw::audio_hook_register_t>>,
}

const ZERO_GUID: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

fn with_string_buffer<T>(
    max_size: u32,
    fill_buffer: impl FnOnce(*mut c_char, i32) -> T,
) -> (CString, T) {
    let vec: Vec<u8> = vec![1; max_size as usize];
    let c_string = unsafe { CString::from_vec_unchecked(vec) };
    let raw = c_string.into_raw();
    let result = fill_buffer(raw, max_size as i32);
    let string = unsafe { CString::from_raw(raw) };
    (string, result)
}

impl Reaper {
    /// Creates a new instance by getting hold of a
    /// [`reaper_rs_low::Reaper`](../../low_level/struct.Reaper.html) instance.
    pub fn new(low: reaper_rs_low::Reaper) -> Reaper {
        Reaper {
            low: Rc::new(low),
            gaccel_registers: Default::default(),
            audio_hook_registers: Default::default(),
            csurf_insts: Default::default(),
            plugin_registrations: Default::default(),
            audio_hook_registrations: Default::default(),
        }
    }

    pub fn low(&self) -> &reaper_rs_low::Reaper {
        &self.low
    }

    pub fn create_realtime_reaper(&self) -> RealtimeReaper {
        RealtimeReaper::new(self.low.clone())
    }

    /// Returns the requested project and optionally its file name.
    ///
    /// With `projfn_out_optional_sz` you can tell REAPER how many characters of the file name you
    /// want. If you are not interested in the file name at all, pass 0.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let reaper = reaper_rs_medium::Reaper::new(reaper_rs_low::Reaper::default());
    /// use reaper_rs_medium::ProjectRef::Tab;
    ///
    /// let result = reaper.enum_projects(Tab(4), 256).ok_or("No such tab")?;
    /// let project_dir = result.file_path.ok_or("Project not saved yet")?.parent();
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn enum_projects(
        &self,
        proj_ref: ProjectRef,
        projfn_out_optional_sz: u32,
    ) -> Option<EnumProjectsResult> {
        use ProjectRef::*;
        let idx = proj_ref.into();
        if projfn_out_optional_sz == 0 {
            let ptr = unsafe { self.low.EnumProjects(idx, null_mut(), 0) };
            let project = NonNull::new(ptr)?;
            Some(EnumProjectsResult {
                project,
                file_path: None,
            })
        } else {
            let (owned_c_string, ptr) =
                with_string_buffer(projfn_out_optional_sz, |buffer, max_size| unsafe {
                    self.low.EnumProjects(idx, buffer, max_size)
                });
            let project = NonNull::new(ptr)?;
            if owned_c_string.to_bytes().len() == 0 {
                return Some(EnumProjectsResult {
                    project,
                    file_path: None,
                });
            }
            let owned_string = owned_c_string
                .into_string()
                .expect("Path contains non-UTF8 characters");
            Some(EnumProjectsResult {
                project,
                file_path: Some(PathBuf::from(owned_string)),
            })
        }
    }

    /// Returns the track at the given index. Set `proj` to `null_mut()` in order to look for tracks
    /// in the current project.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let reaper = reaper_rs_medium::Reaper::new(reaper_rs_low::Reaper::default());
    /// use reaper_rs_medium::ProjectContext::CurrentProject;
    ///
    /// let track = reaper.get_track(CurrentProject, 3).ok_or("No such track")?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_track(&self, proj: ProjectContext, trackidx: u32) -> Option<MediaTrack> {
        self.require_valid_project(proj);
        unsafe { self.get_track_unchecked(proj, trackidx) }
    }

    pub unsafe fn get_track_unchecked(
        &self,
        proj: ProjectContext,
        trackidx: u32,
    ) -> Option<MediaTrack> {
        let ptr = self.low.GetTrack(proj.into(), trackidx as i32);
        NonNull::new(ptr)
    }

    /// Returns `true` if the given pointer is a valid object of the right type in project `proj`
    /// (`proj` is ignored if pointer is itself a project).
    pub fn validate_ptr_2<'a>(
        &self,
        proj: ProjectContext,
        pointer: impl Into<ReaperPointer<'a>>,
    ) -> bool {
        let pointer = pointer.into();
        unsafe {
            self.low
                .ValidatePtr2(proj.into(), pointer.as_void(), Cow::from(pointer).as_ptr())
        }
    }

    pub fn validate_ptr<'a>(&self, pointer: impl Into<ReaperPointer<'a>>) -> bool {
        let pointer = pointer.into();
        unsafe {
            self.low
                .ValidatePtr(pointer.as_void(), Cow::from(pointer).as_ptr())
        }
    }

    /// Shows a message to the user (also useful for debugging). Send "\n" for newline and "" to
    /// clear the console.
    pub fn show_console_msg<'a>(&self, msg: impl Into<ReaperStringArg<'a>>) {
        unsafe { self.low.ShowConsoleMsg(msg.into().as_ptr()) }
    }

    /// Gets or sets track arbitrary attributes. This just delegates to the low-level analog. Using
    /// this function is not fun and requires you to use unsafe code. Consider using one of the
    /// type-safe convenience functions instead. They start with `get_media_track_info_` or
    /// `set_media_track_info_`.
    pub unsafe fn get_set_media_track_info(
        &self,
        tr: MediaTrack,
        parmname: TrackInfoKey,
        set_new_value: *mut c_void,
    ) -> *mut c_void {
        self.low
            .GetSetMediaTrackInfo(tr.as_ptr(), Cow::from(parmname).as_ptr(), set_new_value)
    }

    /// Convenience function which returns the given track's parent track (`P_PARTRACK`).
    pub unsafe fn get_media_track_info_partrack(&self, tr: MediaTrack) -> Option<MediaTrack> {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::ParTrack, null_mut())
            as *mut raw::MediaTrack;
        NonNull::new(ptr)
    }

    pub fn is_in_real_time_audio(&self) -> bool {
        self.low.IsInRealTimeAudio() != 0
    }

    /// Convenience function which returns the given track's parent project (`P_PROJECT`).
    // In REAPER < 5.95 this returns nullptr
    pub unsafe fn get_media_track_info_project(&self, tr: MediaTrack) -> Option<ReaProject> {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::Project, null_mut())
            as *mut raw::ReaProject;
        NonNull::new(ptr)
    }

    /// Convenience function which let's you use the given track's name (`P_NAME`).
    pub unsafe fn get_media_track_info_name<R>(
        &self,
        tr: MediaTrack,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R> {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::Name, null_mut());
        unsafe { create_passing_c_str(ptr as *const c_char) }.map(f)
    }

    /// Convenience function which returns the given track's input monitoring mode (I_RECMON).
    pub unsafe fn get_media_track_info_recmon(&self, tr: MediaTrack) -> InputMonitoringMode {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::RecMon, null_mut());
        let irecmon = unsafe { unref_as::<i32>(ptr) }.unwrap();
        InputMonitoringMode::try_from(irecmon).expect("Unknown input monitoring mode")
    }

    /// Convenience function which returns the given track's recording input (I_RECINPUT).
    pub unsafe fn get_media_track_info_recinput(&self, tr: MediaTrack) -> Option<RecordingInput> {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::RecInput, null_mut());
        let rec_input_index = unsafe { unref_as::<i32>(ptr) }.unwrap();
        if rec_input_index < 0 {
            None
        } else {
            Some(RecordingInput::try_from(rec_input_index as u32).unwrap())
        }
    }

    /// Convenience function which returns the given track's number (IP_TRACKNUMBER).
    pub unsafe fn get_media_track_info_tracknumber(&self, tr: MediaTrack) -> Option<TrackRef> {
        use TrackRef::*;
        match self.get_set_media_track_info(tr, TrackInfoKey::TrackNumber, null_mut()) as i32 {
            -1 => Some(MasterTrack),
            0 => None,
            n if n > 0 => Some(NormalTrack(n as u32 - 1)),
            _ => unreachable!(),
        }
    }

    /// Convenience function which returns the given track's GUID (GUID).
    pub unsafe fn get_media_track_info_guid(&self, tr: MediaTrack) -> GUID {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::Guid, null_mut());
        unsafe { unref_as::<GUID>(ptr) }.unwrap()
    }

    // Kept return value type i32 because meaning of return value depends very much on the actual
    // thing which is registered and probably is not safe to generalize.
    // Unregistering is optional! It will be done anyway on Drop via RAII.
    pub unsafe fn plugin_register_add(&mut self, reg: PluginRegistration) -> i32 {
        self.plugin_registrations.insert(reg.clone().to_owned());
        let infostruct = reg.infostruct();
        let result = self
            .low
            .plugin_register(Cow::from(reg).as_ptr(), infostruct);
        result
    }

    pub unsafe fn plugin_register_remove(&mut self, reg: PluginRegistration) -> i32 {
        let infostruct = reg.infostruct();
        let name_with_minus = concat_c_strs(c_str!("-"), Cow::from(reg.clone()).as_ref());
        let result = self
            .low
            .plugin_register(name_with_minus.as_ptr(), infostruct);
        self.plugin_registrations.remove(&reg.to_owned());
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
            DelegatingControlSurface::new(control_surface, &self.get_app_version());
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

    /// Performs an action belonging to the main action section. To perform non-native actions
    /// (ReaScripts, custom or extension plugins' actions) safely, see
    /// [`named_command_lookup`](struct.Reaper.html#method.named_command_lookup).
    pub fn main_on_command_ex(&self, command: CommandId, flag: i32, proj: ProjectContext) {
        self.require_valid_project(proj);
        unsafe { self.main_on_command_ex_unchecked(command, flag, proj) }
    }

    pub unsafe fn main_on_command_ex_unchecked(
        &self,
        command: CommandId,
        flag: i32,
        proj: ProjectContext,
    ) {
        self.low.Main_OnCommandEx(command.into(), flag, proj.into());
    }

    /// # Example
    ///
    /// ```no_run
    /// # let reaper = reaper_rs_medium::Reaper::new(reaper_rs_low::Reaper::default());
    /// use reaper_rs_medium::{NotificationBehavior::NotifyAllExcept, ProjectContext::CurrentProject};
    /// use reaper_rs_medium::get_cpp_control_surface;
    ///
    /// let track = reaper.get_track(CurrentProject, 0).ok_or("Track doesn't exist")?;
    /// unsafe {
    ///     reaper.csurf_set_surface_mute(track, true, NotifyAllExcept(get_cpp_control_surface()));
    /// }
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub unsafe fn csurf_set_surface_mute(
        &self,
        trackid: MediaTrack,
        mute: bool,
        ignoresurf: NotificationBehavior,
    ) {
        self.low
            .CSurf_SetSurfaceMute(trackid.as_ptr(), mute, ignoresurf.into());
    }

    pub unsafe fn csurf_set_surface_solo(
        &self,
        trackid: MediaTrack,
        solo: bool,
        ignoresurf: NotificationBehavior,
    ) {
        self.low
            .CSurf_SetSurfaceSolo(trackid.as_ptr(), solo, ignoresurf.into());
    }

    /// Generates a random GUID.
    pub fn gen_guid(&self) -> GUID {
        let mut guid = MaybeUninit::uninit();
        unsafe {
            self.low.genGuid(guid.as_mut_ptr());
        }
        unsafe { guid.assume_init() }
    }

    // In order to not need unsafe, we take the closure. For normal medium-level API usage, this is
    // the safe way to go.
    pub fn section_from_unique_id<R>(
        &self,
        unique_id: SectionId,
        f: impl FnOnce(&KbdSectionInfo) -> R,
    ) -> Option<R> {
        let ptr = self.low.SectionFromUniqueID(unique_id.into());
        if ptr.is_null() {
            return None;
        }
        NonNull::new(ptr).map(|nnp| f(&KbdSectionInfo(nnp)))
    }

    // The closure-taking function might be too restrictive in some cases, e.g. it wouldn't let us
    // return an iterator (which is of course lazily evaluated). Also, in some cases we might know
    // that a section is always valid, e.g. if it's the main section. A higher-level API could
    // use this for such edge cases. If not the main section, a higher-level API
    // should check if the section still exists (via section index) before each usage.
    pub unsafe fn section_from_unique_id_unchecked(
        &self,
        unique_id: SectionId,
    ) -> Option<KbdSectionInfo> {
        let ptr = self.low.SectionFromUniqueID(unique_id.into());
        NonNull::new(ptr).map(KbdSectionInfo)
    }

    // Kept return value type i32 because I have no idea what the return value is about.
    pub unsafe fn kbd_on_main_action_ex(
        &self,
        cmd: CommandId,
        value: ActionValueChange,
        hwnd: WindowContext,
        proj: ProjectContext,
    ) -> i32 {
        use ActionValueChange::*;
        let (val, valhw, relmode) = match value {
            AbsoluteLowRes(v) => (i32::from(v), -1, 0),
            AbsoluteHighRes(v) => (
                ((u32::from(v) >> 7) & 0x7f) as i32,
                (u32::from(v) & 0x7f) as i32,
                0,
            ),
            Relative1(v) => (i32::from(v), -1, 1),
            Relative2(v) => (i32::from(v), -1, 2),
            Relative3(v) => (i32::from(v), -1, 3),
        };
        self.low
            .KBD_OnMainActionEx(cmd.into(), val, valhw, relmode, hwnd.into(), proj.into())
    }

    /// Returns the REAPER main window handle.
    pub fn get_main_hwnd(&self) -> Hwnd {
        require_non_null_panic(self.low.GetMainHwnd())
    }

    pub fn named_command_lookup<'a>(
        &self,
        command_name: impl Into<ReaperStringArg<'a>>,
    ) -> Option<CommandId> {
        let raw_id = unsafe { self.low.NamedCommandLookup(command_name.into().as_ptr()) as u32 };
        if raw_id == 0 {
            return None;
        }
        Some(CommandId(raw_id))
    }

    /// Clears the ReaScript console.
    pub fn clear_console(&self) {
        self.low.ClearConsole();
    }

    /// Returns the number of tracks in the given project (pass `null_mut()` for current project)
    pub fn count_tracks(&self, proj: ProjectContext) -> u32 {
        self.require_valid_project(proj);
        unsafe { self.count_tracks_unchecked(proj) }
    }

    pub unsafe fn count_tracks_unchecked(&self, proj: ProjectContext) -> u32 {
        self.low.CountTracks(proj.into()) as u32
    }

    pub fn insert_track_at_index(&self, idx: u32, want_defaults: TrackDefaultsBehavior) {
        self.low.InsertTrackAtIndex(
            idx as i32,
            want_defaults == TrackDefaultsBehavior::AddDefaultEnvAndFx,
        );
    }

    pub fn get_max_midi_inputs(&self) -> u32 {
        self.low.GetMaxMidiInputs() as u32
    }

    pub fn get_max_midi_outputs(&self) -> u32 {
        self.low.GetMaxMidiOutputs() as u32
    }

    pub fn get_midi_input_name(
        &self,
        dev: MidiInputDeviceId,
        nameout_sz: u32,
    ) -> GetMidiDevNameResult {
        if nameout_sz == 0 {
            let is_present = unsafe { self.low.GetMIDIInputName(dev.into(), null_mut(), 0) };
            GetMidiDevNameResult {
                is_present,
                name: None,
            }
        } else {
            let (name, is_present) = with_string_buffer(nameout_sz, |buffer, max_size| unsafe {
                self.low.GetMIDIInputName(dev.into(), buffer, max_size)
            });
            if name.to_bytes().len() == 0 {
                return GetMidiDevNameResult {
                    is_present,
                    name: None,
                };
            }
            GetMidiDevNameResult {
                is_present,
                name: Some(name),
            }
        }
    }

    // Return type Option or Result can't be easily chosen here because if instantiate is 0, it
    // should be Option, if it's -1 or > 0, it should be Result. So we just keep the i32.
    pub unsafe fn track_fx_add_by_name<'a>(
        &self,
        track: MediaTrack,
        fxname: impl Into<ReaperStringArg<'a>>,
        rec_fx: TrackFxChainType,
        instantiate: FxAddByNameBehavior,
    ) -> i32 {
        self.low.TrackFX_AddByName(
            track.as_ptr(),
            fxname.into().as_ptr(),
            rec_fx == TrackFxChainType::InputFxChain,
            instantiate.into(),
        )
    }

    pub unsafe fn track_fx_add_by_name_query<'a>(
        &self,
        track: MediaTrack,
        fxname: impl Into<ReaperStringArg<'a>>,
        rec_fx: TrackFxChainType,
    ) -> Option<u32> {
        match self.track_fx_add_by_name(track, fxname, rec_fx, FxAddByNameBehavior::Query) {
            -1 => None,
            idx if idx >= 0 => Some(idx as u32),
            _ => unreachable!(),
        }
    }

    pub unsafe fn track_fx_add_by_name_add<'a>(
        &self,
        track: MediaTrack,
        fxname: impl Into<ReaperStringArg<'a>>,
        rec_fx: TrackFxChainType,
        force_add: AddFxBehavior,
    ) -> Result<u32, ()> {
        match self.track_fx_add_by_name(track, fxname, rec_fx, force_add.into()) {
            -1 => Err(()),
            idx if idx >= 0 => Ok(idx as u32),
            _ => unreachable!(),
        }
    }

    pub fn get_midi_output_name(
        &self,
        dev: MidiOutputDeviceId,
        nameout_sz: u32,
    ) -> GetMidiDevNameResult {
        if nameout_sz == 0 {
            let is_present = unsafe { self.low.GetMIDIOutputName(dev.into(), null_mut(), 0) };
            GetMidiDevNameResult {
                is_present,
                name: None,
            }
        } else {
            let (name, is_present) = with_string_buffer(nameout_sz, |buffer, max_size| unsafe {
                self.low.GetMIDIOutputName(dev.into(), buffer, max_size)
            });
            if name.to_bytes().len() == 0 {
                return GetMidiDevNameResult {
                    is_present,
                    name: None,
                };
            }
            GetMidiDevNameResult {
                is_present,
                name: Some(name),
            }
        }
    }

    pub unsafe fn track_fx_get_enabled(&self, track: MediaTrack, fx: TrackFxRef) -> bool {
        self.low.TrackFX_GetEnabled(track.as_ptr(), fx.into())
    }

    // Returns Err if FX doesn't exist
    pub unsafe fn track_fx_get_fx_name(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
        buf_sz: u32,
    ) -> Result<CString, ()> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            self.low
                .TrackFX_GetFXName(track.as_ptr(), fx.into(), buffer, max_size)
        });
        if !successful {
            return Err(());
        }
        Ok(name)
    }

    pub unsafe fn track_fx_get_instrument(&self, track: MediaTrack) -> Option<u32> {
        let index = self.low.TrackFX_GetInstrument(track.as_ptr());
        if index == -1 {
            return None;
        }
        Some(index as u32)
    }

    pub unsafe fn track_fx_set_enabled(&self, track: MediaTrack, fx: TrackFxRef, enabled: bool) {
        self.low
            .TrackFX_SetEnabled(track.as_ptr(), fx.into(), enabled);
    }

    pub unsafe fn track_fx_get_num_params(&self, track: MediaTrack, fx: TrackFxRef) -> u32 {
        self.low.TrackFX_GetNumParams(track.as_ptr(), fx.into()) as u32
    }

    pub fn get_current_project_in_load_save(&self) -> Option<ReaProject> {
        let ptr = self.low.GetCurrentProjectInLoadSave();
        NonNull::new(ptr)
    }

    // Returns Err if FX or parameter doesn't exist
    pub unsafe fn track_fx_get_param_name(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
        param: u32,
        buf_sz: u32,
    ) -> Result<CString, ()> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            self.low
                .TrackFX_GetParamName(track.as_ptr(), fx.into(), param as i32, buffer, max_size)
        });
        if !successful {
            return Err(());
        }
        Ok(name)
    }

    // Returns Err if FX or parameter doesn't exist
    pub unsafe fn track_fx_get_formatted_param_value(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
        param: u32,
        buf_sz: u32,
    ) -> Result<CString, ()> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            self.low.TrackFX_GetFormattedParamValue(
                track.as_ptr(),
                fx.into(),
                param as i32,
                buffer,
                max_size,
            )
        });
        if !successful {
            return Err(());
        }
        Ok(name)
    }

    // Returns Err if FX or parameter doesn't exist or if FX doesn't support formatting arbitrary
    // parameter values and the given value is not equal to the current one.
    pub unsafe fn track_fx_format_param_value_normalized(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
        param: u32,
        value: ReaperNormalizedValue,
        buf_sz: u32,
    ) -> Result<CString, ()> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            self.low.TrackFX_FormatParamValueNormalized(
                track.as_ptr(),
                fx.into(),
                param as i32,
                value.get(),
                buffer,
                max_size,
            )
        });
        if !successful {
            return Err(());
        }
        Ok(name)
    }

    // Returns Err if FX or parameter doesn't exist
    pub unsafe fn track_fx_set_param_normalized(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
        param: u32,
        value: ReaperNormalizedValue,
    ) -> Result<(), ()> {
        let successful = self.low.TrackFX_SetParamNormalized(
            track.as_ptr(),
            fx.into(),
            param as i32,
            value.get(),
        );
        if !successful {
            return Err(());
        }
        Ok(())
    }

    pub fn get_focused_fx(&self) -> Option<GetFocusedFxResult> {
        let mut tracknumber = MaybeUninit::uninit();
        let mut itemnumber = MaybeUninit::uninit();
        let mut fxnumber = MaybeUninit::uninit();
        let result = unsafe {
            self.low.GetFocusedFX(
                tracknumber.as_mut_ptr(),
                itemnumber.as_mut_ptr(),
                fxnumber.as_mut_ptr(),
            )
        };
        let tracknumber = unsafe { tracknumber.assume_init() as u32 };
        let fxnumber = unsafe { fxnumber.assume_init() as u32 };
        use GetFocusedFxResult::*;
        match result {
            0 => None,
            1 => Some(TrackFx {
                track_ref: convert_tracknumber_to_track_ref(tracknumber),
                fx_ref: fxnumber.into(),
            }),
            2 => {
                // TODO-low Add test
                Some(ItemFx {
                    // Master track can't contain items
                    track_index: tracknumber - 1,
                    // Although the parameter is called itemnumber, it's zero-based (mentioned in
                    // official doc and checked)
                    item_index: unsafe { itemnumber.assume_init() as u32 },
                    take_index: (fxnumber >> 16) & 0xFFFF,
                    fx_index: fxnumber & 0xFFFF,
                })
            }
            _ => panic!("Unknown GetFocusedFX result value"),
        }
    }

    // Returns None if no FX has been touched yet or if the last-touched FX doesn't exist anymore
    pub fn get_last_touched_fx(&self) -> Option<GetLastTouchedFxResult> {
        let mut tracknumber = MaybeUninit::uninit();
        let mut fxnumber = MaybeUninit::uninit();
        let mut paramnumber = MaybeUninit::uninit();
        let is_valid = unsafe {
            self.low.GetLastTouchedFX(
                tracknumber.as_mut_ptr(),
                fxnumber.as_mut_ptr(),
                paramnumber.as_mut_ptr(),
            )
        };
        if !is_valid {
            return None;
        }
        let tracknumber = unsafe { tracknumber.assume_init() as u32 };
        let tracknumber_high_word = (tracknumber >> 16) & 0xFFFF;
        let fxnumber = unsafe { fxnumber.assume_init() as u32 };
        let paramnumber = unsafe { paramnumber.assume_init() as u32 };
        use GetLastTouchedFxResult::*;
        if tracknumber_high_word == 0 {
            Some(TrackFx {
                track_ref: convert_tracknumber_to_track_ref(tracknumber),
                fx_ref: fxnumber.into(),
                // Although the parameter is called paramnumber, it's zero-based (checked)
                param_index: paramnumber,
            })
        } else {
            // TODO-low Add test
            Some(ItemFx {
                // Master track can't contain items
                track_index: (tracknumber & 0xFFFF) - 1,
                item_index: tracknumber_high_word - 1,
                take_index: (fxnumber >> 16) & 0xFFFF,
                fx_index: fxnumber & 0xFFFF,
                // Although the parameter is called paramnumber, it's zero-based (checked)
                param_index: paramnumber,
            })
        }
    }

    pub unsafe fn track_fx_copy_to_track(
        &self,
        src: (MediaTrack, TrackFxRef),
        dest: (MediaTrack, TrackFxRef),
        is_move: TransferBehavior,
    ) {
        self.low.TrackFX_CopyToTrack(
            src.0.as_ptr(),
            src.1.into(),
            dest.0.as_ptr(),
            dest.1.into(),
            is_move == TransferBehavior::Move,
        );
    }

    // Returns Err if FX doesn't exist (maybe also in other cases?)
    pub unsafe fn track_fx_delete(&self, track: MediaTrack, fx: TrackFxRef) -> Result<(), ()> {
        let succesful = self.low.TrackFX_Delete(track.as_ptr(), fx.into());
        if !succesful {
            return Err(());
        }
        Ok(())
    }

    // Returns None if the FX parameter doesn't report step sizes (or if the FX or parameter doesn't
    // exist, but that can be checked before). Option makes more sense than Result here because
    // this function is at the same time the correct function to be used to determine *if* a
    // parameter reports step sizes. So "parameter doesn't report step sizes" is a valid result.
    pub unsafe fn track_fx_get_parameter_step_sizes(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
        param: u32,
    ) -> Option<GetParameterStepSizesResult> {
        let mut step = MaybeUninit::uninit();
        let mut small_step = MaybeUninit::uninit();
        let mut large_step = MaybeUninit::uninit();
        let mut is_toggle = MaybeUninit::uninit();
        let successful = self.low.TrackFX_GetParameterStepSizes(
            track.as_ptr(),
            fx.into(),
            param as i32,
            step.as_mut_ptr(),
            small_step.as_mut_ptr(),
            large_step.as_mut_ptr(),
            is_toggle.as_mut_ptr(),
        );
        if !successful {
            return None;
        }
        let is_toggle = is_toggle.assume_init();
        if is_toggle {
            Some(GetParameterStepSizesResult::Toggle)
        } else {
            Some(GetParameterStepSizesResult::Normal {
                step: step.assume_init(),
                small_step: make_some_if_greater_than_zero(small_step.assume_init()),
                large_step: make_some_if_greater_than_zero(large_step.assume_init()),
            })
        }
    }

    pub unsafe fn track_fx_get_param_ex(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
        param: u32,
    ) -> GetParamExResult {
        let mut min_val = MaybeUninit::uninit();
        let mut max_val = MaybeUninit::uninit();
        let mut mid_val = MaybeUninit::uninit();
        let value = self.low.TrackFX_GetParamEx(
            track.as_ptr(),
            fx.into(),
            param as i32,
            min_val.as_mut_ptr(),
            max_val.as_mut_ptr(),
            mid_val.as_mut_ptr(),
        );
        GetParamExResult {
            value,
            min_val: min_val.assume_init(),
            mid_val: mid_val.assume_init(),
            max_val: max_val.assume_init(),
        }
        .into()
    }

    /// # Example
    ///
    /// ```no_run
    /// # let reaper = reaper_rs_medium::Reaper::new(reaper_rs_low::Reaper::default());
    /// use reaper_rs_medium::{ProjectContext::CurrentProject, UndoScope::Scoped, UndoFlag::*};
    ///
    /// reaper.undo_begin_block_2(CurrentProject);
    /// // ... do something incredible ...
    /// reaper.undo_end_block_2(CurrentProject, "Do something incredible", Scoped(Items | Fx));
    /// ```
    pub fn undo_begin_block_2(&self, proj: ProjectContext) {
        self.require_valid_project(proj);
        unsafe { self.undo_begin_block_2_unchecked(proj) };
    }

    pub unsafe fn undo_begin_block_2_unchecked(&self, proj: ProjectContext) {
        self.low.Undo_BeginBlock2(proj.into());
    }

    pub fn undo_end_block_2<'a>(
        &self,
        proj: ProjectContext,
        descchange: impl Into<ReaperStringArg<'a>>,
        extraflags: UndoScope,
    ) {
        unsafe {
            self.undo_end_block_2_unchecked(proj, descchange, extraflags);
        }
    }

    pub unsafe fn undo_end_block_2_unchecked<'a>(
        &self,
        proj: ProjectContext,
        descchange: impl Into<ReaperStringArg<'a>>,
        extraflags: UndoScope,
    ) {
        self.low
            .Undo_EndBlock2(proj.into(), descchange.into().as_ptr(), extraflags.into());
    }

    pub fn undo_can_undo_2<R>(
        &self,
        proj: ProjectContext,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R> {
        self.require_valid_project(proj);
        unsafe { self.undo_can_undo_2_unchecked(proj, f) }
    }

    pub unsafe fn undo_can_undo_2_unchecked<R>(
        &self,
        proj: ProjectContext,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R> {
        let ptr = self.low.Undo_CanUndo2(proj.into());
        create_passing_c_str(ptr).map(f)
    }

    pub fn undo_can_redo_2<R>(
        &self,
        proj: ProjectContext,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R> {
        self.require_valid_project(proj);
        unsafe { self.undo_can_redo_2_unchecked(proj, f) }
    }

    pub unsafe fn undo_can_redo_2_unchecked<R>(
        &self,
        proj: ProjectContext,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R> {
        let ptr = self.low.Undo_CanRedo2(proj.into());
        create_passing_c_str(ptr).map(f)
    }

    // Returns true if there was something to be undone, false if not
    pub fn undo_do_undo_2(&self, proj: ProjectContext) -> bool {
        self.require_valid_project(proj);
        unsafe { self.undo_do_undo_2_unchecked(proj) }
    }

    pub unsafe fn undo_do_undo_2_unchecked(&self, proj: ProjectContext) -> bool {
        self.low.Undo_DoUndo2(proj.into()) != 0
    }

    // Returns true if there was something to be redone, false if not
    pub fn undo_do_redo_2(&self, proj: ProjectContext) -> bool {
        self.require_valid_project(proj);
        unsafe { self.undo_do_redo_2_unchecked(proj) }
    }

    pub unsafe fn undo_do_redo_2_unchecked(&self, proj: ProjectContext) -> bool {
        self.low.Undo_DoRedo2(proj.into()) != 0
    }

    pub fn mark_project_dirty(&self, proj: ProjectContext) {
        self.require_valid_project(proj);
        unsafe {
            self.mark_project_dirty_unchecked(proj);
        }
    }

    pub unsafe fn mark_project_dirty_unchecked(&self, proj: ProjectContext) {
        self.low.MarkProjectDirty(proj.into());
    }

    // Returns true if project dirty, false if not
    pub fn is_project_dirty(&self, proj: ProjectContext) -> bool {
        self.require_valid_project(proj);
        unsafe { self.is_project_dirty_unchecked(proj) }
    }

    pub unsafe fn is_project_dirty_unchecked(&self, proj: ProjectContext) -> bool {
        self.low.IsProjectDirty(proj.into()) != 0
    }

    pub fn track_list_update_all_external_surfaces(&self) {
        self.low.TrackList_UpdateAllExternalSurfaces();
    }

    pub fn get_app_version(&self) -> ReaperVersion {
        let ptr = self.low.GetAppVersion();
        let version_str = unsafe { CStr::from_ptr(ptr) };
        ReaperVersion::from(version_str)
    }

    pub unsafe fn get_track_automation_mode(&self, tr: MediaTrack) -> AutomationMode {
        let result = self.low.GetTrackAutomationMode(tr.as_ptr());
        AutomationMode::try_from(result).expect("Unknown automation mode")
    }

    pub fn get_global_automation_override(&self) -> Option<GlobalAutomationOverride> {
        use GlobalAutomationOverride::*;
        match self.low.GetGlobalAutomationOverride() {
            -1 => None,
            6 => Some(Bypass),
            x => Some(Mode(x.try_into().expect("Unknown automation mode"))),
        }
    }

    // TODO-low Test
    pub unsafe fn get_track_envelope_by_chunk_name(
        &self,
        track: MediaTrack,
        cfgchunkname: EnvChunkName,
    ) -> Option<TrackEnvelope> {
        let ptr = self
            .low
            .GetTrackEnvelopeByChunkName(track.as_ptr(), Cow::from(cfgchunkname).as_ptr());
        NonNull::new(ptr)
    }

    // For getting common envelopes (like volume or pan) I recommend using
    // get_track_envelope_by_chunk_name
    pub unsafe fn get_track_envelope_by_name<'a>(
        &self,
        track: MediaTrack,
        envname: impl Into<ReaperStringArg<'a>>,
    ) -> Option<TrackEnvelope> {
        let ptr = self
            .low
            .GetTrackEnvelopeByName(track.as_ptr(), envname.into().as_ptr());
        NonNull::new(ptr)
    }

    pub unsafe fn get_media_track_info_value(&self, tr: MediaTrack, parmname: TrackInfoKey) -> f64 {
        self.low
            .GetMediaTrackInfo_Value(tr.as_ptr(), Cow::from(parmname).as_ptr())
    }

    pub unsafe fn track_fx_get_count(&self, track: MediaTrack) -> u32 {
        self.low.TrackFX_GetCount(track.as_ptr()) as u32
    }

    pub unsafe fn track_fx_get_rec_count(&self, track: MediaTrack) -> u32 {
        self.low.TrackFX_GetRecCount(track.as_ptr()) as u32
    }

    pub unsafe fn track_fx_get_fx_guid(&self, track: MediaTrack, fx: TrackFxRef) -> Option<GUID> {
        let ptr = self.low.TrackFX_GetFXGUID(track.as_ptr(), fx.into());
        unref(ptr)
    }

    pub unsafe fn track_fx_get_param_normalized(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
        param: u32,
    ) -> Result<ReaperNormalizedValue, ()> {
        let raw_value =
            self.low
                .TrackFX_GetParamNormalized(track.as_ptr(), fx.into(), param as i32);
        if raw_value < 0.0 {
            return Err(());
        }
        Ok(ReaperNormalizedValue::new(raw_value))
    }

    pub fn get_master_track(&self, proj: ProjectContext) -> MediaTrack {
        self.require_valid_project(proj);
        unsafe { self.get_master_track_unchecked(proj) }
    }

    pub unsafe fn get_master_track_unchecked(&self, proj: ProjectContext) -> MediaTrack {
        let ptr = self.low.GetMasterTrack(proj.into());
        require_non_null_panic(ptr)
    }

    pub fn guid_to_string(&self, g: &GUID) -> CString {
        let (guid_string, _) = with_string_buffer(64, |buffer, _| unsafe {
            self.low.guidToString(g as *const GUID, buffer)
        });
        guid_string
    }

    pub fn master_get_tempo(&self) -> Bpm {
        Bpm(self.low.Master_GetTempo())
    }

    pub fn set_current_bpm(&self, proj: ProjectContext, bpm: Bpm, want_undo: UndoBehavior) {
        self.require_valid_project(proj);
        unsafe {
            self.set_current_bpm_unchecked(proj, bpm, want_undo);
        }
    }

    pub unsafe fn set_current_bpm_unchecked(
        &self,
        proj: ProjectContext,
        bpm: Bpm,
        want_undo: UndoBehavior,
    ) {
        self.low.SetCurrentBPM(
            proj.into(),
            bpm.get(),
            want_undo == UndoBehavior::AddUndoPoint,
        );
    }

    pub fn master_get_play_rate(&self, project: ProjectContext) -> PlaybackSpeedFactor {
        self.require_valid_project(project);
        unsafe { self.master_get_play_rate_unchecked(project) }
    }

    pub unsafe fn master_get_play_rate_unchecked(
        &self,
        project: ProjectContext,
    ) -> PlaybackSpeedFactor {
        let raw = unsafe { self.low.Master_GetPlayRate(project.into()) };
        PlaybackSpeedFactor(raw)
    }

    pub fn csurf_on_play_rate_change(&self, playrate: PlaybackSpeedFactor) {
        self.low.CSurf_OnPlayRateChange(playrate.get());
    }

    pub fn show_message_box<'a>(
        &self,
        msg: impl Into<ReaperStringArg<'a>>,
        title: impl Into<ReaperStringArg<'a>>,
        r#type: MessageBoxType,
    ) -> MessageBoxResult {
        let result = unsafe {
            self.low
                .ShowMessageBox(msg.into().as_ptr(), title.into().as_ptr(), r#type.into())
        };
        result.try_into().expect("Unknown message box result")
    }

    // Returns Err if given string is not a valid GUID string
    pub fn string_to_guid<'a>(&self, str: impl Into<ReaperStringArg<'a>>) -> Result<GUID, ()> {
        let mut guid = MaybeUninit::uninit();
        unsafe {
            self.low
                .stringToGuid(str.into().as_ptr(), guid.as_mut_ptr());
        }
        let guid = unsafe { guid.assume_init() };
        if guid == ZERO_GUID {
            return Err(());
        }
        Ok(guid)
    }

    pub unsafe fn csurf_on_input_monitoring_change_ex(
        &self,
        trackid: MediaTrack,
        monitor: InputMonitoringMode,
        allowgang: GangBehavior,
    ) -> i32 {
        self.low.CSurf_OnInputMonitorChangeEx(
            trackid.as_ptr(),
            monitor.into(),
            allowgang == GangBehavior::AllowGang,
        )
    }

    // Returns Err if invalid parameter name given (maybe also in other situations)
    pub unsafe fn set_media_track_info_value(
        &self,
        tr: MediaTrack,
        parmname: TrackInfoKey,
        newvalue: f64,
    ) -> Result<(), ()> {
        let successful =
            self.low
                .SetMediaTrackInfo_Value(tr.as_ptr(), Cow::from(parmname).as_ptr(), newvalue);
        if !successful {
            return Err(());
        }
        Ok(())
    }

    pub fn stuff_midimessage(&self, mode: StuffMidiMessageTarget, msg: impl ShortMessage) {
        let bytes = msg.to_bytes();
        self.low
            .StuffMIDIMessage(mode.into(), bytes.0.into(), bytes.1.into(), bytes.2.into());
    }

    pub fn db2slider(&self, x: Db) -> VolumeSliderValue {
        VolumeSliderValue(self.low.DB2SLIDER(x.get()))
    }

    pub fn slider2db(&self, y: VolumeSliderValue) -> Db {
        Db(self.low.SLIDER2DB(y.get()))
    }

    // I guess it returns Err if the track doesn't exist
    pub unsafe fn get_track_ui_vol_pan(&self, track: MediaTrack) -> Result<VolumeAndPan, ()> {
        let mut volume = MaybeUninit::uninit();
        let mut pan = MaybeUninit::uninit();
        let successful =
            self.low
                .GetTrackUIVolPan(track.as_ptr(), volume.as_mut_ptr(), pan.as_mut_ptr());
        if !successful {
            return Err(());
        }
        Ok(VolumeAndPan {
            volume: ReaperVolumeValue::new(volume.assume_init()),
            pan: ReaperPanValue::new(pan.assume_init()),
        })
    }

    // The given audio_hook_register_t will be modified by REAPER. After registering it, it must
    // only be accessed from within OnAudioBuffer callback (passed as param).
    // Returns true on success
    pub unsafe fn audio_reg_hardware_hook_add_unchecked(
        &mut self,
        reg: NonNull<audio_hook_register_t>,
    ) -> Result<(), ()> {
        self.audio_hook_registrations.insert(reg);
        let result = self.low.Audio_RegHardwareHook(true, reg.as_ptr());
        ok_if_one(result)
    }

    pub unsafe fn audio_reg_hardware_hook_remove_unchecked(
        &mut self,
        reg: NonNull<audio_hook_register_t>,
    ) {
        self.low.Audio_RegHardwareHook(false, reg.as_ptr());
        self.audio_hook_registrations.remove(&reg);
    }

    pub fn audio_reg_hardware_hook_add(
        &mut self,
        reg: MediumAudioHookRegister,
    ) -> Result<NonNull<audio_hook_register_t>, ()> {
        let handle = self.audio_hook_registers.keep(reg);
        unsafe { self.audio_reg_hardware_hook_add_unchecked(handle)? };
        Ok(handle)
    }

    pub fn audio_reg_hardware_hook_remove(
        &mut self,
        reg_handle: NonNull<audio_hook_register_t>,
    ) -> Result<MediumAudioHookRegister, ()> {
        let original = self.audio_hook_registers.release(reg_handle).ok_or(())?;
        unsafe { self.audio_reg_hardware_hook_remove_unchecked(reg_handle) };
        Ok(original)
    }

    pub unsafe fn csurf_set_surface_volume(
        &self,
        trackid: MediaTrack,
        volume: ReaperVolumeValue,
        ignoresurf: NotificationBehavior,
    ) {
        self.low
            .CSurf_SetSurfaceVolume(trackid.as_ptr(), volume.get(), ignoresurf.into());
    }

    // Returns the value that has actually been set. This only deviates if 0 is sent. Then it
    // returns a very slightly higher value - the one which actually corresponds to -150 dB.
    pub unsafe fn csurf_on_volume_change_ex(
        &self,
        trackid: MediaTrack,
        volume: ValueChange<ReaperVolumeValue>,
        allow_gang: GangBehavior,
    ) -> ReaperVolumeValue {
        let raw = self.low.CSurf_OnVolumeChangeEx(
            trackid.as_ptr(),
            volume.value(),
            volume.is_relative(),
            allow_gang == GangBehavior::AllowGang,
        );
        ReaperVolumeValue::new(raw)
    }

    pub unsafe fn csurf_set_surface_pan(
        &self,
        trackid: MediaTrack,
        pan: ReaperPanValue,
        ignoresurf: NotificationBehavior,
    ) {
        self.low
            .CSurf_SetSurfacePan(trackid.as_ptr(), pan.get(), ignoresurf.into());
    }

    pub unsafe fn csurf_on_pan_change_ex(
        &self,
        trackid: MediaTrack,
        pan: ValueChange<ReaperPanValue>,
        allow_gang: GangBehavior,
    ) -> ReaperPanValue {
        let raw = self.low.CSurf_OnPanChangeEx(
            trackid.as_ptr(),
            pan.value(),
            pan.is_relative(),
            allow_gang == GangBehavior::AllowGang,
        );
        ReaperPanValue::new(raw)
    }

    pub fn count_selected_tracks_2(
        &self,
        proj: ProjectContext,
        wantmaster: MasterTrackBehavior,
    ) -> u32 {
        self.require_valid_project(proj);
        unsafe { self.count_selected_tracks_2_unchecked(proj, wantmaster) }
    }

    pub unsafe fn count_selected_tracks_2_unchecked(
        &self,
        proj: ProjectContext,
        wantmaster: MasterTrackBehavior,
    ) -> u32 {
        self.low.CountSelectedTracks2(
            proj.into(),
            wantmaster == MasterTrackBehavior::IncludeMasterTrack,
        ) as u32
    }

    pub unsafe fn set_track_selected(&self, track: MediaTrack, selected: bool) {
        self.low.SetTrackSelected(track.as_ptr(), selected);
    }

    pub fn get_selected_track_2(
        &self,
        proj: ProjectContext,
        seltrackidx: u32,
        wantmaster: MasterTrackBehavior,
    ) -> Option<MediaTrack> {
        self.require_valid_project(proj);
        unsafe { self.get_selected_track_2_unchecked(proj, seltrackidx, wantmaster) }
    }

    pub unsafe fn get_selected_track_2_unchecked(
        &self,
        proj: ProjectContext,
        seltrackidx: u32,
        wantmaster: MasterTrackBehavior,
    ) -> Option<MediaTrack> {
        let ptr = self.low.GetSelectedTrack2(
            proj.into(),
            seltrackidx as i32,
            wantmaster == MasterTrackBehavior::IncludeMasterTrack,
        );
        NonNull::new(ptr)
    }

    pub unsafe fn set_only_track_selected(&self, track: Option<MediaTrack>) {
        let ptr = match track {
            None => null_mut(),
            Some(t) => t.as_ptr(),
        };
        self.low.SetOnlyTrackSelected(ptr);
    }

    pub unsafe fn delete_track(&self, tr: MediaTrack) {
        self.low.DeleteTrack(tr.as_ptr());
    }

    pub unsafe fn get_track_num_sends(&self, tr: MediaTrack, category: TrackSendCategory) -> u32 {
        self.low.GetTrackNumSends(tr.as_ptr(), category.into()) as u32
    }

    pub unsafe fn get_set_track_send_info(
        &self,
        tr: MediaTrack,
        category: TrackSendCategory,
        sendidx: u32,
        parmname: TrackSendInfoKey,
        set_new_value: *mut c_void,
    ) -> *mut c_void {
        self.low.GetSetTrackSendInfo(
            tr.as_ptr(),
            category.into(),
            sendidx as i32,
            Cow::from(parmname).as_ptr(),
            set_new_value,
        )
    }

    pub unsafe fn get_track_send_info_desttrack(
        &self,
        tr: MediaTrack,
        category: TrackSendDirection,
        sendidx: u32,
    ) -> Result<MediaTrack, ()> {
        let ptr = self.get_set_track_send_info(
            tr,
            category.into(),
            sendidx,
            TrackSendInfoKey::DestTrack,
            null_mut(),
        ) as *mut raw::MediaTrack;
        require_non_null(ptr)
    }

    // I guess it returns Err if the track doesn't exist
    pub unsafe fn get_track_state_chunk(
        &self,
        track: MediaTrack,
        str_need_big_sz: u32,
        isundo_optional: ChunkCacheHint,
    ) -> Result<CString, ()> {
        let (chunk_content, successful) =
            with_string_buffer(str_need_big_sz, |buffer, max_size| {
                self.low.GetTrackStateChunk(
                    track.as_ptr(),
                    buffer,
                    max_size,
                    isundo_optional == ChunkCacheHint::UndoMode,
                )
            });
        if !successful {
            return Err(());
        }
        Ok(chunk_content)
    }

    /// # Example
    ///
    /// ```no_run
    /// # let reaper = reaper_rs_medium::Reaper::new(reaper_rs_low::Reaper::default());
    /// use reaper_rs_medium::{ProjectContext::CurrentProject, SendTarget::HardwareOutput};
    ///
    /// let src_track = reaper.get_track(CurrentProject, 0).ok_or("Source track doesn't exist")?;
    /// let send_index = unsafe {
    ///     reaper.create_track_send(src_track, HardwareOutput)?;
    /// };
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub unsafe fn create_track_send(
        &self,
        tr: MediaTrack,
        desttr_in_optional: SendTarget,
    ) -> Result<u32, CreateTrackSendFailed> {
        let result = self
            .low
            .CreateTrackSend(tr.as_ptr(), desttr_in_optional.into());
        if result < 0 {
            return Err(CreateTrackSendFailed);
        }
        Ok(result as u32)
    }

    // Seems to return true if was armed and false if not
    pub unsafe fn csurf_on_rec_arm_change_ex(
        &self,
        trackid: MediaTrack,
        recarm: RecordArmState,
        allowgang: GangBehavior,
    ) -> bool {
        self.low.CSurf_OnRecArmChangeEx(
            trackid.as_ptr(),
            recarm.into(),
            allowgang == GangBehavior::AllowGang,
        )
    }

    // Returns Err for example if given chunk is invalid
    pub unsafe fn set_track_state_chunk<'a>(
        &self,
        track: MediaTrack,
        str: impl Into<ReaperStringArg<'a>>,
        isundo_optional: ChunkCacheHint,
    ) -> Result<(), ()> {
        let successful = self.low.SetTrackStateChunk(
            track.as_ptr(),
            str.into().as_ptr(),
            isundo_optional == ChunkCacheHint::UndoMode,
        );
        if !successful {
            return Err(());
        }
        Ok(())
    }

    pub unsafe fn track_fx_show(
        &self,
        track: MediaTrack,
        index: TrackFxRef,
        show_flag: FxShowFlag,
    ) {
        self.low
            .TrackFX_Show(track.as_ptr(), index.into(), show_flag.into());
    }

    pub unsafe fn track_fx_get_floating_window(
        &self,
        track: MediaTrack,
        index: TrackFxRef,
    ) -> Option<Hwnd> {
        let ptr = self
            .low
            .TrackFX_GetFloatingWindow(track.as_ptr(), index.into());
        NonNull::new(ptr)
    }

    pub unsafe fn track_fx_get_open(&self, track: MediaTrack, fx: TrackFxRef) -> bool {
        self.low.TrackFX_GetOpen(track.as_ptr(), fx.into())
    }

    // Returns the value which has actually been set. If the send doesn't exist, returns 0.0 (which
    // can also be a valid value that has been set, so not very useful ...).
    pub unsafe fn csurf_on_send_volume_change(
        &self,
        trackid: MediaTrack,
        send_index: u32,
        volume: ValueChange<ReaperVolumeValue>,
    ) -> ReaperVolumeValue {
        let raw = self.low.CSurf_OnSendVolumeChange(
            trackid.as_ptr(),
            send_index as i32,
            volume.value(),
            volume.is_relative(),
        );
        ReaperVolumeValue::new(raw)
    }

    pub unsafe fn csurf_on_send_pan_change(
        &self,
        trackid: MediaTrack,
        send_index: u32,
        pan: ValueChange<ReaperPanValue>,
    ) -> ReaperPanValue {
        let raw = self.low.CSurf_OnSendPanChange(
            trackid.as_ptr(),
            send_index as i32,
            pan.value(),
            pan.is_relative(),
        );
        ReaperPanValue::new(raw)
    }

    // Returns None if section or command not existing (can't think of any other case)
    pub unsafe fn kbd_get_text_from_cmd<R>(
        &self,
        cmd: CommandId,
        section: SectionContext,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R> {
        let ptr = self.low.kbd_getTextFromCmd(cmd.get(), section.into());
        create_passing_c_str(ptr)
            // Removed action returns empty string for some reason. We want None in this case!
            .filter(|s| s.to_bytes().len() > 0)
            .map(f)
    }

    // Returns None if action doesn't report on/off states (or if action not existing).
    // Option makes more sense than Result here because this function is at the same time the
    // correct function to be used to determine *if* an action reports on/off states. So
    // "action doesn't report on/off states" is a valid result.
    pub unsafe fn get_toggle_command_state_2(
        &self,
        section: SectionContext,
        command_id: CommandId,
    ) -> Option<bool> {
        let result = self
            .low
            .GetToggleCommandState2(section.into(), command_id.into());
        if result == -1 {
            return None;
        }
        return Some(result != 0);
    }

    // Returns None if lookup was not successful, that is, the command couldn't be found
    pub fn reverse_named_command_lookup<R>(
        &self,
        command_id: CommandId,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R> {
        let ptr = self.low.ReverseNamedCommandLookup(command_id.into());
        unsafe { create_passing_c_str(ptr) }.map(f)
    }

    // Returns Err if send not existing
    pub unsafe fn get_track_send_ui_vol_pan(
        &self,
        track: MediaTrack,
        send_index: u32,
    ) -> Result<VolumeAndPan, ()> {
        let mut volume = MaybeUninit::uninit();
        let mut pan = MaybeUninit::uninit();
        let successful = self.low.GetTrackSendUIVolPan(
            track.as_ptr(),
            send_index as i32,
            volume.as_mut_ptr(),
            pan.as_mut_ptr(),
        );
        if !successful {
            return Err(());
        }
        Ok(VolumeAndPan {
            volume: ReaperVolumeValue::new(volume.assume_init()),
            pan: ReaperPanValue::new(pan.assume_init()),
        })
    }

    // Returns Err e.g. if FX doesn't exist
    pub unsafe fn track_fx_get_preset_index(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
    ) -> Result<TrackFxGetPresetIndexResult, ()> {
        let mut num_presets = MaybeUninit::uninit();
        let index =
            self.low
                .TrackFX_GetPresetIndex(track.as_ptr(), fx.into(), num_presets.as_mut_ptr());
        if index == -1 {
            return Err(());
        }
        Ok(TrackFxGetPresetIndexResult {
            index: index as u32,
            count: num_presets.assume_init() as u32,
        })
    }

    // Returns Err e.g. if FX or preset doesn't exist
    pub unsafe fn track_fx_set_preset_by_index(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
        idx: FxPresetRef,
    ) -> Result<(), ()> {
        let successful = self
            .low
            .TrackFX_SetPresetByIndex(track.as_ptr(), fx.into(), idx.into());
        if !successful {
            return Err(());
        }
        Ok(())
    }

    // Returns Err e.g. if FX doesn't exist
    pub unsafe fn track_fx_navigate_presets(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
        presetmove: i32,
    ) -> Result<(), ()> {
        let successful = self
            .low
            .TrackFX_NavigatePresets(track.as_ptr(), fx.into(), presetmove);
        if !successful {
            return Err(());
        }
        Ok(())
    }

    pub unsafe fn track_fx_get_preset(
        &self,
        track: MediaTrack,
        fx: TrackFxRef,
        presetname_sz: u32,
    ) -> TrackFxGetPresetResult {
        if presetname_sz == 0 {
            let state_matches_preset =
                self.low
                    .TrackFX_GetPreset(track.as_ptr(), fx.into(), null_mut(), 0);
            TrackFxGetPresetResult {
                state_matches_preset,
                name: None,
            }
        } else {
            let (name, state_matches_preset) =
                with_string_buffer(presetname_sz, |buffer, max_size| {
                    self.low
                        .TrackFX_GetPreset(track.as_ptr(), fx.into(), buffer, max_size)
                });
            if name.to_bytes().len() == 0 {
                return TrackFxGetPresetResult {
                    state_matches_preset,
                    name: None,
                };
            }
            TrackFxGetPresetResult {
                state_matches_preset,
                name: Some(name),
            }
        }
    }

    fn require_valid_project(&self, proj: ProjectContext) {
        if let ProjectContext::Proj(p) = proj {
            assert!(
                self.validate_ptr_2(CurrentProject, p),
                "ReaProject doesn't exist anymore"
            )
        }
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

pub enum GetParameterStepSizesResult {
    // Each of the decimal numbers are > 0
    Normal {
        step: f64,
        small_step: Option<f64>,
        large_step: Option<f64>,
    },
    Toggle,
}

// Each of the attributes can be negative! These are not normalized values (0..1).
pub struct GetParamExResult {
    pub value: f64,
    pub min_val: f64,
    pub mid_val: f64,
    pub max_val: f64,
}

pub struct EnumProjectsResult {
    pub project: ReaProject,
    pub file_path: Option<PathBuf>,
}

pub struct GetMidiDevNameResult {
    pub is_present: bool,
    pub name: Option<CString>,
}

pub struct TrackFxGetPresetResult {
    pub state_matches_preset: bool,
    pub name: Option<CString>,
}

pub struct TrackFxGetPresetIndexResult {
    pub index: u32,
    pub count: u32,
}

#[derive(Debug)]
pub struct VolumeAndPan {
    pub volume: ReaperVolumeValue,
    pub pan: ReaperPanValue,
}

pub enum GetLastTouchedFxResult {
    TrackFx {
        track_ref: TrackRef,
        fx_ref: TrackFxRef,
        param_index: u32,
    },
    ItemFx {
        track_index: u32,
        /// **Attention:** It's an index so it's zero-based (the one-based result from the
        /// low-level function is transformed).
        item_index: u32,
        take_index: u32,
        fx_index: u32,
        param_index: u32,
    },
}

pub enum GetFocusedFxResult {
    TrackFx {
        track_ref: TrackRef,
        fx_ref: TrackFxRef,
    },
    ItemFx {
        track_index: u32,
        item_index: u32,
        take_index: u32,
        fx_index: u32,
    },
}

fn make_some_if_greater_than_zero(value: f64) -> Option<f64> {
    if value <= 0.0 || value.is_nan() {
        return None;
    }
    Some(value)
}

unsafe fn unref<T: Copy>(ptr: *const T) -> Option<T> {
    if ptr.is_null() {
        return None;
    }
    Some(*ptr)
}

unsafe fn unref_as<T: Copy>(ptr: *mut c_void) -> Option<T> {
    unref(ptr as *const T)
}

unsafe fn create_passing_c_str<'a>(ptr: *const c_char) -> Option<&'a CStr> {
    if ptr.is_null() {
        return None;
    }
    Some(CStr::from_ptr(ptr))
}

fn convert_tracknumber_to_track_ref(tracknumber: u32) -> TrackRef {
    if tracknumber == 0 {
        TrackRef::MasterTrack
    } else {
        TrackRef::NormalTrack(tracknumber - 1)
    }
}

fn ok_if_one(result: i32) -> Result<(), ()> {
    if result == 1 { Ok(()) } else { Err(()) }
}
