//! Provides all functions from `reaper_plugin_functions.h` with the following improvements:
//! - Snake-case function and parameter names
//! - Return values instead of output parameters
//! - No C strings
//! - Panics if function not available (we should make sure on plug-in load that all necessary
//!   functions are available, maybe provide "_available" functions for conditional execution)
//! - Use u32 instead of i32 when applicable
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr::{null, null_mut};

use c_str_macro::c_str;

use crate::low_level;
use crate::low_level::{
    audio_hook_register_t, midi_Input, midi_Output, IReaperControlSurface, KbdSectionInfo,
    MediaTrack, ReaProject, TrackEnvelope, GUID, HWND,
};
pub use crate::medium_level::control_surface::ControlSurface;
use crate::medium_level::control_surface::DelegatingControlSurface;

mod control_surface;

pub struct Reaper {
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

macro_rules! require {
    ($low:expr, $func:ident) => {{
        match $low.$func {
            None => panic!(format!(
                "Couldn't load REAPER function {}",
                stringify!($func)
            )),
            Some(f) => f,
        }
    }};
}

impl Reaper {
    pub fn new(low: low_level::Reaper) -> Reaper {
        Reaper { low }
    }

    pub fn enum_projects(
        &self,
        idx: i32,
        projfn_out_optional_sz: u32,
    ) -> (*mut ReaProject, Option<CString>) {
        if projfn_out_optional_sz == 0 {
            let project = require!(self.low, EnumProjects)(idx, null_mut(), 0);
            (project, None)
        } else {
            let (file_path, project) =
                with_string_buffer(projfn_out_optional_sz, |buffer, max_size| {
                    require!(self.low, EnumProjects)(idx, buffer, max_size)
                });

            (
                project,
                if file_path.as_bytes().len() == 0 {
                    None
                } else {
                    Some(file_path)
                },
            )
        }
    }

    pub fn get_track(&self, proj: *mut ReaProject, trackidx: u32) -> *mut MediaTrack {
        require!(self.low, GetTrack)(proj, trackidx as i32)
    }

    pub fn validate_ptr_2(
        &self,
        proj: *mut ReaProject,
        pointer: *mut c_void,
        ctypename: &CStr,
    ) -> bool {
        require!(self.low, ValidatePtr2)(proj, pointer, ctypename.as_ptr())
    }

    pub fn get_set_media_track_info(
        &self,
        tr: *mut MediaTrack,
        parmname: &CStr,
        set_new_value: *mut c_void,
    ) -> *mut c_void {
        require!(self.low, GetSetMediaTrackInfo)(tr, parmname.as_ptr(), set_new_value)
    }

    pub fn show_console_msg(&self, msg: &CStr) {
        require!(self.low, ShowConsoleMsg)(msg.as_ptr())
    }

    pub fn plugin_register(&self, name: &CStr, infostruct: *mut c_void) -> i32 {
        require!(self.low, plugin_register)(name.as_ptr(), infostruct)
    }

    pub fn main_on_command_ex(&self, command: u32, flag: i32, proj: *mut ReaProject) {
        require!(self.low, Main_OnCommandEx)(command as i32, flag, proj);
    }

    pub fn csurf_set_surface_mute(
        &self,
        trackid: *mut MediaTrack,
        mute: bool,
        ignoresurf: *mut IReaperControlSurface,
    ) {
        require!(self.low, CSurf_SetSurfaceMute)(trackid, mute, ignoresurf);
    }

    pub fn csurf_set_surface_solo(
        &self,
        trackid: *mut MediaTrack,
        solo: bool,
        ignoresurf: *mut IReaperControlSurface,
    ) {
        require!(self.low, CSurf_SetSurfaceSolo)(trackid, solo, ignoresurf);
    }

    pub fn gen_guid(&self) -> GUID {
        let mut guid = ZERO_GUID;
        require!(self.low, genGuid)(&mut guid as *mut GUID);
        guid
    }

