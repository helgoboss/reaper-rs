use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr::null_mut;

use c_str_macro::c_str;

use crate::low_level;
use crate::low_level::raw::{
    audio_hook_register_t, gaccel_register_t, midi_Input, midi_Output, IReaperControlSurface,
    KbdSectionInfo, MediaTrack, ReaProject, TrackEnvelope, GUID, HWND, UNDO_STATE_ALL,
};
use crate::low_level::{get_cpp_control_surface, install_control_surface};
use crate::medium_level::{
    AutomationMode, ControlSurface, DelegatingControlSurface, ExtensionType, FxShowFlag, Gang,
    GlobalAutomationOverride, HookCommand, HookPostCommand, InputMonitoringMode, KbdActionValue,
    MessageBoxResult, MessageBoxType, ProjectRef, ReaperPointerType, ReaperStringArg,
    ReaperVersion, RecArmState, RecordingInput, RegInstr, SendOrReceive, StuffMidiMessageTarget,
    ToggleAction, TrackFxAddByNameVariant, TrackFxRef, TrackInfoKey, TrackRef, TrackSendCategory,
    TrackSendInfoKey, UndoFlag,
};
use enumflags2::BitFlags;
use helgoboss_midi::MidiMessage;
use std::convert::{TryFrom, TryInto};
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};

/// This is the medium-level API access point to all REAPER functions. In order to use it, you first
/// must obtain an instance of this struct by invoking [`new`](struct.Reaper.html#method.new).
///
/// It's always possible that a function from the low-level API is missing in the medium-level one.
/// That's because unlike the low-level API, the medium-level API is hand-written and a perpetual
/// work in progress. If you can't find the function that you need, you can always resort to the
/// low-level API by navigating to [`low`](struct.Reaper.html#structfield.low). Of course you are
/// welcome to contribute to bring the medium-level API on par with the low-level one.  
pub struct Reaper {
    /// Returns the low-level REAPER instance
    pub low: low_level::Reaper,
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
    /// [`low_level::Reaper`](../../low_level/struct.Reaper.html) instance.
    pub fn new(low: low_level::Reaper) -> Reaper {
        Reaper { low }
    }

    /// Returns the requested project and optionally its file name.
    ///
    /// With `projfn_out_optional_sz` you can tell REAPER how many characters of the file name you
    /// want. If you are not interested in the file name at all, pass 0.
    pub fn enum_projects(
        &self,
        proj_ref: ProjectRef,
        projfn_out_optional_sz: u32,
    ) -> (*mut ReaProject, Option<PathBuf>) {
        use ProjectRef::*;
        let idx = match proj_ref {
            Current => -1,
            CurrentlyRendering => 0x40000000,
            TabIndex(i) => i as i32,
        };
        if projfn_out_optional_sz == 0 {
            let project = unsafe { self.low.EnumProjects(idx, null_mut(), 0) };
            (project, None)
        } else {
            let (owned_c_string, project) =
                with_string_buffer(projfn_out_optional_sz, |buffer, max_size| unsafe {
                    self.low.EnumProjects(idx, buffer, max_size)
                });
            if owned_c_string.to_bytes().len() == 0 {
                return (project, None);
            }
            let owned_string = owned_c_string
                .into_string()
                .expect("Path contains non-UTF8 characters");
            (project, Some(PathBuf::from(owned_string)))
        }
    }

    /// Returns the track at the given index. Set `proj` to `null_mut()` in order to look for tracks
    /// in the current project.
    pub fn get_track(&self, proj: *mut ReaProject, trackidx: u32) -> *mut MediaTrack {
        unsafe { self.low.GetTrack(proj, trackidx as i32) }
    }

    // TODO If we take pointers in medium-level methods, even they should be marked unsafe. Because
    //  we can do anything with pointers, e.g. pretend we are passing MediaTrack when we actually
    //  pass a ReaProject. Maybe we should introduce newtypes. Definitely! I checked with
    //  plugin_register(). If you pretend a toggle action to be a hook command, then things
    //  stop working! Completely. This is undefined behavior and this is supposed to be marked as
    //  unsafe.
    /// Returns `true` if the given pointer is a valid object of the right type in project `proj`
    /// (`proj` is ignored if pointer is itself a project).
    pub fn validate_ptr_2(
        &self,
        proj: *mut ReaProject,
        pointer: *mut c_void,
        ctypename: ReaperPointerType,
    ) -> bool {
        unsafe {
            self.low
                .ValidatePtr2(proj, pointer, Cow::from(ctypename).as_ptr())
        }
    }