    // Once installed, it stays installed until this module unloaded
    pub fn install_control_surface(&self, control_surface: impl ControlSurface + 'static) {
        let delegating_control_surface = DelegatingControlSurface::new(control_surface);
        self.low.install_control_surface(delegating_control_surface);
    }

    // Must be idempotent
    // Please take care of unregistering once you are done!
    pub fn register_control_surface(&self) {
        self.plugin_register(c_str!("csurf_inst"), self.low.get_cpp_control_surface());
    }

    // Must be idempotent
    pub fn unregister_control_surface(&self) {
        self.plugin_register(c_str!("-csurf_inst"), self.low.get_cpp_control_surface());
    }

    pub fn section_from_unique_id(&self, unique_id: u32) -> *mut KbdSectionInfo {
        require!(self.low, SectionFromUniqueID)(unique_id as i32)
    }

    pub fn kbd_on_main_action_ex(
        &self,
        cmd: u32,
        val: u32,
        valhw: i32,
        relmode: u32,
        hwnd: HWND,
        proj: *mut ReaProject,
    ) -> i32 {
        require!(self.low, KBD_OnMainActionEx)(
            cmd as i32,
            val as i32,
            valhw,
            relmode as i32,
            hwnd,
            proj,
        )
    }

    pub fn get_main_hwnd(&self) -> HWND {
        require!(self.low, GetMainHwnd)()
    }

    pub fn named_command_lookup(&self, command_name: &CStr) -> u32 {
        require!(self.low, NamedCommandLookup)(command_name.as_ptr()) as u32
    }

    pub fn clear_console(&self) {
        require!(self.low, ClearConsole)();
    }

    pub fn count_tracks(&self, proj: *mut ReaProject) -> u32 {
        require!(self.low, CountTracks)(proj) as u32
    }

    pub fn insert_track_at_index(&self, idx: u32, want_defaults: bool) {
        require!(self.low, InsertTrackAtIndex)(idx as i32, want_defaults);
    }

    pub fn get_midi_input(&self, idx: u32) -> *mut midi_Input {
        require!(self.low, GetMidiInput)(idx as i32)
    }

    pub fn get_midi_output(&self, idx: u32) -> *mut midi_Output {
        require!(self.low, GetMidiOutput)(idx as i32)
    }

    pub fn get_max_midi_inputs(&self) -> u32 {
        require!(self.low, GetMaxMidiInputs)() as u32
    }

    pub fn get_max_midi_outputs(&self) -> u32 {
        require!(self.low, GetMaxMidiOutputs)() as u32
    }

    pub fn get_midi_input_name(&self, dev: u32, nameout_sz: u32) -> (bool, Option<CString>) {
        if nameout_sz == 0 {
            let is_present = require!(self.low, GetMIDIInputName)(dev as i32, null_mut(), 0);
            (is_present, None)
        } else {
            let (name, is_present) = with_string_buffer(nameout_sz, |buffer, max_size| {
                require!(self.low, GetMIDIInputName)(dev as i32, buffer, max_size)
            });
            (
                is_present,
                if name.as_bytes().len() == 0 {
                    None
                } else {
                    Some(name)
                },
            )
        }
    }

    pub fn track_fx_add_by_name(
        &self,
        track: *mut MediaTrack,
        fxname: &CStr,
        rec_fx: bool,
        instantiate: i32,
    ) -> i32 {
        require!(self.low, TrackFX_AddByName)(track, fxname.as_ptr(), rec_fx, instantiate)
    }

    pub fn get_midi_output_name(&self, dev: u32, nameout_sz: u32) -> (bool, Option<CString>) {
        if nameout_sz == 0 {
            let is_present = require!(self.low, GetMIDIOutputName)(dev as i32, null_mut(), 0);
            (is_present, None)
        } else {
            let (name, is_present) = with_string_buffer(nameout_sz, |buffer, max_size| {
                require!(self.low, GetMIDIOutputName)(dev as i32, buffer, max_size)
            });
            (
                is_present,
                if name.as_bytes().len() == 0 {
                    None
                } else {
                    Some(name)
                },
            )
        }
    }