    /// Shows a message to the user (also useful for debugging). Send "\n" for newline and "" to
    /// clear the console.
    pub fn show_console_msg<'a>(&self, msg: impl Into<ReaperStringArg<'a>>) {
        unsafe { self.low.ShowConsoleMsg(msg.into().as_ptr()) }
    }

    /// Gets or sets track arbitrary attributes. This just delegates to the low-level analog. Using
    /// this function is not fun and requires you to use unsafe code. Consider using one of
    /// type-safe convenience functions instead. They start with `get_media_track_info_` or
    /// `set_media_track_info_`.
    pub unsafe fn get_set_media_track_info(
        &self,
        tr: *mut MediaTrack,
        parmname: TrackInfoKey,
        set_new_value: *mut c_void,
    ) -> *mut c_void {
        self.low
            .GetSetMediaTrackInfo(tr, Cow::from(parmname).as_ptr(), set_new_value)
    }

    fn get_media_track_info(&self, tr: *mut MediaTrack, parmname: TrackInfoKey) -> *mut c_void {
        unsafe { self.get_set_media_track_info(tr, parmname, null_mut()) }
    }

    /// Convenience function which returns the given track's parent track (`P_PARTRACK`).
    pub fn get_media_track_info_partrack(&self, tr: *mut MediaTrack) -> *mut MediaTrack {
        self.get_media_track_info(tr, TrackInfoKey::P_PARTRACK) as *mut MediaTrack
    }

    /// Convenience function which returns the given track's parent project (`P_PROJECT`).
    pub fn get_media_track_info_project(&self, tr: *mut MediaTrack) -> *mut ReaProject {
        self.get_media_track_info(tr, TrackInfoKey::P_PROJECT) as *mut ReaProject
    }

    /// Convenience function which let's you use the given track's name (`P_NAME`).
    pub fn get_media_track_info_name<R>(
        &self,
        tr: *mut MediaTrack,
        f: impl Fn(&CStr) -> R,
    ) -> Option<R> {
        let ptr = self.get_media_track_info(tr, TrackInfoKey::P_NAME);
        unsafe { create_passing_c_str(ptr as *const c_char) }.map(f)
    }

    /// Convenience function which returns the given track's input monitoring mode (I_RECMON).
    pub fn get_media_track_info_recmon(&self, tr: *mut MediaTrack) -> InputMonitoringMode {
        let ptr = self.get_media_track_info(tr, TrackInfoKey::I_RECMON);
        let irecmon = unsafe { copy_void_ptr_target::<i32>(ptr) }.unwrap();
        InputMonitoringMode::try_from(irecmon).expect("Unknown input monitoring mode")
    }

    /// Convenience function which returns the given track's recording input (I_RECINPUT).
    pub fn get_media_track_info_recinput(&self, tr: *mut MediaTrack) -> RecordingInput {
        let ptr = self.get_media_track_info(tr, TrackInfoKey::I_RECINPUT);
        let rec_input_index = unsafe { copy_void_ptr_target::<i32>(ptr) }.unwrap();
        RecordingInput::from_rec_input_index(rec_input_index)
    }

    /// Convenience function which returns the given track's number (IP_TRACKNUMBER).
    pub fn get_media_track_info_tracknumber(&self, tr: *mut MediaTrack) -> Option<TrackRef> {
        use TrackRef::*;
        match self.get_media_track_info(tr, TrackInfoKey::IP_TRACKNUMBER) as i32 {
            -1 => Some(MasterTrack),
            0 => None,
            n if n > 0 => Some(TrackIndex(n as u32 - 1)),
            _ => unreachable!(),
        }
    }

    /// Convenience function which returns the given track's GUID (GUID).
    pub fn get_media_track_info_guid(&self, tr: *mut MediaTrack) -> GUID {
        let ptr = self.get_media_track_info(tr, TrackInfoKey::GUID);
        unsafe { copy_void_ptr_target::<GUID>(ptr) }.unwrap()
    }

    // TODO Doc
    // Kept return value type i32 because meaning of return value depends very much on the actual
    // thing which is registered and probably is not safe to generalize.
    pub unsafe fn plugin_register(&self, name: RegInstr, infostruct: *mut c_void) -> i32 {
        self.low
            .plugin_register(Cow::from(name).as_ptr(), infostruct)
    }

    // TODO Doc
    pub fn plugin_register_hookcommand(&self, hookcommand: HookCommand) -> Result<(), ()> {
        let result = unsafe {
            self.plugin_register(
                RegInstr::Register(ExtensionType::HookCommand),
                hookcommand as *mut c_void,
            )
        };
        ok_if_one(result)
    }

    // TODO Doc
    pub fn plugin_unregister_hookcommand(&self, hookcommand: HookCommand) {
        unsafe {
            self.plugin_register(
                RegInstr::Unregister(ExtensionType::HookCommand),
                hookcommand as *mut c_void,
            );
        }
    }

    // TODO Doc
    pub fn plugin_register_toggleaction(&self, toggleaction: ToggleAction) -> Result<(), ()> {
        let result = unsafe {
            self.plugin_register(
                RegInstr::Register(ExtensionType::ToggleAction),
                toggleaction as *mut c_void,
            )
        };
        ok_if_one(result)
    }

    // TODO Doc
    pub fn plugin_unregister_toggleaction(&self, toggleaction: ToggleAction) {
        unsafe {
            self.plugin_register(
                RegInstr::Unregister(ExtensionType::ToggleAction),
                toggleaction as *mut c_void,
            );
        }
    }

    // TODO Doc
    pub fn plugin_register_hookpostcommand(
        &self,
        hookpostcommand: HookPostCommand,
    ) -> Result<(), ()> {
        let result = unsafe {
            self.plugin_register(
                RegInstr::Register(ExtensionType::HookPostCommand),
                hookpostcommand as *mut c_void,
            )
        };
        ok_if_one(result)
    }

    // TODO Doc
    pub fn plugin_unregister_hookpostcommand(&self, hookpostcommand: HookPostCommand) {
        unsafe {
            self.plugin_register(
                RegInstr::Unregister(ExtensionType::HookPostCommand),
                hookpostcommand as *mut c_void,
            );
        }
    }

    // Returns the assigned command index.
    // If the command ID is already used, it just returns the index which has been assigned before.
    // Passing an empty string actually works (!). If a null pointer is passed, 0 is returned, but
    // we can't do that using this signature. If a very large string is passed, it works. If a
    // number of a built-in command is passed, it works.
    // TODO Doc
    pub fn plugin_register_command_id<'a>(
        &self,
        command_id: impl Into<ReaperStringArg<'a>>,
    ) -> u32 {
        unsafe {
            self.plugin_register(
                RegInstr::Register(ExtensionType::CommandId),
                command_id.into().as_ptr() as *mut c_void,
            ) as u32
        }
    }

    // TODO Doc
    pub fn plugin_register_gaccel(&self, gaccel: &mut gaccel_register_t) -> Result<(), ()> {
        let result = unsafe {
            self.plugin_register(
                RegInstr::Register(ExtensionType::GAccel),
                gaccel as *mut _ as *mut c_void,
            )
        };
        ok_if_one(result)
    }

    // TODO Doc
    pub fn plugin_unregister_gaccel(&self, gaccel: &mut gaccel_register_t) {
        unsafe {
            self.plugin_register(
                RegInstr::Unregister(ExtensionType::GAccel),
                gaccel as *mut _ as *mut c_void,
            );
        }
    }

    // TODO Doc
    pub fn plugin_register_csurf_inst(
        &self,
        csurf_inst: &mut IReaperControlSurface,
    ) -> Result<(), ()> {
        let result = unsafe {
            self.plugin_register(
                RegInstr::Register(ExtensionType::CSurfInst),
                csurf_inst as *mut _ as *mut c_void,
            )
        };
        ok_if_one(result)
    }

    // TODO Doc
    pub fn plugin_unregister_csurf_inst(&self, csurf_inst: &mut IReaperControlSurface) {
        unsafe {
            self.plugin_register(
                RegInstr::Unregister(ExtensionType::CSurfInst),
                csurf_inst as *mut _ as *mut c_void,
            );
        }
    }

    /// Performs an action belonging to the main action section. To perform non-native actions
    /// (ReaScripts, custom or extension plugins' actions) safely, see
    /// [`named_command_lookup`](struct.Reaper.html#method.named_command_lookup).
    pub fn main_on_command_ex(&self, command: u32, flag: i32, proj: *mut ReaProject) {
        unsafe {
            self.low.Main_OnCommandEx(command as i32, flag, proj);
        }
    }

    // TODO Doc
    pub fn csurf_set_surface_mute(
        &self,
        trackid: *mut MediaTrack,
        mute: bool,
        ignoresurf: *mut IReaperControlSurface,
    ) {
        unsafe {
            self.low.CSurf_SetSurfaceMute(trackid, mute, ignoresurf);
        }
    }

    // TODO Doc
    pub fn csurf_set_surface_solo(
        &self,
        trackid: *mut MediaTrack,
        solo: bool,
        ignoresurf: *mut IReaperControlSurface,
    ) {
        unsafe {
            self.low.CSurf_SetSurfaceSolo(trackid, solo, ignoresurf);
        }
    }

    /// Generates a random GUID.
    pub fn gen_guid(&self) -> GUID {
        let mut guid = MaybeUninit::uninit();
        unsafe {
            self.low.genGuid(guid.as_mut_ptr());
        }
        unsafe { guid.assume_init() }
    }

    // TODO Doc
    // This method is not idempotent. If you call it two times, you will have every callback TWICE.
    // Please take care of unregistering once you are done!
    pub fn register_control_surface(&self) -> Result<(), ()> {
        self.plugin_register_csurf_inst(get_cpp_control_surface())
    }

    // TODO Doc
    // This method is idempotent
    pub fn unregister_control_surface(&self) {
        self.plugin_unregister_csurf_inst(get_cpp_control_surface());
    }

    // TODO Doc
    pub fn section_from_unique_id(&self, unique_id: u32) -> *mut KbdSectionInfo {
        unsafe { self.low.SectionFromUniqueID(unique_id as i32) }
    }

    // TODO Doc
    // Kept return value type i32 because I have no idea what the return value is about.
    pub fn kbd_on_main_action_ex(
        &self,
        cmd: u32,
        value: KbdActionValue,
        hwnd: HWND,
        proj: *mut ReaProject,
    ) -> i32 {
        use KbdActionValue::*;
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
        unsafe {
            self.low
                .KBD_OnMainActionEx(cmd as i32, val, valhw, relmode, hwnd, proj)
        }
    }

    /// Returns the REAPER main window handle.
    pub fn get_main_hwnd(&self) -> HWND {
        unsafe { self.low.GetMainHwnd() }
    }

    // TODO Doc
    pub fn named_command_lookup<'a>(&self, command_name: impl Into<ReaperStringArg<'a>>) -> u32 {
        unsafe { self.low.NamedCommandLookup(command_name.into().as_ptr()) as u32 }
    }

    /// Clears the ReaScript console.
    pub fn clear_console(&self) {
        unsafe {
            self.low.ClearConsole();
        }
    }

    /// Returns the number of tracks in the given project (pass `null_mut()` for current project)
    pub fn count_tracks(&self, proj: *mut ReaProject) -> u32 {
        unsafe { self.low.CountTracks(proj) as u32 }
    }

    // TODO Doc
    pub fn insert_track_at_index(&self, idx: u32, want_defaults: bool) {
        unsafe {
            self.low.InsertTrackAtIndex(idx as i32, want_defaults);
        }
    }

    // TODO Doc
    pub fn get_midi_input(&self, idx: u32) -> *mut midi_Input {
        unsafe { self.low.GetMidiInput(idx as i32) }
    }

    // TODO Doc
    pub fn get_midi_output(&self, idx: u32) -> *mut midi_Output {
        unsafe { self.low.GetMidiOutput(idx as i32) }
    }

    // TODO Doc
    pub fn get_max_midi_inputs(&self) -> u32 {
        unsafe { self.low.GetMaxMidiInputs() as u32 }
    }

    // TODO Doc
    pub fn get_max_midi_outputs(&self) -> u32 {
        unsafe { self.low.GetMaxMidiOutputs() as u32 }
    }

    // TODO Doc
    pub fn get_midi_input_name(&self, dev: u32, nameout_sz: u32) -> (bool, Option<CString>) {
        if nameout_sz == 0 {
            let is_present = unsafe { self.low.GetMIDIInputName(dev as i32, null_mut(), 0) };
            (is_present, None)
        } else {
            let (name, is_present) = with_string_buffer(nameout_sz, |buffer, max_size| unsafe {
                self.low.GetMIDIInputName(dev as i32, buffer, max_size)
            });
            if name.to_bytes().len() == 0 {
                return (is_present, None);
            }
            (is_present, Some(name))
        }
    }

    // TODO Doc
    // Return type Option or Result can't be easily chosen here because if instantiate is 0, it
    // should be Option, if it's -1 or > 0, it should be Result. So we just keep the i32.
    pub fn track_fx_add_by_name<'a>(
        &self,
        track: *mut MediaTrack,
        fxname: impl Into<ReaperStringArg<'a>>,
        rec_fx: bool,
        instantiate: TrackFxAddByNameVariant,
    ) -> i32 {
        unsafe {
            self.low
                .TrackFX_AddByName(track, fxname.into().as_ptr(), rec_fx, instantiate.into())
        }
    }

    // TODO Doc
    pub fn track_fx_add_by_name_query<'a>(
        &self,
        track: *mut MediaTrack,
        fxname: impl Into<ReaperStringArg<'a>>,
        rec_fx: bool,
    ) -> Option<u32> {
        match self.track_fx_add_by_name(track, fxname, rec_fx, TrackFxAddByNameVariant::Query) {
            -1 => None,
            idx if idx >= 0 => Some(idx as u32),
            _ => unreachable!(),
        }
    }

    // TODO Doc
    pub fn track_fx_add_by_name_add<'a>(
        &self,
        track: *mut MediaTrack,
        fxname: impl Into<ReaperStringArg<'a>>,
        rec_fx: bool,
        force_add: bool,
    ) -> Result<u32, ()> {
        match self.track_fx_add_by_name(
            track,
            fxname,
            rec_fx,
            if force_add {
                TrackFxAddByNameVariant::Add
            } else {
                TrackFxAddByNameVariant::AddIfNotFound
            },
        ) {
            -1 => Err(()),
            idx if idx >= 0 => Ok(idx as u32),
            _ => unreachable!(),
        }
    }

    // TODO Doc
    pub fn get_midi_output_name(&self, dev: u32, nameout_sz: u32) -> (bool, Option<CString>) {
        if nameout_sz == 0 {
            let is_present = unsafe { self.low.GetMIDIOutputName(dev as i32, null_mut(), 0) };
            (is_present, None)
        } else {
            let (name, is_present) = with_string_buffer(nameout_sz, |buffer, max_size| unsafe {
                self.low.GetMIDIOutputName(dev as i32, buffer, max_size)
            });
            if name.to_bytes().len() == 0 {
                return (is_present, None);
            }
            (is_present, Some(name))
        }
    }

    // TODO Doc
    pub fn track_fx_get_enabled(&self, track: *mut MediaTrack, fx: TrackFxRef) -> bool {
        unsafe { self.low.TrackFX_GetEnabled(track, fx.into()) }
    }

    // TODO Doc
    // Returns Err if FX doesn't exist
    pub fn track_fx_get_fx_name(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
        buf_sz: u32,
    ) -> Result<CString, ()> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| unsafe {
            self.low
                .TrackFX_GetFXName(track, fx.into(), buffer, max_size)
        });
        if !successful {
            return Err(());
        }
        Ok(name)
    }

    // TODO Doc
    pub fn track_fx_get_instrument(&self, track: *mut MediaTrack) -> Option<u32> {
        let index = unsafe { self.low.TrackFX_GetInstrument(track) };
        if index == -1 {
            return None;
        }
        Some(index as u32)
    }

    // TODO Doc
    pub fn track_fx_set_enabled(&self, track: *mut MediaTrack, fx: TrackFxRef, enabled: bool) {
        unsafe {
            self.low.TrackFX_SetEnabled(track, fx.into(), enabled);
        }
    }

    // TODO Doc
    pub fn track_fx_get_num_params(&self, track: *mut MediaTrack, fx: TrackFxRef) -> u32 {
        unsafe { self.low.TrackFX_GetNumParams(track, fx.into()) as u32 }
    }

    // TODO Doc
    pub fn get_current_project_in_load_save(&self) -> *mut ReaProject {
        unsafe { self.low.GetCurrentProjectInLoadSave() }
    }

    // TODO Doc
    // Returns Err if FX or parameter doesn't exist
    pub fn track_fx_get_param_name(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
        param: u32,
        buf_sz: u32,
    ) -> Result<CString, ()> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| unsafe {
            self.low
                .TrackFX_GetParamName(track, fx.into(), param as i32, buffer, max_size)
        });
        if !successful {
            return Err(());
        }
        Ok(name)
    }

    // TODO Doc
    // Returns Err if FX or parameter doesn't exist
    pub fn track_fx_get_formatted_param_value(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
        param: u32,
        buf_sz: u32,
    ) -> Result<CString, ()> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| unsafe {
            self.low.TrackFX_GetFormattedParamValue(
                track,
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

    // TODO Doc
    // Returns Err if FX or parameter doesn't exist or if FX doesn't support formatting arbitrary
    // parameter values and the given value is not equal to the current one.
    pub fn track_fx_format_param_value_normalized(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
        param: u32,
        value: f64,
        buf_sz: u32,
    ) -> Result<CString, ()> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| unsafe {
            self.low.TrackFX_FormatParamValueNormalized(
                track,
                fx.into(),
                param as i32,
                value,
                buffer,
                max_size,
            )
        });
        if !successful {
            return Err(());
        }
        Ok(name)
    }

    // TODO Doc
    // Returns Err if FX or parameter doesn't exist
    pub fn track_fx_set_param_normalized(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
        param: u32,
        value: f64,
    ) -> Result<(), ()> {
        let successful = unsafe {
            self.low
                .TrackFX_SetParamNormalized(track, fx.into(), param as i32, value)
        };
        if !successful {
            return Err(());
        }
        Ok(())
    }

    // TODO Doc
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

    // TODO Doc
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

    // TODO Doc
    pub fn track_fx_copy_to_track(
        &self,
        src_track: *mut MediaTrack,
        src_fx: TrackFxRef,
        dest_track: *mut MediaTrack,
        dest_fx: TrackFxRef,
        is_move: bool,
    ) {
        unsafe {
            self.low.TrackFX_CopyToTrack(
                src_track,
                src_fx.into(),
                dest_track,
                dest_fx.into(),
                is_move,
            );
        }
    }

    // TODO Doc
    // Returns Err if FX doesn't exist (maybe also in other cases?)
    pub fn track_fx_delete(&self, track: *mut MediaTrack, fx: TrackFxRef) -> Result<(), ()> {
        let succesful = unsafe { self.low.TrackFX_Delete(track, fx.into()) };
        if !succesful {
            return Err(());
        }
        Ok(())
    }

    // TODO Doc
    // Returns None if the FX parameter doesn't report step sizes (or if the FX or parameter doesn't
    // exist, but that can be checked before). Option makes more sense than Result here because
    // this function is at the same time the correct function to be used to determine *if* a
    // parameter reports step sizes. So "parameter doesn't report step sizes" is a valid result.
    pub fn track_fx_get_parameter_step_sizes(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
        param: u32,
    ) -> Option<GetParameterStepSizesResult> {
        let mut step = MaybeUninit::uninit();
        let mut small_step = MaybeUninit::uninit();
        let mut large_step = MaybeUninit::uninit();
        let mut is_toggle = MaybeUninit::uninit();
        let successful = unsafe {
            self.low.TrackFX_GetParameterStepSizes(
                track,
                fx.into(),
                param as i32,
                step.as_mut_ptr(),
                small_step.as_mut_ptr(),
                large_step.as_mut_ptr(),
                is_toggle.as_mut_ptr(),
            )
        };
        if !successful {
            return None;
        }
        Some(GetParameterStepSizesResult {
            step: make_some_if_greater_than_zero(unsafe { step.assume_init() }),
            small_step: make_some_if_greater_than_zero(unsafe { small_step.assume_init() }),
            large_step: make_some_if_greater_than_zero(unsafe { large_step.assume_init() }),
            is_toggle: unsafe { is_toggle.assume_init() },
        })
    }

    // TODO Doc
    pub fn track_fx_get_param_ex(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
        param: u32,
    ) -> GetParamExResult {
        let mut min_val = MaybeUninit::uninit();
        let mut max_val = MaybeUninit::uninit();
        let mut mid_val = MaybeUninit::uninit();
        let value = unsafe {
            self.low.TrackFX_GetParamEx(
                track,
                fx.into(),
                param as i32,
                min_val.as_mut_ptr(),
                max_val.as_mut_ptr(),
                mid_val.as_mut_ptr(),
            )
        };
        GetParamExResult {
            value: value,
            min_val: unsafe { min_val.assume_init() },
            mid_val: unsafe { mid_val.assume_init() },
            max_val: unsafe { max_val.assume_init() },
        }
        .into()
    }

    // TODO Doc
    pub fn undo_begin_block_2(&self, proj: *mut ReaProject) {
        unsafe {
            self.low.Undo_BeginBlock2(proj);
        }
    }

    // TODO Doc
    pub fn undo_end_block_2<'a>(
        &self,
        proj: *mut ReaProject,
        descchange: impl Into<ReaperStringArg<'a>>,
        extraflags: Option<BitFlags<UndoFlag>>,
    ) {
        unsafe {
            self.low.Undo_EndBlock2(
                proj,
                descchange.into().as_ptr(),
                match extraflags {
                    Some(flags) => flags.bits(),
                    None => UNDO_STATE_ALL,
                } as i32,
            );
        }
    }

    // TODO Doc
    pub fn undo_can_undo_2<R>(&self, proj: *mut ReaProject, f: impl Fn(&CStr) -> R) -> Option<R> {
        let ptr = unsafe { self.low.Undo_CanUndo2(proj) };
        unsafe { create_passing_c_str(ptr) }.map(f)
    }

    // TODO Doc
    pub fn undo_can_redo_2<R>(&self, proj: *mut ReaProject, f: impl Fn(&CStr) -> R) -> Option<R> {
        let ptr = unsafe { self.low.Undo_CanRedo2(proj) };
        unsafe { create_passing_c_str(ptr) }.map(f)
    }

    // TODO Doc
    // Returns true if there was something to be undone, false if not
    pub fn undo_do_undo_2(&self, proj: *mut ReaProject) -> bool {
        unsafe { self.low.Undo_DoUndo2(proj) != 0 }
    }

    // TODO Doc
    // Returns true if there was something to be redone, false if not
    pub fn undo_do_redo_2(&self, proj: *mut ReaProject) -> bool {
        unsafe { self.low.Undo_DoRedo2(proj) != 0 }
    }

    // TODO Doc
    pub fn mark_project_dirty(&self, proj: *mut ReaProject) {
        unsafe {
            self.low.MarkProjectDirty(proj);
        }
    }

    // TODO Doc
    // Returns true if project dirty, false if not
    pub fn is_project_dirty(&self, proj: *mut ReaProject) -> bool {
        unsafe { self.low.IsProjectDirty(proj) != 0 }
    }

    // TODO Doc
    pub fn track_list_update_all_external_surfaces(&self) {
        unsafe {
            self.low.TrackList_UpdateAllExternalSurfaces();
        }
    }

    // TODO Doc
    pub fn get_app_version(&self) -> ReaperVersion {
        let ptr = unsafe { self.low.GetAppVersion() };
        let version_str = unsafe { CStr::from_ptr(ptr) };
        version_str.into()
    }

    // TODO Doc
    pub fn get_track_automation_mode(&self, tr: *mut MediaTrack) -> AutomationMode {
        let result = unsafe { self.low.GetTrackAutomationMode(tr) as u32 };
        AutomationMode::try_from(result).expect("Unknown automation mode")
    }

    // TODO Doc
    pub fn get_global_automation_override(&self) -> Option<GlobalAutomationOverride> {
        use GlobalAutomationOverride::*;
        match unsafe { self.low.GetGlobalAutomationOverride() } {
            -1 => None,
            6 => Some(Bypass),
            x => Some(Mode(
                AutomationMode::try_from(x as u32).expect("Unknown automation mode"),
            )),
        }
    }

    // TODO Doc
    pub fn get_track_envelope_by_name<'a>(
        &self,
        track: *mut MediaTrack,
        envname: impl Into<ReaperStringArg<'a>>,
    ) -> *mut TrackEnvelope {
        unsafe {
            self.low
                .GetTrackEnvelopeByName(track, envname.into().as_ptr())
        }
    }

    // TODO Doc
    pub fn get_media_track_info_value(&self, tr: *mut MediaTrack, parmname: TrackInfoKey) -> f64 {
        unsafe {
            self.low
                .GetMediaTrackInfo_Value(tr, Cow::from(parmname).as_ptr())
        }
    }

    // TODO Doc
    pub fn track_fx_get_count(&self, track: *mut MediaTrack) -> u32 {
        unsafe { self.low.TrackFX_GetCount(track) as u32 }
    }

    // TODO Doc
    pub fn track_fx_get_rec_count(&self, track: *mut MediaTrack) -> u32 {
        unsafe { self.low.TrackFX_GetRecCount(track) as u32 }
    }

    // TODO Doc
    pub fn track_fx_get_fx_guid(&self, track: *mut MediaTrack, fx: TrackFxRef) -> Option<GUID> {
        let ptr = unsafe { self.low.TrackFX_GetFXGUID(track, fx.into()) };
        unsafe { copy_ptr_target(ptr) }
    }

    // TODO Doc
    pub fn track_fx_get_param_normalized(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
        param: u32,
    ) -> f64 {
        unsafe {
            self.low
                .TrackFX_GetParamNormalized(track, fx.into(), param as i32)
        }
    }

    // TODO Doc
    pub fn get_master_track(&self, proj: *mut ReaProject) -> *mut MediaTrack {
        unsafe { self.low.GetMasterTrack(proj) }
    }

    // TODO Doc
    pub fn guid_to_string(&self, g: &GUID) -> CString {
        let (guid_string, _) = with_string_buffer(64, |buffer, _| unsafe {
            self.low.guidToString(g as *const GUID, buffer)
        });
        guid_string
    }

    // TODO Doc
    pub fn master_get_tempo(&self) -> f64 {
        unsafe { self.low.Master_GetTempo() }
    }

    // TODO Doc
    pub fn set_current_bpm(&self, __proj: *mut ReaProject, bpm: f64, want_undo: bool) {
        unsafe {
            self.low.SetCurrentBPM(__proj, bpm, want_undo);
        }
    }

    // TODO Doc
    pub fn master_get_play_rate(&self, project: *mut ReaProject) -> f64 {
        unsafe { self.low.Master_GetPlayRate(project) }
    }

    // TODO Doc
    pub fn csurf_on_play_rate_change(&self, playrate: f64) {
        unsafe {
            self.low.CSurf_OnPlayRateChange(playrate);
        }
    }

    // TODO Doc
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

    // TODO Doc
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

    // TODO Doc
    pub fn csurf_on_input_monitoring_change_ex(
        &self,
        trackid: *mut MediaTrack,
        monitor: InputMonitoringMode,
        allowgang: bool,
    ) -> i32 {
        unsafe {
            self.low
                .CSurf_OnInputMonitorChangeEx(trackid, monitor.into(), allowgang)
        }
    }

    // TODO Doc
    // Returns Err if invalid parameter name given (maybe also in other situations)
    pub fn set_media_track_info_value(
        &self,
        tr: *mut MediaTrack,
        parmname: TrackInfoKey,
        newvalue: f64,
    ) -> Result<(), ()> {
        let successful = unsafe {
            self.low
                .SetMediaTrackInfo_Value(tr, Cow::from(parmname).as_ptr(), newvalue)
        };
        if !successful {
            return Err(());
        }
        Ok(())
    }

    // TODO Doc
    pub fn stuff_midimessage(&self, mode: StuffMidiMessageTarget, msg: impl MidiMessage) {
        unsafe {
            self.low.StuffMIDIMessage(
                mode.into(),
                msg.get_status_byte().into(),
                msg.get_data_byte_1().into(),
                msg.get_data_byte_2().into(),
            );
        }
    }

    // TODO Doc
    pub fn db2slider(&self, x: f64) -> f64 {
        unsafe { self.low.DB2SLIDER(x) }
    }

    // TODO Doc
    pub fn slider2db(&self, y: f64) -> f64 {
        unsafe { self.low.SLIDER2DB(y) }
    }

    // TODO Doc
    // I guess it returns Err if the track doesn't exist
    pub fn get_track_ui_vol_pan(&self, track: *mut MediaTrack) -> Result<(f64, f64), ()> {
        let mut volume = MaybeUninit::uninit();
        let mut pan = MaybeUninit::uninit();
        let successful = unsafe {
            self.low
                .GetTrackUIVolPan(track, volume.as_mut_ptr(), pan.as_mut_ptr())
        };
        if !successful {
            return Err(());
        }
        Ok((unsafe { volume.assume_init() }, unsafe {
            pan.assume_init()
        }))
    }

    // TODO Doc
    // Returns true on success
    pub fn audio_reg_hardware_hook(&self, is_add: bool, reg: *mut audio_hook_register_t) -> bool {
        unsafe { self.low.Audio_RegHardwareHook(is_add, reg) > 0 }
    }

    // TODO Doc
    pub fn csurf_set_surface_volume(
        &self,
        trackid: *mut MediaTrack,
        volume: f64,
        ignoresurf: *mut IReaperControlSurface,
    ) {
        unsafe {
            self.low.CSurf_SetSurfaceVolume(trackid, volume, ignoresurf);
        }
    }

    // TODO Doc
    pub fn csurf_on_volume_change_ex(
        &self,
        trackid: *mut MediaTrack,
        volume: f64,
        relative: bool,
        allow_gang: bool,
    ) -> f64 {
        unsafe {
            self.low
                .CSurf_OnVolumeChangeEx(trackid, volume, relative, allow_gang)
        }
    }

    // TODO Doc
    pub fn csurf_set_surface_pan(
        &self,
        trackid: *mut MediaTrack,
        pan: f64,
        ignoresurf: *mut IReaperControlSurface,
    ) {
        unsafe {
            self.low.CSurf_SetSurfacePan(trackid, pan, ignoresurf);
        }
    }

    // TODO Doc
    pub fn csurf_on_pan_change_ex(
        &self,
        trackid: *mut MediaTrack,
        pan: f64,
        relative: bool,
        allow_gang: bool,
    ) -> f64 {
        unsafe {
            self.low
                .CSurf_OnPanChangeEx(trackid, pan, relative, allow_gang)
        }
    }

    // TODO Doc
    pub fn count_selected_tracks_2(&self, proj: *mut ReaProject, wantmaster: bool) -> u32 {
        unsafe { self.low.CountSelectedTracks2(proj, wantmaster) as u32 }
    }

    // TODO Doc
    pub fn set_track_selected(&self, track: *mut MediaTrack, selected: bool) {
        unsafe {
            self.low.SetTrackSelected(track, selected);
        }
    }

    // TODO Doc
    pub fn get_selected_track_2(
        &self,
        proj: *mut ReaProject,
        seltrackidx: u32,
        wantmaster: bool,
    ) -> *mut MediaTrack {
        unsafe {
            self.low
                .GetSelectedTrack2(proj, seltrackidx as i32, wantmaster)
        }
    }

    // TODO Doc
    pub fn set_only_track_selected(&self, track: *mut MediaTrack) {
        unsafe {
            self.low.SetOnlyTrackSelected(track);
        }
    }

    // TODO Doc
    pub fn delete_track(&self, tr: *mut MediaTrack) {
        unsafe {
            self.low.DeleteTrack(tr);
        }
    }

    // TODO Doc
    pub fn get_track_num_sends(&self, tr: *mut MediaTrack, category: TrackSendCategory) -> u32 {
        unsafe { self.low.GetTrackNumSends(tr, category.into()) as u32 }
    }

    // TODO Doc
    pub unsafe fn get_set_track_send_info(
        &self,
        tr: *mut MediaTrack,
        category: TrackSendCategory,
        sendidx: u32,
        parmname: TrackSendInfoKey,
        set_new_value: *mut c_void,
    ) -> *mut c_void {
        self.low.GetSetTrackSendInfo(
            tr,
            category.into(),
            sendidx as i32,
            Cow::from(parmname).as_ptr(),
            set_new_value,
        )
    }

    fn get_track_send_info(
        &self,
        tr: *mut MediaTrack,
        category: TrackSendCategory,
        sendidx: u32,
        parmname: TrackSendInfoKey,
    ) -> *mut c_void {
        unsafe { self.get_set_track_send_info(tr, category, sendidx, parmname, null_mut()) }
    }

    // TODO Doc
    pub fn get_track_send_info_desttrack(
        &self,
        tr: *mut MediaTrack,
        category: SendOrReceive,
        sendidx: u32,
    ) -> *mut MediaTrack {
        self.get_track_send_info(tr, category.into(), sendidx, TrackSendInfoKey::P_DESTTRACK)
            as *mut MediaTrack
    }

    // TODO Doc
    // I guess it returns Err if the track doesn't exist
    pub fn get_track_state_chunk(
        &self,
        track: *mut MediaTrack,
        str_need_big_sz: u32,
        isundo_optional: bool,
    ) -> Result<CString, ()> {
        let (chunk_content, successful) =
            with_string_buffer(str_need_big_sz, |buffer, max_size| unsafe {
                self.low
                    .GetTrackStateChunk(track, buffer, max_size, isundo_optional)
            });
        if !successful {
            return Err(());
        }
        Ok(chunk_content)
    }

    // TODO Doc
    pub fn create_track_send(
        &self,
        tr: *mut MediaTrack,
        desttr_in_optional: *mut MediaTrack,
    ) -> u32 {
        unsafe { self.low.CreateTrackSend(tr, desttr_in_optional) as u32 }
    }

    // TODO Doc
    // TODO Check other bools in this API and consider using an enum
    // Seems to return true if was armed and false if not
    pub fn csurf_on_rec_arm_change_ex(
        &self,
        trackid: *mut MediaTrack,
        recarm: RecArmState,
        allowgang: Gang,
    ) -> bool {
        unsafe {
            self.low
                .CSurf_OnRecArmChangeEx(trackid, recarm.into(), allowgang == Gang::Allow)
        }
    }

    // TODO Doc
    // Returns Err for example if given chunk is invalid
    pub fn set_track_state_chunk<'a>(
        &self,
        track: *mut MediaTrack,
        str: impl Into<ReaperStringArg<'a>>,
        isundo_optional: bool,
    ) -> Result<(), ()> {
        let successful = unsafe {
            self.low
                .SetTrackStateChunk(track, str.into().as_ptr(), isundo_optional)
        };
        if !successful {
            return Err(());
        }
        Ok(())
    }

    // TODO Doc
    pub fn track_fx_show(&self, track: *mut MediaTrack, index: TrackFxRef, show_flag: FxShowFlag) {
        unsafe {
            self.low.TrackFX_Show(track, index.into(), show_flag.into());
        }
    }

    // TODO Doc
    pub fn track_fx_get_floating_window(&self, track: *mut MediaTrack, index: TrackFxRef) -> HWND {
        unsafe { self.low.TrackFX_GetFloatingWindow(track, index.into()) }
    }

    // TODO Doc
    pub fn track_fx_get_open(&self, track: *mut MediaTrack, fx: TrackFxRef) -> bool {
        unsafe { self.low.TrackFX_GetOpen(track, fx.into()) }
    }

    // TODO Doc
    pub fn csurf_on_send_volume_change(
        &self,
        trackid: *mut MediaTrack,
        send_index: u32,
        volume: f64,
        relative: bool,
    ) -> f64 {
        unsafe {
            self.low
                .CSurf_OnSendVolumeChange(trackid, send_index as i32, volume, relative)
        }
    }

    // TODO Doc
    pub fn csurf_on_send_pan_change(
        &self,
        trackid: *mut MediaTrack,
        send_index: u32,
        pan: f64,
        relative: bool,
    ) -> f64 {
        unsafe {
            self.low
                .CSurf_OnSendPanChange(trackid, send_index as i32, pan, relative)
        }
    }

    // TODO Doc
    // Returns None if section or command not existing (can't think of any other case)
    pub fn kbd_get_text_from_cmd<R>(
        &self,
        cmd: u32,
        section: *mut KbdSectionInfo,
        f: impl Fn(&CStr) -> R,
    ) -> Option<R> {
        let ptr = unsafe { self.low.kbd_getTextFromCmd(cmd, section) };
        unsafe { create_passing_c_str(ptr) }
            // Removed action returns empty string for some reason. We want None in this case!
            .filter(|s| s.to_bytes().len() > 0)
            .map(f)
    }

    // TODO Doc
    // Returns None if action doesn't report on/off states (or if action not existing).
    // Option makes more sense than Result here because this function is at the same time the
    // correct function to be used to determine *if* an action reports on/off states. So
    // "action doesn't report on/off states" is a valid result.
    pub fn get_toggle_command_state_2(
        &self,
        section: *mut KbdSectionInfo,
        command_id: u32,
    ) -> Option<bool> {
        let result = unsafe { self.low.GetToggleCommandState2(section, command_id as i32) };
        if result == -1 {
            return None;
        }
        return Some(result != 0);
    }

    // TODO Doc
    // Returns None if lookup was not successful, that is, the command couldn't be found
    pub fn reverse_named_command_lookup<R>(
        &self,
        command_id: u32,
        f: impl Fn(&CStr) -> R,
    ) -> Option<R> {
        let ptr = unsafe { self.low.ReverseNamedCommandLookup(command_id as i32) };
        unsafe { create_passing_c_str(ptr) }.map(f)
    }

    // TODO Doc
    // Returns Err if send not existing
    pub fn get_track_send_ui_vol_pan(
        &self,
        track: *mut MediaTrack,
        send_index: u32,
    ) -> Result<(f64, f64), ()> {
        let mut volume = MaybeUninit::uninit();
        let mut pan = MaybeUninit::uninit();
        let successful = unsafe {
            self.low.GetTrackSendUIVolPan(
                track,
                send_index as i32,
                volume.as_mut_ptr(),
                pan.as_mut_ptr(),
            )
        };
        if !successful {
            return Err(());
        }
        Ok((unsafe { volume.assume_init() }, unsafe {
            pan.assume_init()
        }))
    }

    // TODO Doc
    // Returns Err e.g. if FX doesn't exist
    pub fn track_fx_get_preset_index(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
    ) -> Result<(u32, u32), ()> {
        let mut num_presets = MaybeUninit::uninit();
        let index = unsafe {
            self.low
                .TrackFX_GetPresetIndex(track, fx.into(), num_presets.as_mut_ptr())
        };
        if index == -1 {
            return Err(());
        }
        return Ok((index as u32, unsafe { num_presets.assume_init() as u32 }));
    }

    // TODO Doc
    // Returns Err e.g. if FX doesn't exist
    pub fn track_fx_set_preset_by_index(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
        idx: i32,
    ) -> Result<(), ()> {
        let successful = unsafe { self.low.TrackFX_SetPresetByIndex(track, fx.into(), idx) };
        if !successful {
            return Err(());
        }
        Ok(())
    }

    // TODO Doc
    // Returns Err e.g. if FX doesn't exist
    pub fn track_fx_navigate_presets(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
        presetmove: i32,
    ) -> Result<(), ()> {
        let successful = unsafe {
            self.low
                .TrackFX_NavigatePresets(track, fx.into(), presetmove)
        };
        if !successful {
            return Err(());
        }
        Ok(())
    }

    // TODO Doc
    pub fn track_fx_get_preset(
        &self,
        track: *mut MediaTrack,
        fx: TrackFxRef,
        presetname_sz: u32,
    ) -> (bool, Option<CString>) {
        if presetname_sz == 0 {
            let state_matches_preset =
                unsafe { self.low.TrackFX_GetPreset(track, fx.into(), null_mut(), 0) };
            (state_matches_preset, None)
        } else {
            let (name, state_matches_preset) =
                with_string_buffer(presetname_sz, |buffer, max_size| unsafe {
                    self.low
                        .TrackFX_GetPreset(track, fx.into(), buffer, max_size)
                });
            if name.to_bytes().len() == 0 {
                return (state_matches_preset, None);
            }
            (state_matches_preset, Some(name))
        }
    }
}

// Each of the decimal numbers are > 0
pub struct GetParameterStepSizesResult {
    // TODO-low Not sure if this can ever be 0 when TrackFX_GetParameterStepSizes returns true
    //  So the Option might be obsolete here
    pub step: Option<f64>,
    pub small_step: Option<f64>,
    pub large_step: Option<f64>,
    pub is_toggle: bool,
}

// Each of the attributes can be negative! These are not normalized values (0..1).
pub struct GetParamExResult {
    pub value: f64,
    pub min_val: f64,
    pub mid_val: f64,
    pub max_val: f64,
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

unsafe fn copy_ptr_target<T: Copy>(ptr: *const T) -> Option<T> {
    if ptr.is_null() {
        return None;
    }
    Some(*ptr)
}

unsafe fn copy_void_ptr_target<T: Copy>(ptr: *mut c_void) -> Option<T> {
    copy_ptr_target(ptr as *const T)
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
        TrackRef::TrackIndex(tracknumber - 1)
    }
}

fn ok_if_one(result: i32) -> Result<(), ()> {
    if result == 1 { Ok(()) } else { Err(()) }
}