    pub fn track_fx_get_enabled(&self, track: *mut MediaTrack, fx: u32) -> bool {
        require!(self.low, TrackFX_GetEnabled)(track, fx as i32)
    }

    pub fn track_fx_get_fx_name(
        &self,
        track: *mut MediaTrack,
        fx: u32,
        buf_sz: u32,
    ) -> Option<CString> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            require!(self.low, TrackFX_GetFXName)(track, fx as i32, buffer, max_size)
        });
        if !successful || name.as_bytes().len() == 0 {
            return None;
        }
        Some(name)
    }

    // TODO-high Return None if result is -1
    pub fn track_fx_get_instrument(&self, track: *mut MediaTrack) -> i32 {
        require!(self.low, TrackFX_GetInstrument)(track)
    }

    pub fn track_fx_set_enabled(&self, track: *mut MediaTrack, fx: u32, enabled: bool) {
        require!(self.low, TrackFX_SetEnabled)(track, fx as i32, enabled);
    }

    pub fn track_fx_get_num_params(&self, track: *mut MediaTrack, fx: u32) -> u32 {
        require!(self.low, TrackFX_GetNumParams)(track, fx as i32) as u32
    }

    pub fn get_current_project_in_load_save(&self) -> *mut ReaProject {
        require!(self.low, GetCurrentProjectInLoadSave)()
    }

    pub fn track_fx_get_param_name(
        &self,
        track: *mut MediaTrack,
        fx: u32,
        param: u32,
        buf_sz: u32,
    ) -> Option<CString> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            require!(self.low, TrackFX_GetParamName)(
                track,
                fx as i32,
                param as i32,
                buffer,
                max_size,
            )
        });
        if !successful || name.as_bytes().len() == 0 {
            return None;
        }
        Some(name)
    }

    pub fn track_fx_get_formatted_param_value(
        &self,
        track: *mut MediaTrack,
        fx: u32,
        param: u32,
        buf_sz: u32,
    ) -> Option<CString> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            require!(self.low, TrackFX_GetFormattedParamValue)(
                track,
                fx as i32,
                param as i32,
                buffer,
                max_size,
            )
        });
        if !successful || name.as_bytes().len() == 0 {
            return None;
        }
        Some(name)
    }

    pub fn track_fx_format_param_value_normalized(
        &self,
        track: *mut MediaTrack,
        fx: u32,
        param: u32,
        value: f64,
        buf_sz: u32,
    ) -> Option<CString> {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            require!(self.low, TrackFX_FormatParamValueNormalized)(
                track,
                fx as i32,
                param as i32,
                value,
                buffer,
                max_size,
            )
        });
        if !successful || name.as_bytes().len() == 0 {
            return None;
        }
        Some(name)
    }

    pub fn track_fx_set_param_normalized(
        &self,
        track: *mut MediaTrack,
        fx: u32,
        param: u32,
        value: f64,
    ) -> bool {
        require!(self.low, TrackFX_SetParamNormalized)(track, fx as i32, param as i32, value)
    }

    pub fn get_focused_fx(&self) -> GetFocusedFxResult {
        let mut tracknumber = -1;
        let mut itemnumber = -1;
        let mut fxnumber = -1;
        let result = require!(self.low, GetFocusedFX)(
            &mut tracknumber as *mut i32,
            &mut itemnumber as *mut i32,
            &mut fxnumber as *mut i32,
        );
        match result {
            0 => GetFocusedFxResult::None,
            1 => GetFocusedFxResult::TrackFx(GetFocusedFxTrackFxResultData {
                tracknumber,
                fxnumber,
            }),
            2 => GetFocusedFxResult::ItemFx(GetFocusedFxItemFxResultData {
                tracknumber,
                fxnumber,
                itemnumber,
            }),
            default => panic!("Unknown GetFocusedFX result value"),
        }
    }

    pub fn get_last_touched_fx(&self) -> Option<GetLastTouchedFxResultData> {
        let mut tracknumber = -1;
        let mut fxnumber = -1;
        let mut paramnumber = -1;
        let is_valid = require!(self.low, GetLastTouchedFX)(
            &mut tracknumber as *mut i32,
            &mut fxnumber as *mut i32,
            &mut paramnumber as *mut i32,
        );
        if !is_valid {
            return None;
        }
        Some(GetLastTouchedFxResultData {
            tracknumber,
            fxnumber,
            paramnumber,
        })
    }

    pub fn track_fx_copy_to_track(
        &self,
        src_track: *mut MediaTrack,
        src_fx: u32,
        dest_track: *mut MediaTrack,
        dest_fx: u32,
        is_move: bool,
    ) {
        require!(self.low, TrackFX_CopyToTrack)(
            src_track,
            src_fx as i32,
            dest_track,
            dest_fx as i32,
            is_move,
        );
    }

    pub fn track_fx_delete(&self, track: *mut MediaTrack, fx: u32) -> bool {
        require!(self.low, TrackFX_Delete)(track, fx as i32)
    }

    pub fn track_fx_get_parameter_step_sizes(
        &self,
        track: *mut MediaTrack,
        fx: u32,
        param: u32,
    ) -> Option<GetParameterStepSizesResult> {
        let mut step = -1.0;
        let mut small_step = -1.0;
        let mut large_step = -1.0;
        let mut is_toggle = false;
        let successful = require!(self.low, TrackFX_GetParameterStepSizes)(
            track,
            fx as i32,
            param as i32,
            &mut step as *mut f64,
            &mut small_step as *mut f64,
            &mut large_step as *mut f64,
            &mut is_toggle as *mut bool,
        );
        if !successful {
            return None;
        }
        GetParameterStepSizesResult {
            step: complain_if_minus_one(step),
            small_step: complain_if_minus_one(small_step),
            large_step: complain_if_minus_one(large_step),
            is_toggle,
        }
        .into()
    }

    pub fn track_fx_get_param_ex(
        &self,
        track: *mut MediaTrack,
        fx: u32,
        param: u32,
    ) -> GetParamExResult {
        let mut min_val = -1.0;
        let mut max_val = -1.0;
        let mut mid_val = -1.0;
        let value = require!(self.low, TrackFX_GetParamEx)(
            track,
            fx as i32,
            param as i32,
            &mut min_val as *mut f64,
            &mut max_val as *mut f64,
            &mut mid_val as *mut f64,
        );
        GetParamExResult {
            value: complain_if_minus_one(value),
            min_val: complain_if_minus_one(min_val),
            mid_val: complain_if_minus_one(mid_val),
            max_val: complain_if_minus_one(max_val),
        }
        .into()
    }

    pub fn undo_begin_block_2(&self, proj: *mut ReaProject) {
        require!(self.low, Undo_BeginBlock2)(proj);
    }

    pub fn undo_end_block_2(&self, proj: *mut ReaProject, descchange: &CStr, extraflags: u32) {
        require!(self.low, Undo_EndBlock2)(proj, descchange.as_ptr(), extraflags as i32);
    }

    pub fn undo_can_undo_2(&self, proj: *mut ReaProject) -> Option<&CStr> {
        let ptr = require!(self.low, Undo_CanUndo2)(proj);
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(ptr) })
    }

    pub fn undo_can_redo_2(&self, proj: *mut ReaProject) -> Option<&CStr> {
        let ptr = require!(self.low, Undo_CanRedo2)(proj);
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(ptr) })
    }

    // TODO-medium Should the return value be boolean?
    pub fn undo_do_undo_2(&self, proj: *mut ReaProject) -> i32 {
        require!(self.low, Undo_DoUndo2)(proj)
    }

    // TODO-medium Should the return value be boolean?
    pub fn undo_do_redo_2(&self, proj: *mut ReaProject) -> i32 {
        require!(self.low, Undo_DoRedo2)(proj)
    }

    pub fn mark_project_dirty(&self, proj: *mut ReaProject) {
        require!(self.low, MarkProjectDirty)(proj);
    }

    pub fn is_project_dirty(&self, proj: *mut ReaProject) -> i32 {
        require!(self.low, IsProjectDirty)(proj)
    }

    pub fn track_list_update_all_external_surfaces(&self) {
        require!(self.low, TrackList_UpdateAllExternalSurfaces)();
    }

    pub fn get_app_version(&self) -> &'static CStr {
        let ptr = require!(self.low, GetAppVersion)();
        unsafe { CStr::from_ptr(ptr) }
    }

    pub fn get_track_automation_mode(&self, tr: *mut MediaTrack) -> i32 {
        // TODO-high Use macro instead of unwrap everywhere in order to panic with good message on non-loaded function
        require!(self.low, GetTrackAutomationMode)(tr)
    }

    pub fn get_global_automation_override(&self) -> i32 {
        require!(self.low, GetGlobalAutomationOverride)()
    }

    pub fn get_track_envelope_by_name(
        &self,
        track: *mut MediaTrack,
        envname: &CStr,
    ) -> *mut TrackEnvelope {
        require!(self.low, GetTrackEnvelopeByName)(track, envname.as_ptr())
    }

    pub fn get_media_track_info_value(&self, tr: *mut MediaTrack, parmname: &CStr) -> f64 {
        require!(self.low, GetMediaTrackInfo_Value)(tr, parmname.as_ptr())
    }

    pub fn track_fx_get_count(&self, track: *mut MediaTrack) -> u32 {
        require!(self.low, TrackFX_GetCount)(track) as u32
    }

    pub fn track_fx_get_rec_count(&self, track: *mut MediaTrack) -> u32 {
        require!(self.low, TrackFX_GetRecCount)(track) as u32
    }

    pub fn track_fx_get_fx_guid(&self, track: *mut MediaTrack, fx: u32) -> *mut GUID {
        require!(self.low, TrackFX_GetFXGUID)(track, fx as i32)
    }

    pub fn track_fx_get_param_normalized(
        &self,
        track: *mut MediaTrack,
        fx: u32,
        param: u32,
    ) -> f64 {
        require!(self.low, TrackFX_GetParamNormalized)(track, fx as i32, param as i32)
    }

    pub fn get_master_track(&self, proj: *mut ReaProject) -> *mut MediaTrack {
        require!(self.low, GetMasterTrack)(proj)
    }

    pub fn guid_to_string(&self, g: &GUID) -> CString {
        let (guid_string, _) = with_string_buffer(64, |buffer, _| {
            require!(self.low, guidToString)(g as *const GUID, buffer)
        });
        guid_string
    }

    pub fn master_get_tempo(&self) -> f64 {
        require!(self.low, Master_GetTempo)()
    }

    pub fn set_current_bpm(&self, __proj: *mut ReaProject, bpm: f64, want_undo: bool) {
        require!(self.low, SetCurrentBPM)(__proj, bpm, want_undo);
    }

    pub fn master_get_play_rate(&self, project: *mut ReaProject) -> f64 {
        require!(self.low, Master_GetPlayRate)(project)
    }

    pub fn csurf_on_play_rate_change(&self, playrate: f64) {
        require!(self.low, CSurf_OnPlayRateChange)(playrate);
    }

    pub fn show_message_box(&self, msg: &CStr, title: &CStr, type_: u32) -> u32 {
        require!(self.low, ShowMessageBox)(msg.as_ptr(), title.as_ptr(), type_ as i32) as u32
    }

    pub fn string_to_guid(&self, str: &CStr) -> Option<GUID> {
        let mut guid = ZERO_GUID;
        require!(self.low, stringToGuid)(str.as_ptr(), &mut guid as *mut GUID);
        if guid == ZERO_GUID {
            return None;
        }
        Some(guid)
    }

    pub fn csurf_on_input_monitoring_change_ex(
        &self,
        trackid: *mut MediaTrack,
        monitor: u32,
        allowgang: bool,
    ) -> i32 {
        require!(self.low, CSurf_OnInputMonitorChangeEx)(trackid, monitor as i32, allowgang)
    }

    pub fn set_media_track_info_value(
        &self,
        tr: *mut MediaTrack,
        parmname: &CStr,
        newvalue: f64,
    ) -> bool {
        require!(self.low, SetMediaTrackInfo_Value)(tr, parmname.as_ptr(), newvalue)
    }

    pub fn stuff_midimessage(&self, mode: u32, msg1: u8, msg2: u8, msg3: u8) {
        require!(self.low, StuffMIDIMessage)(mode as i32, msg1 as i32, msg2 as i32, msg3 as i32);
    }

    pub fn db2slider(&self, x: f64) -> f64 {
        require!(self.low, DB2SLIDER)(x)
    }

    pub fn slider2db(&self, y: f64) -> f64 {
        require!(self.low, SLIDER2DB)(y)
    }

    pub fn get_track_ui_vol_pan(&self, track: *mut MediaTrack) -> Option<(f64, f64)> {
        let mut volume = 0.0;
        let mut pan = 0.0;
        let successful = require!(self.low, GetTrackUIVolPan)(
            track,
            &mut volume as *mut f64,
            &mut pan as *mut f64,
        );
        if !successful {
            return None;
        }
        Some((volume, pan))
    }

    // TODO-medium return boolean/Result?
    pub fn audio_reg_hardware_hook(&self, is_add: bool, reg: *const audio_hook_register_t) -> i32 {
        require!(self.low, Audio_RegHardwareHook)(is_add, reg)
    }

    pub fn csurf_set_surface_volume(
        &self,
        trackid: *mut MediaTrack,
        volume: f64,
        ignoresurf: *mut IReaperControlSurface,
    ) {
        require!(self.low, CSurf_SetSurfaceVolume)(trackid, volume, ignoresurf);
    }

    pub fn csurf_on_volume_change_ex(
        &self,
        trackid: *mut MediaTrack,
        volume: f64,
        relative: bool,
        allow_gang: bool,
    ) -> f64 {
        require!(self.low, CSurf_OnVolumeChangeEx)(trackid, volume, relative, allow_gang)
    }

    pub fn csurf_set_surface_pan(
        &self,
        trackid: *mut MediaTrack,
        pan: f64,
        ignoresurf: *mut IReaperControlSurface,
    ) {
        require!(self.low, CSurf_SetSurfacePan)(trackid, pan, ignoresurf);
    }

    pub fn csurf_on_pan_change_ex(
        &self,
        trackid: *mut MediaTrack,
        pan: f64,
        relative: bool,
        allow_gang: bool,
    ) -> f64 {
        require!(self.low, CSurf_OnPanChangeEx)(trackid, pan, relative, allow_gang)
    }

    pub fn count_selected_tracks_2(&self, proj: *mut ReaProject, wantmaster: bool) -> u32 {
        require!(self.low, CountSelectedTracks2)(proj, wantmaster) as u32
    }

    pub fn set_track_selected(&self, track: *mut MediaTrack, selected: bool) {
        require!(self.low, SetTrackSelected)(track, selected);
    }

    pub fn get_selected_track_2(
        &self,
        proj: *mut ReaProject,
        seltrackidx: u32,
        wantmaster: bool,
    ) -> *mut MediaTrack {
        require!(self.low, GetSelectedTrack2)(proj, seltrackidx as i32, wantmaster)
    }

    pub fn set_only_track_selected(&self, track: *mut MediaTrack) {
        require!(self.low, SetOnlyTrackSelected)(track);
    }

    pub fn delete_track(&self, tr: *mut MediaTrack) {
        require!(self.low, DeleteTrack)(tr);
    }

    pub fn get_track_num_sends(&self, tr: *mut MediaTrack, category: i32) -> u32 {
        require!(self.low, GetTrackNumSends)(tr, category) as u32
    }

    pub fn get_set_track_send_info(
        &self,
        tr: *mut MediaTrack,
        category: i32,
        sendidx: u32,
        parmname: &CStr,
        set_new_value: *mut c_void,
    ) -> *mut c_void {
        require!(self.low, GetSetTrackSendInfo)(
            tr,
            category,
            sendidx as i32,
            parmname.as_ptr(),
            set_new_value,
        )
    }

    pub fn get_track_state_chunk(
        &self,
        track: *mut MediaTrack,
        str_need_big_sz: u32,
        isundo_optional: bool,
    ) -> Option<CString> {
        let (chunk_content, successful) =
            with_string_buffer(str_need_big_sz, |buffer, max_size| {
                require!(self.low, GetTrackStateChunk)(track, buffer, max_size, isundo_optional)
            });
        if !successful {
            return None;
        }
        Some(chunk_content)
    }

    pub fn create_track_send(
        &self,
        tr: *mut MediaTrack,
        desttr_in_optional: *mut MediaTrack,
    ) -> u32 {
        require!(self.low, CreateTrackSend)(tr, desttr_in_optional) as u32
    }

    pub fn csurf_on_rec_arm_change_ex(
        &self,
        trackid: *mut MediaTrack,
        recarm: u32, // TODO-medium boolean!?
        allowgang: bool,
    ) -> bool {
        require!(self.low, CSurf_OnRecArmChangeEx)(trackid, recarm as i32, allowgang)
    }

    pub fn set_track_state_chunk(
        &self,
        track: *mut MediaTrack,
        str: &CStr,
        isundo_optional: bool,
    ) -> bool {
        require!(self.low, SetTrackStateChunk)(track, str.as_ptr(), isundo_optional)
    }

    pub fn track_fx_show(&self, track: *mut MediaTrack, index: u32, show_flag: u32) {
        require!(self.low, TrackFX_Show)(track, index as i32, show_flag as i32);
    }

    pub fn track_fx_get_floating_window(&self, track: *mut MediaTrack, index: u32) -> HWND {
        require!(self.low, TrackFX_GetFloatingWindow)(track, index as i32)
    }

    pub fn track_fx_get_open(&self, track: *mut MediaTrack, fx: u32) -> bool {
        require!(self.low, TrackFX_GetOpen)(track, fx as i32)
    }

    pub fn csurf_on_send_volume_change(
        &self,
        trackid: *mut MediaTrack,
        send_index: u32,
        volume: f64,
        relative: bool,
    ) -> f64 {
        require!(self.low, CSurf_OnSendVolumeChange)(trackid, send_index as i32, volume, relative)
    }

    pub fn csurf_on_send_pan_change(
        &self,
        trackid: *mut MediaTrack,
        send_index: u32,
        pan: f64,
        relative: bool,
    ) -> f64 {
        require!(self.low, CSurf_OnSendPanChange)(trackid, send_index as i32, pan, relative)
    }

    pub fn kbd_get_text_from_cmd(&self, cmd: u32, section: *mut KbdSectionInfo) -> Option<&CStr> {
        let ptr = require!(self.low, kbd_getTextFromCmd)(cmd, section);
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(ptr) })
    }

    // TODO-medium Maybe return Err if result is -1
    pub fn get_toggle_command_state_2(&self, section: *mut KbdSectionInfo, command_id: u32) -> i32 {
        require!(self.low, GetToggleCommandState2)(section, command_id as i32)
    }

    pub fn reverse_named_command_lookup(&self, command_id: i32) -> Option<&CStr> {
        let ptr = require!(self.low, ReverseNamedCommandLookup)(command_id);
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(ptr) })
    }

    // TODO-high Maybe prefer Result as return data type in such cases
    pub fn get_track_send_ui_vol_pan(
        &self,
        track: *mut MediaTrack,
        send_index: u32,
    ) -> Option<(f64, f64)> {
        let mut volume = 0.0;
        let mut pan = 0.0;
        let successful = require!(self.low, GetTrackSendUIVolPan)(
            track,
            send_index as i32,
            &mut volume as *mut f64,
            &mut pan as *mut f64,
        );
        if successful {
            Some((volume, pan))
        } else {
            None
        }
    }

    pub fn track_fx_get_preset_index(
        &self,
        track: *mut MediaTrack,
        fx: u32,
    ) -> Result<(u32, u32), ()> {
        let mut num_presets: i32 = 0;
        let index = require!(self.low, TrackFX_GetPresetIndex)(
            track,
            fx as i32,
            &mut num_presets as *mut i32,
        );
        if index == -1 {
            return Err(());
        }
        return Ok((index as u32, num_presets as u32));
    }

    pub fn track_fx_set_preset_by_index(&self, track: *mut MediaTrack, fx: u32, idx: i32) -> bool {
        require!(self.low, TrackFX_SetPresetByIndex)(track, fx as i32, idx)
    }

    pub fn track_fx_navigate_presets(
        &self,
        track: *mut MediaTrack,
        fx: u32,
        presetmove: i32,
    ) -> bool {
        require!(self.low, TrackFX_NavigatePresets)(track, fx as i32, presetmove)
    }

    pub fn track_fx_get_preset(
        &self,
        track: *mut MediaTrack,
        fx: u32,
        presetname_sz: u32,
    ) -> (bool, Option<CString>) {
        if presetname_sz == 0 {
            let state_matches_preset =
                require!(self.low, TrackFX_GetPreset)(track, fx as i32, null_mut(), 0);
            (state_matches_preset, None)
        } else {
            let (name, state_matches_preset) =
                with_string_buffer(presetname_sz, |buffer, max_size| {
                    require!(self.low, TrackFX_GetPreset)(track, fx as i32, buffer, max_size)
                });
            let name = if name.as_bytes().len() == 0 {
                None
            } else {
                Some(name)
            };
            (state_matches_preset, name)
        }
    }

    // TODO-high Rename
    // TODO-high Don't turn to owned string immediately
    pub fn convenient_get_media_track_info_string(
        &self,
        tr: *mut MediaTrack,
        parmname: &CStr,
    ) -> CString {
        let info = self.get_set_media_track_info(tr, parmname, null_mut());
        let info = info as *const c_char;
        let c_str = unsafe { CStr::from_ptr(info) };
        c_str.to_owned()
    }

    // TODO-high Rename or remove
    pub fn convenient_get_media_track_info_i32_value(
        &self,
        tr: *mut MediaTrack,
        parmname: &CStr,
    ) -> i32 {
        self.get_set_media_track_info(tr, parmname, null_mut()) as i32
    }

    // TODO-high Rename or remove
    pub fn convenient_get_media_track_info_i32_ptr(
        &self,
        tr: *mut MediaTrack,
        parmname: &CStr,
    ) -> i32 {
        let ptr = self.get_set_media_track_info(tr, parmname, null_mut()) as *mut i32;
        unsafe { *ptr }
    }

    // TODO-high Rename or remove
    pub fn convenient_get_media_track_info_guid(
        &self,
        tr: *mut MediaTrack,
        parmname: &CStr,
    ) -> *mut GUID {
        self.get_set_media_track_info(tr, parmname, null_mut()) as *mut GUID
    }
}

// TODO-high See what's really an option and what not
pub struct GetParameterStepSizesResult {
    pub step: f64,
    pub small_step: f64,
    pub large_step: f64,
    pub is_toggle: bool,
}

// TODO-high See what's really an option and what not
pub struct GetParamExResult {
    pub value: f64,
    pub min_val: f64,
    pub mid_val: f64,
    pub max_val: f64,
}

pub struct GetLastTouchedFxResultData {
    pub tracknumber: i32,
    pub fxnumber: i32,
    pub paramnumber: i32,
}

pub enum GetFocusedFxResult {
    None,
    TrackFx(GetFocusedFxTrackFxResultData),
    ItemFx(GetFocusedFxItemFxResultData),
}

pub struct GetFocusedFxItemFxResultData {
    pub tracknumber: i32,
    pub fxnumber: i32,
    pub itemnumber: i32,
}

pub struct GetFocusedFxTrackFxResultData {
    pub tracknumber: i32,
    pub fxnumber: i32,
}

// TODO-high Panic for now, just to detect which situations can actually occur
fn complain_if_minus_one(value: f64) -> f64 {
    if value == -1.0 {
        panic!("Out parameter was not set by REAPER")
    }
    value
}
