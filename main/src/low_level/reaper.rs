# ! [ allow ( non_upper_case_globals ) ] # ! [ allow ( non_camel_case_types ) ] # ! [ allow ( non_snake_case ) ]use super::{bindings::root, ReaperPluginContext};
use c_str_macro::c_str;
#[doc = r" This is the low-level API access point to all REAPER functions. In order to use it, you first"]
#[doc = r" must obtain an instance of this struct by invoking [`load`](struct.Reaper.html#method.load)."]
#[doc = r""]
#[doc = r" Please note that it's possible that functions are *not available*. This can be the case if"]
#[doc = r" the user runs your plug-in in an older version of REAPER which doesn't have that function yet."]
#[doc = r" Therefore each function in this struct is actually a function pointer wrapped"]
#[doc = r" in an `Option`. If you are sure your function will be there, you can just unwrap the option."]
#[doc = r" The medium-level API doesn't have this distinction anymore. It just unwraps the options"]
#[doc = r" automatically for the sake of convenience."]
#[derive(Default)]
pub struct Reaper {
    pub __mergesort: Option<
        unsafe extern "C" fn(
            base: *mut ::std::os::raw::c_void,
            nmemb: usize,
            size: usize,
            cmpfunc: ::std::option::Option<
                unsafe extern "C" fn(
                    arg1: *const ::std::os::raw::c_void,
                    arg2: *const ::std::os::raw::c_void,
                ) -> ::std::os::raw::c_int,
            >,
            tmpspace: *mut ::std::os::raw::c_void,
        ),
    >,
    pub AddCustomizableMenu: Option<
        unsafe extern "C" fn(
            menuidstr: *const ::std::os::raw::c_char,
            menuname: *const ::std::os::raw::c_char,
            kbdsecname: *const ::std::os::raw::c_char,
            addtomainmenu: bool,
        ) -> bool,
    >,
    pub AddExtensionsMainMenu: Option<unsafe extern "C" fn() -> bool>,
    pub AddMediaItemToTrack:
        Option<unsafe extern "C" fn(tr: *mut root::MediaTrack) -> *mut root::MediaItem>,
    pub AddProjectMarker: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            isrgn: bool,
            pos: f64,
            rgnend: f64,
            name: *const ::std::os::raw::c_char,
            wantidx: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub AddProjectMarker2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            isrgn: bool,
            pos: f64,
            rgnend: f64,
            name: *const ::std::os::raw::c_char,
            wantidx: ::std::os::raw::c_int,
            color: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub AddRemoveReaScript: Option<
        unsafe extern "C" fn(
            add: bool,
            sectionID: ::std::os::raw::c_int,
            scriptfn: *const ::std::os::raw::c_char,
            commit: bool,
        ) -> ::std::os::raw::c_int,
    >,
    pub AddTakeToMediaItem:
        Option<unsafe extern "C" fn(item: *mut root::MediaItem) -> *mut root::MediaItem_Take>,
    pub AddTempoTimeSigMarker: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            timepos: f64,
            bpm: f64,
            timesig_num: ::std::os::raw::c_int,
            timesig_denom: ::std::os::raw::c_int,
            lineartempochange: bool,
        ) -> bool,
    >,
    pub adjustZoom: Option<
        unsafe extern "C" fn(
            amt: f64,
            forceset: ::std::os::raw::c_int,
            doupd: bool,
            centermode: ::std::os::raw::c_int,
        ),
    >,
    pub AnyTrackSolo: Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> bool>,
    pub APIExists:
        Option<unsafe extern "C" fn(function_name: *const ::std::os::raw::c_char) -> bool>,
    pub APITest: Option<unsafe extern "C" fn()>,
    pub ApplyNudge: Option<
        unsafe extern "C" fn(
            project: *mut root::ReaProject,
            nudgeflag: ::std::os::raw::c_int,
            nudgewhat: ::std::os::raw::c_int,
            nudgeunits: ::std::os::raw::c_int,
            value: f64,
            reverse: bool,
            copies: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub ArmCommand: Option<
        unsafe extern "C" fn(
            cmd: ::std::os::raw::c_int,
            sectionname: *const ::std::os::raw::c_char,
        ),
    >,
    pub Audio_Init: Option<unsafe extern "C" fn()>,
    pub Audio_IsPreBuffer: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub Audio_IsRunning: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub Audio_Quit: Option<unsafe extern "C" fn()>,
    pub Audio_RegHardwareHook: Option<
        unsafe extern "C" fn(
            isAdd: bool,
            reg: *mut root::audio_hook_register_t,
        ) -> ::std::os::raw::c_int,
    >,
    pub AudioAccessorStateChanged:
        Option<unsafe extern "C" fn(accessor: *mut root::reaper_functions::AudioAccessor) -> bool>,
    pub AudioAccessorUpdate:
        Option<unsafe extern "C" fn(accessor: *mut root::reaper_functions::AudioAccessor)>,
    pub AudioAccessorValidateState:
        Option<unsafe extern "C" fn(accessor: *mut root::reaper_functions::AudioAccessor) -> bool>,
    pub BypassFxAllTracks: Option<unsafe extern "C" fn(bypass: ::std::os::raw::c_int)>,
    pub CalculatePeaks: Option<
        unsafe extern "C" fn(
            srcBlock: *mut root::PCM_source_transfer_t,
            pksBlock: *mut root::PCM_source_peaktransfer_t,
        ) -> ::std::os::raw::c_int,
    >,
    pub CalculatePeaksFloatSrcPtr: Option<
        unsafe extern "C" fn(
            srcBlock: *mut root::PCM_source_transfer_t,
            pksBlock: *mut root::PCM_source_peaktransfer_t,
        ) -> ::std::os::raw::c_int,
    >,
    pub ClearAllRecArmed: Option<unsafe extern "C" fn()>,
    pub ClearConsole: Option<unsafe extern "C" fn()>,
    pub ClearPeakCache: Option<unsafe extern "C" fn()>,
    pub ColorFromNative: Option<
        unsafe extern "C" fn(
            col: ::std::os::raw::c_int,
            rOut: *mut ::std::os::raw::c_int,
            gOut: *mut ::std::os::raw::c_int,
            bOut: *mut ::std::os::raw::c_int,
        ),
    >,
    pub ColorToNative: Option<
        unsafe extern "C" fn(
            r: ::std::os::raw::c_int,
            g: ::std::os::raw::c_int,
            b: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub CountActionShortcuts: Option<
        unsafe extern "C" fn(
            section: *mut root::KbdSectionInfo,
            cmdID: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub CountAutomationItems:
        Option<unsafe extern "C" fn(env: *mut root::TrackEnvelope) -> ::std::os::raw::c_int>,
    pub CountEnvelopePoints:
        Option<unsafe extern "C" fn(envelope: *mut root::TrackEnvelope) -> ::std::os::raw::c_int>,
    pub CountEnvelopePointsEx: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            autoitem_idx: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub CountMediaItems:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> ::std::os::raw::c_int>,
    pub CountProjectMarkers: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            num_markersOut: *mut ::std::os::raw::c_int,
            num_regionsOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub CountSelectedMediaItems:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> ::std::os::raw::c_int>,
    pub CountSelectedTracks:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> ::std::os::raw::c_int>,
    pub CountSelectedTracks2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            wantmaster: bool,
        ) -> ::std::os::raw::c_int,
    >,
    pub CountTakeEnvelopes:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take) -> ::std::os::raw::c_int>,
    pub CountTakes:
        Option<unsafe extern "C" fn(item: *mut root::MediaItem) -> ::std::os::raw::c_int>,
    pub CountTCPFXParms: Option<
        unsafe extern "C" fn(
            project: *mut root::ReaProject,
            track: *mut root::MediaTrack,
        ) -> ::std::os::raw::c_int,
    >,
    pub CountTempoTimeSigMarkers:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> ::std::os::raw::c_int>,
    pub CountTrackEnvelopes:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack) -> ::std::os::raw::c_int>,
    pub CountTrackMediaItems:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack) -> ::std::os::raw::c_int>,
    pub CountTracks:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> ::std::os::raw::c_int>,
    pub CreateLocalOscHandler: Option<
        unsafe extern "C" fn(
            obj: *mut ::std::os::raw::c_void,
            callback: *mut ::std::os::raw::c_void,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub CreateMIDIInput:
        Option<unsafe extern "C" fn(dev: ::std::os::raw::c_int) -> *mut root::midi_Input>,
    pub CreateMIDIOutput: Option<
        unsafe extern "C" fn(
            dev: ::std::os::raw::c_int,
            streamMode: bool,
            msoffset100: *mut ::std::os::raw::c_int,
        ) -> *mut root::midi_Output,
    >,
    pub CreateNewMIDIItemInProj: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            starttime: f64,
            endtime: f64,
            qnInOptional: *const bool,
        ) -> *mut root::MediaItem,
    >,
    pub CreateTakeAudioAccessor: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
        ) -> *mut root::reaper_functions::AudioAccessor,
    >,
    pub CreateTrackAudioAccessor: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
        ) -> *mut root::reaper_functions::AudioAccessor,
    >,
    pub CreateTrackSend: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            desttrInOptional: *mut root::MediaTrack,
        ) -> ::std::os::raw::c_int,
    >,
    pub CSurf_FlushUndo: Option<unsafe extern "C" fn(force: bool)>,
    pub CSurf_GetTouchState: Option<
        unsafe extern "C" fn(trackid: *mut root::MediaTrack, isPan: ::std::os::raw::c_int) -> bool,
    >,
    pub CSurf_GoEnd: Option<unsafe extern "C" fn()>,
    pub CSurf_GoStart: Option<unsafe extern "C" fn()>,
    pub CSurf_NumTracks: Option<unsafe extern "C" fn(mcpView: bool) -> ::std::os::raw::c_int>,
    pub CSurf_OnArrow:
        Option<unsafe extern "C" fn(whichdir: ::std::os::raw::c_int, wantzoom: bool)>,
    pub CSurf_OnFwd: Option<unsafe extern "C" fn(seekplay: ::std::os::raw::c_int)>,
    pub CSurf_OnFXChange: Option<
        unsafe extern "C" fn(trackid: *mut root::MediaTrack, en: ::std::os::raw::c_int) -> bool,
    >,
    pub CSurf_OnInputMonitorChange: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            monitor: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub CSurf_OnInputMonitorChangeEx: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            monitor: ::std::os::raw::c_int,
            allowgang: bool,
        ) -> ::std::os::raw::c_int,
    >,
    pub CSurf_OnMuteChange: Option<
        unsafe extern "C" fn(trackid: *mut root::MediaTrack, mute: ::std::os::raw::c_int) -> bool,
    >,
    pub CSurf_OnMuteChangeEx: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            mute: ::std::os::raw::c_int,
            allowgang: bool,
        ) -> bool,
    >,
    pub CSurf_OnOscControlMessage:
        Option<unsafe extern "C" fn(msg: *const ::std::os::raw::c_char, arg: *const f32)>,
    pub CSurf_OnPanChange: Option<
        unsafe extern "C" fn(trackid: *mut root::MediaTrack, pan: f64, relative: bool) -> f64,
    >,
    pub CSurf_OnPanChangeEx: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            pan: f64,
            relative: bool,
            allowGang: bool,
        ) -> f64,
    >,
    pub CSurf_OnPause: Option<unsafe extern "C" fn()>,
    pub CSurf_OnPlay: Option<unsafe extern "C" fn()>,
    pub CSurf_OnPlayRateChange: Option<unsafe extern "C" fn(playrate: f64)>,
    pub CSurf_OnRecArmChange: Option<
        unsafe extern "C" fn(trackid: *mut root::MediaTrack, recarm: ::std::os::raw::c_int) -> bool,
    >,
    pub CSurf_OnRecArmChangeEx: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            recarm: ::std::os::raw::c_int,
            allowgang: bool,
        ) -> bool,
    >,
    pub CSurf_OnRecord: Option<unsafe extern "C" fn()>,
    pub CSurf_OnRecvPanChange: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            recv_index: ::std::os::raw::c_int,
            pan: f64,
            relative: bool,
        ) -> f64,
    >,
    pub CSurf_OnRecvVolumeChange: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            recv_index: ::std::os::raw::c_int,
            volume: f64,
            relative: bool,
        ) -> f64,
    >,
    pub CSurf_OnRew: Option<unsafe extern "C" fn(seekplay: ::std::os::raw::c_int)>,
    pub CSurf_OnRewFwd:
        Option<unsafe extern "C" fn(seekplay: ::std::os::raw::c_int, dir: ::std::os::raw::c_int)>,
    pub CSurf_OnScroll:
        Option<unsafe extern "C" fn(xdir: ::std::os::raw::c_int, ydir: ::std::os::raw::c_int)>,
    pub CSurf_OnSelectedChange: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            selected: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub CSurf_OnSendPanChange: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            send_index: ::std::os::raw::c_int,
            pan: f64,
            relative: bool,
        ) -> f64,
    >,
    pub CSurf_OnSendVolumeChange: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            send_index: ::std::os::raw::c_int,
            volume: f64,
            relative: bool,
        ) -> f64,
    >,
    pub CSurf_OnSoloChange: Option<
        unsafe extern "C" fn(trackid: *mut root::MediaTrack, solo: ::std::os::raw::c_int) -> bool,
    >,
    pub CSurf_OnSoloChangeEx: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            solo: ::std::os::raw::c_int,
            allowgang: bool,
        ) -> bool,
    >,
    pub CSurf_OnStop: Option<unsafe extern "C" fn()>,
    pub CSurf_OnTempoChange: Option<unsafe extern "C" fn(bpm: f64)>,
    pub CSurf_OnTrackSelection: Option<unsafe extern "C" fn(trackid: *mut root::MediaTrack)>,
    pub CSurf_OnVolumeChange: Option<
        unsafe extern "C" fn(trackid: *mut root::MediaTrack, volume: f64, relative: bool) -> f64,
    >,
    pub CSurf_OnVolumeChangeEx: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            volume: f64,
            relative: bool,
            allowGang: bool,
        ) -> f64,
    >,
    pub CSurf_OnWidthChange: Option<
        unsafe extern "C" fn(trackid: *mut root::MediaTrack, width: f64, relative: bool) -> f64,
    >,
    pub CSurf_OnWidthChangeEx: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            width: f64,
            relative: bool,
            allowGang: bool,
        ) -> f64,
    >,
    pub CSurf_OnZoom:
        Option<unsafe extern "C" fn(xdir: ::std::os::raw::c_int, ydir: ::std::os::raw::c_int)>,
    pub CSurf_ResetAllCachedVolPanStates: Option<unsafe extern "C" fn()>,
    pub CSurf_ScrubAmt: Option<unsafe extern "C" fn(amt: f64)>,
    pub CSurf_SetAutoMode: Option<
        unsafe extern "C" fn(
            mode: ::std::os::raw::c_int,
            ignoresurf: *mut root::IReaperControlSurface,
        ),
    >,
    pub CSurf_SetPlayState: Option<
        unsafe extern "C" fn(
            play: bool,
            pause: bool,
            rec: bool,
            ignoresurf: *mut root::IReaperControlSurface,
        ),
    >,
    pub CSurf_SetRepeatState:
        Option<unsafe extern "C" fn(rep: bool, ignoresurf: *mut root::IReaperControlSurface)>,
    pub CSurf_SetSurfaceMute: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            mute: bool,
            ignoresurf: *mut root::IReaperControlSurface,
        ),
    >,
    pub CSurf_SetSurfacePan: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            pan: f64,
            ignoresurf: *mut root::IReaperControlSurface,
        ),
    >,
    pub CSurf_SetSurfaceRecArm: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            recarm: bool,
            ignoresurf: *mut root::IReaperControlSurface,
        ),
    >,
    pub CSurf_SetSurfaceSelected: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            selected: bool,
            ignoresurf: *mut root::IReaperControlSurface,
        ),
    >,
    pub CSurf_SetSurfaceSolo: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            solo: bool,
            ignoresurf: *mut root::IReaperControlSurface,
        ),
    >,
    pub CSurf_SetSurfaceVolume: Option<
        unsafe extern "C" fn(
            trackid: *mut root::MediaTrack,
            volume: f64,
            ignoresurf: *mut root::IReaperControlSurface,
        ),
    >,
    pub CSurf_SetTrackListChange: Option<unsafe extern "C" fn()>,
    pub CSurf_TrackFromID: Option<
        unsafe extern "C" fn(idx: ::std::os::raw::c_int, mcpView: bool) -> *mut root::MediaTrack,
    >,
    pub CSurf_TrackToID: Option<
        unsafe extern "C" fn(track: *mut root::MediaTrack, mcpView: bool) -> ::std::os::raw::c_int,
    >,
    pub DB2SLIDER: Option<unsafe extern "C" fn(x: f64) -> f64>,
    pub DeleteActionShortcut: Option<
        unsafe extern "C" fn(
            section: *mut root::KbdSectionInfo,
            cmdID: ::std::os::raw::c_int,
            shortcutidx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub DeleteEnvelopePointEx: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            autoitem_idx: ::std::os::raw::c_int,
            ptidx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub DeleteEnvelopePointRange: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            time_start: f64,
            time_end: f64,
        ) -> bool,
    >,
    pub DeleteEnvelopePointRangeEx: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            autoitem_idx: ::std::os::raw::c_int,
            time_start: f64,
            time_end: f64,
        ) -> bool,
    >,
    pub DeleteExtState: Option<
        unsafe extern "C" fn(
            section: *const ::std::os::raw::c_char,
            key: *const ::std::os::raw::c_char,
            persist: bool,
        ),
    >,
    pub DeleteProjectMarker: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            markrgnindexnumber: ::std::os::raw::c_int,
            isrgn: bool,
        ) -> bool,
    >,
    pub DeleteProjectMarkerByIndex: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            markrgnidx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub DeleteTakeStretchMarkers: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            idx: ::std::os::raw::c_int,
            countInOptional: *const ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub DeleteTempoTimeSigMarker: Option<
        unsafe extern "C" fn(
            project: *mut root::ReaProject,
            markerindex: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub DeleteTrack: Option<unsafe extern "C" fn(tr: *mut root::MediaTrack)>,
    pub DeleteTrackMediaItem:
        Option<unsafe extern "C" fn(tr: *mut root::MediaTrack, it: *mut root::MediaItem) -> bool>,
    pub DestroyAudioAccessor:
        Option<unsafe extern "C" fn(accessor: *mut root::reaper_functions::AudioAccessor)>,
    pub DestroyLocalOscHandler:
        Option<unsafe extern "C" fn(local_osc_handler: *mut ::std::os::raw::c_void)>,
    pub DoActionShortcutDialog: Option<
        unsafe extern "C" fn(
            hwnd: root::HWND,
            section: *mut root::KbdSectionInfo,
            cmdID: ::std::os::raw::c_int,
            shortcutidx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub Dock_UpdateDockID: Option<
        unsafe extern "C" fn(
            ident_str: *const ::std::os::raw::c_char,
            whichDock: ::std::os::raw::c_int,
        ),
    >,
    pub DockGetPosition:
        Option<unsafe extern "C" fn(whichDock: ::std::os::raw::c_int) -> ::std::os::raw::c_int>,
    pub DockIsChildOfDock: Option<
        unsafe extern "C" fn(
            hwnd: root::HWND,
            isFloatingDockerOut: *mut bool,
        ) -> ::std::os::raw::c_int,
    >,
    pub DockWindowActivate: Option<unsafe extern "C" fn(hwnd: root::HWND)>,
    pub DockWindowAdd: Option<
        unsafe extern "C" fn(
            hwnd: root::HWND,
            name: *const ::std::os::raw::c_char,
            pos: ::std::os::raw::c_int,
            allowShow: bool,
        ),
    >,
    pub DockWindowAddEx: Option<
        unsafe extern "C" fn(
            hwnd: root::HWND,
            name: *const ::std::os::raw::c_char,
            identstr: *const ::std::os::raw::c_char,
            allowShow: bool,
        ),
    >,
    pub DockWindowRefresh: Option<unsafe extern "C" fn()>,
    pub DockWindowRefreshForHWND: Option<unsafe extern "C" fn(hwnd: root::HWND)>,
    pub DockWindowRemove: Option<unsafe extern "C" fn(hwnd: root::HWND)>,
    pub DuplicateCustomizableMenu: Option<
        unsafe extern "C" fn(
            srcmenu: *mut ::std::os::raw::c_void,
            destmenu: *mut ::std::os::raw::c_void,
        ) -> bool,
    >,
    pub EditTempoTimeSigMarker: Option<
        unsafe extern "C" fn(
            project: *mut root::ReaProject,
            markerindex: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub EnsureNotCompletelyOffscreen: Option<unsafe extern "C" fn(rInOut: *mut root::RECT)>,
    pub EnumerateFiles: Option<
        unsafe extern "C" fn(
            path: *const ::std::os::raw::c_char,
            fileindex: ::std::os::raw::c_int,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub EnumerateSubdirectories: Option<
        unsafe extern "C" fn(
            path: *const ::std::os::raw::c_char,
            subdirindex: ::std::os::raw::c_int,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub EnumPitchShiftModes: Option<
        unsafe extern "C" fn(
            mode: ::std::os::raw::c_int,
            strOut: *mut *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub EnumPitchShiftSubModes: Option<
        unsafe extern "C" fn(
            mode: ::std::os::raw::c_int,
            submode: ::std::os::raw::c_int,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub EnumProjectMarkers: Option<
        unsafe extern "C" fn(
            idx: ::std::os::raw::c_int,
            isrgnOut: *mut bool,
            posOut: *mut f64,
            rgnendOut: *mut f64,
            nameOut: *mut *const ::std::os::raw::c_char,
            markrgnindexnumberOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub EnumProjectMarkers2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            idx: ::std::os::raw::c_int,
            isrgnOut: *mut bool,
            posOut: *mut f64,
            rgnendOut: *mut f64,
            nameOut: *mut *const ::std::os::raw::c_char,
            markrgnindexnumberOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub EnumProjectMarkers3: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            idx: ::std::os::raw::c_int,
            isrgnOut: *mut bool,
            posOut: *mut f64,
            rgnendOut: *mut f64,
            nameOut: *mut *const ::std::os::raw::c_char,
            markrgnindexnumberOut: *mut ::std::os::raw::c_int,
            colorOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub EnumProjects: Option<
        unsafe extern "C" fn(
            idx: ::std::os::raw::c_int,
            projfnOutOptional: *mut ::std::os::raw::c_char,
            projfnOutOptional_sz: ::std::os::raw::c_int,
        ) -> *mut root::ReaProject,
    >,
    pub EnumProjExtState: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            extname: *const ::std::os::raw::c_char,
            idx: ::std::os::raw::c_int,
            keyOutOptional: *mut ::std::os::raw::c_char,
            keyOutOptional_sz: ::std::os::raw::c_int,
            valOutOptional: *mut ::std::os::raw::c_char,
            valOutOptional_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub EnumRegionRenderMatrix: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            regionindex: ::std::os::raw::c_int,
            rendertrack: ::std::os::raw::c_int,
        ) -> *mut root::MediaTrack,
    >,
    pub EnumTrackMIDIProgramNames: Option<
        unsafe extern "C" fn(
            track: ::std::os::raw::c_int,
            programNumber: ::std::os::raw::c_int,
            programName: *mut ::std::os::raw::c_char,
            programName_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub EnumTrackMIDIProgramNamesEx: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            track: *mut root::MediaTrack,
            programNumber: ::std::os::raw::c_int,
            programName: *mut ::std::os::raw::c_char,
            programName_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub Envelope_Evaluate: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            time: f64,
            samplerate: f64,
            samplesRequested: ::std::os::raw::c_int,
            valueOutOptional: *mut f64,
            dVdSOutOptional: *mut f64,
            ddVdSOutOptional: *mut f64,
            dddVdSOutOptional: *mut f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub Envelope_FormatValue: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            value: f64,
            bufOut: *mut ::std::os::raw::c_char,
            bufOut_sz: ::std::os::raw::c_int,
        ),
    >,
    pub Envelope_GetParentTake: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            indexOutOptional: *mut ::std::os::raw::c_int,
            index2OutOptional: *mut ::std::os::raw::c_int,
        ) -> *mut root::MediaItem_Take,
    >,
    pub Envelope_GetParentTrack: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            indexOutOptional: *mut ::std::os::raw::c_int,
            index2OutOptional: *mut ::std::os::raw::c_int,
        ) -> *mut root::MediaTrack,
    >,
    pub Envelope_SortPoints:
        Option<unsafe extern "C" fn(envelope: *mut root::TrackEnvelope) -> bool>,
    pub Envelope_SortPointsEx: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            autoitem_idx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub ExecProcess: Option<
        unsafe extern "C" fn(
            cmdline: *const ::std::os::raw::c_char,
            timeoutmsec: ::std::os::raw::c_int,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub file_exists: Option<unsafe extern "C" fn(path: *const ::std::os::raw::c_char) -> bool>,
    pub FindTempoTimeSigMarker: Option<
        unsafe extern "C" fn(project: *mut root::ReaProject, time: f64) -> ::std::os::raw::c_int,
    >,
    pub format_timestr: Option<
        unsafe extern "C" fn(
            tpos: f64,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ),
    >,
    pub format_timestr_len: Option<
        unsafe extern "C" fn(
            tpos: f64,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
            offset: f64,
            modeoverride: ::std::os::raw::c_int,
        ),
    >,
    pub format_timestr_pos: Option<
        unsafe extern "C" fn(
            tpos: f64,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
            modeoverride: ::std::os::raw::c_int,
        ),
    >,
    pub FreeHeapPtr: Option<unsafe extern "C" fn(ptr: *mut ::std::os::raw::c_void)>,
    pub genGuid: Option<unsafe extern "C" fn(g: *mut root::GUID)>,
    pub get_config_var: Option<
        unsafe extern "C" fn(
            name: *const ::std::os::raw::c_char,
            szOut: *mut ::std::os::raw::c_int,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub get_config_var_string: Option<
        unsafe extern "C" fn(
            name: *const ::std::os::raw::c_char,
            bufOut: *mut ::std::os::raw::c_char,
            bufOut_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub get_ini_file: Option<unsafe extern "C" fn() -> *const ::std::os::raw::c_char>,
    pub get_midi_config_var: Option<
        unsafe extern "C" fn(
            name: *const ::std::os::raw::c_char,
            szOut: *mut ::std::os::raw::c_int,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub GetActionShortcutDesc: Option<
        unsafe extern "C" fn(
            section: *mut root::KbdSectionInfo,
            cmdID: ::std::os::raw::c_int,
            shortcutidx: ::std::os::raw::c_int,
            desc: *mut ::std::os::raw::c_char,
            desclen: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetActiveTake:
        Option<unsafe extern "C" fn(item: *mut root::MediaItem) -> *mut root::MediaItem_Take>,
    pub GetAllProjectPlayStates:
        Option<unsafe extern "C" fn(ignoreProject: *mut root::ReaProject) -> ::std::os::raw::c_int>,
    pub GetAppVersion: Option<unsafe extern "C" fn() -> *const ::std::os::raw::c_char>,
    pub GetArmedCommand: Option<
        unsafe extern "C" fn(
            secOut: *mut ::std::os::raw::c_char,
            secOut_sz: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetAudioAccessorEndTime:
        Option<unsafe extern "C" fn(accessor: *mut root::reaper_functions::AudioAccessor) -> f64>,
    pub GetAudioAccessorHash: Option<
        unsafe extern "C" fn(
            accessor: *mut root::reaper_functions::AudioAccessor,
            hashNeed128: *mut ::std::os::raw::c_char,
        ),
    >,
    pub GetAudioAccessorSamples: Option<
        unsafe extern "C" fn(
            accessor: *mut root::reaper_functions::AudioAccessor,
            samplerate: ::std::os::raw::c_int,
            numchannels: ::std::os::raw::c_int,
            starttime_sec: f64,
            numsamplesperchannel: ::std::os::raw::c_int,
            samplebuffer: *mut f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetAudioAccessorStartTime:
        Option<unsafe extern "C" fn(accessor: *mut root::reaper_functions::AudioAccessor) -> f64>,
    pub GetAudioDeviceInfo: Option<
        unsafe extern "C" fn(
            attribute: *const ::std::os::raw::c_char,
            desc: *mut ::std::os::raw::c_char,
            desc_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetColorTheme: Option<
        unsafe extern "C" fn(
            idx: ::std::os::raw::c_int,
            defval: ::std::os::raw::c_int,
        ) -> root::INT_PTR,
    >,
    pub GetColorThemeStruct: Option<
        unsafe extern "C" fn(szOut: *mut ::std::os::raw::c_int) -> *mut ::std::os::raw::c_void,
    >,
    pub GetConfigWantsDock: Option<
        unsafe extern "C" fn(ident_str: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int,
    >,
    pub GetContextMenu: Option<unsafe extern "C" fn(idx: ::std::os::raw::c_int) -> root::HMENU>,
    pub GetCurrentProjectInLoadSave: Option<unsafe extern "C" fn() -> *mut root::ReaProject>,
    pub GetCursorContext: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetCursorContext2:
        Option<unsafe extern "C" fn(want_last_valid: bool) -> ::std::os::raw::c_int>,
    pub GetCursorPosition: Option<unsafe extern "C" fn() -> f64>,
    pub GetCursorPositionEx: Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> f64>,
    pub GetDisplayedMediaItemColor:
        Option<unsafe extern "C" fn(item: *mut root::MediaItem) -> ::std::os::raw::c_int>,
    pub GetDisplayedMediaItemColor2: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            take: *mut root::MediaItem_Take,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetEnvelopeInfo_Value: Option<
        unsafe extern "C" fn(
            tr: *mut root::TrackEnvelope,
            parmname: *const ::std::os::raw::c_char,
        ) -> f64,
    >,
    pub GetEnvelopeName: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            bufOut: *mut ::std::os::raw::c_char,
            bufOut_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetEnvelopePoint: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            ptidx: ::std::os::raw::c_int,
            timeOutOptional: *mut f64,
            valueOutOptional: *mut f64,
            shapeOutOptional: *mut ::std::os::raw::c_int,
            tensionOutOptional: *mut f64,
            selectedOutOptional: *mut bool,
        ) -> bool,
    >,
    pub GetEnvelopePointByTime: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            time: f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetEnvelopePointByTimeEx: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            autoitem_idx: ::std::os::raw::c_int,
            time: f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetEnvelopePointEx: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            autoitem_idx: ::std::os::raw::c_int,
            ptidx: ::std::os::raw::c_int,
            timeOutOptional: *mut f64,
            valueOutOptional: *mut f64,
            shapeOutOptional: *mut ::std::os::raw::c_int,
            tensionOutOptional: *mut f64,
            selectedOutOptional: *mut bool,
        ) -> bool,
    >,
    pub GetEnvelopeScalingMode:
        Option<unsafe extern "C" fn(env: *mut root::TrackEnvelope) -> ::std::os::raw::c_int>,
    pub GetEnvelopeStateChunk: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            strNeedBig: *mut ::std::os::raw::c_char,
            strNeedBig_sz: ::std::os::raw::c_int,
            isundoOptional: bool,
        ) -> bool,
    >,
    pub GetExePath: Option<unsafe extern "C" fn() -> *const ::std::os::raw::c_char>,
    pub GetExtState: Option<
        unsafe extern "C" fn(
            section: *const ::std::os::raw::c_char,
            key: *const ::std::os::raw::c_char,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub GetFocusedFX: Option<
        unsafe extern "C" fn(
            tracknumberOut: *mut ::std::os::raw::c_int,
            itemnumberOut: *mut ::std::os::raw::c_int,
            fxnumberOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetFreeDiskSpaceForRecordPath: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            pathidx: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetFXEnvelope: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fxindex: ::std::os::raw::c_int,
            parameterindex: ::std::os::raw::c_int,
            create: bool,
        ) -> *mut root::TrackEnvelope,
    >,
    pub GetGlobalAutomationOverride: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetHZoomLevel: Option<unsafe extern "C" fn() -> f64>,
    pub GetIconThemePointer: Option<
        unsafe extern "C" fn(name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_void,
    >,
    pub GetIconThemePointerForDPI: Option<
        unsafe extern "C" fn(
            name: *const ::std::os::raw::c_char,
            dpisc: ::std::os::raw::c_int,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub GetIconThemeStruct: Option<
        unsafe extern "C" fn(szOut: *mut ::std::os::raw::c_int) -> *mut ::std::os::raw::c_void,
    >,
    pub GetInputChannelName: Option<
        unsafe extern "C" fn(channelIndex: ::std::os::raw::c_int) -> *const ::std::os::raw::c_char,
    >,
    pub GetInputOutputLatency: Option<
        unsafe extern "C" fn(
            inputlatencyOut: *mut ::std::os::raw::c_int,
            outputLatencyOut: *mut ::std::os::raw::c_int,
        ),
    >,
    pub GetItemEditingTime2: Option<
        unsafe extern "C" fn(
            which_itemOut: *mut *mut root::PCM_source,
            flagsOut: *mut ::std::os::raw::c_int,
        ) -> f64,
    >,
    pub GetItemFromPoint: Option<
        unsafe extern "C" fn(
            screen_x: ::std::os::raw::c_int,
            screen_y: ::std::os::raw::c_int,
            allow_locked: bool,
            takeOutOptional: *mut *mut root::MediaItem_Take,
        ) -> *mut root::MediaItem,
    >,
    pub GetItemProjectContext:
        Option<unsafe extern "C" fn(item: *mut root::MediaItem) -> *mut root::ReaProject>,
    pub GetItemStateChunk: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            strNeedBig: *mut ::std::os::raw::c_char,
            strNeedBig_sz: ::std::os::raw::c_int,
            isundoOptional: bool,
        ) -> bool,
    >,
    pub GetLastColorThemeFile: Option<unsafe extern "C" fn() -> *const ::std::os::raw::c_char>,
    pub GetLastMarkerAndCurRegion: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            time: f64,
            markeridxOut: *mut ::std::os::raw::c_int,
            regionidxOut: *mut ::std::os::raw::c_int,
        ),
    >,
    pub GetLastTouchedFX: Option<
        unsafe extern "C" fn(
            tracknumberOut: *mut ::std::os::raw::c_int,
            fxnumberOut: *mut ::std::os::raw::c_int,
            paramnumberOut: *mut ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetLastTouchedTrack: Option<unsafe extern "C" fn() -> *mut root::MediaTrack>,
    pub GetMainHwnd: Option<unsafe extern "C" fn() -> root::HWND>,
    pub GetMasterMuteSoloFlags: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetMasterTrack:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> *mut root::MediaTrack>,
    pub GetMasterTrackVisibility: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetMaxMidiInputs: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetMaxMidiOutputs: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetMediaItem: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            itemidx: ::std::os::raw::c_int,
        ) -> *mut root::MediaItem,
    >,
    pub GetMediaItem_Track:
        Option<unsafe extern "C" fn(item: *mut root::MediaItem) -> *mut root::MediaTrack>,
    pub GetMediaItemInfo_Value: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            parmname: *const ::std::os::raw::c_char,
        ) -> f64,
    >,
    pub GetMediaItemNumTakes:
        Option<unsafe extern "C" fn(item: *mut root::MediaItem) -> ::std::os::raw::c_int>,
    pub GetMediaItemTake: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            tk: ::std::os::raw::c_int,
        ) -> *mut root::MediaItem_Take,
    >,
    pub GetMediaItemTake_Item:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take) -> *mut root::MediaItem>,
    pub GetMediaItemTake_Peaks: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            peakrate: f64,
            starttime: f64,
            numchannels: ::std::os::raw::c_int,
            numsamplesperchannel: ::std::os::raw::c_int,
            want_extra_type: ::std::os::raw::c_int,
            buf: *mut f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetMediaItemTake_Source:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take) -> *mut root::PCM_source>,
    pub GetMediaItemTake_Track:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take) -> *mut root::MediaTrack>,
    pub GetMediaItemTakeByGUID: Option<
        unsafe extern "C" fn(
            project: *mut root::ReaProject,
            guid: *const root::GUID,
        ) -> *mut root::MediaItem_Take,
    >,
    pub GetMediaItemTakeInfo_Value: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            parmname: *const ::std::os::raw::c_char,
        ) -> f64,
    >,
    pub GetMediaItemTrack:
        Option<unsafe extern "C" fn(item: *mut root::MediaItem) -> *mut root::MediaTrack>,
    pub GetMediaSourceFileName: Option<
        unsafe extern "C" fn(
            source: *mut root::PCM_source,
            filenamebuf: *mut ::std::os::raw::c_char,
            filenamebuf_sz: ::std::os::raw::c_int,
        ),
    >,
    pub GetMediaSourceLength: Option<
        unsafe extern "C" fn(source: *mut root::PCM_source, lengthIsQNOut: *mut bool) -> f64,
    >,
    pub GetMediaSourceNumChannels:
        Option<unsafe extern "C" fn(source: *mut root::PCM_source) -> ::std::os::raw::c_int>,
    pub GetMediaSourceParent:
        Option<unsafe extern "C" fn(src: *mut root::PCM_source) -> *mut root::PCM_source>,
    pub GetMediaSourceSampleRate:
        Option<unsafe extern "C" fn(source: *mut root::PCM_source) -> ::std::os::raw::c_int>,
    pub GetMediaSourceType: Option<
        unsafe extern "C" fn(
            source: *mut root::PCM_source,
            typebuf: *mut ::std::os::raw::c_char,
            typebuf_sz: ::std::os::raw::c_int,
        ),
    >,
    pub GetMediaTrackInfo_Value: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            parmname: *const ::std::os::raw::c_char,
        ) -> f64,
    >,
    pub GetMIDIInputName: Option<
        unsafe extern "C" fn(
            dev: ::std::os::raw::c_int,
            nameout: *mut ::std::os::raw::c_char,
            nameout_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetMIDIOutputName: Option<
        unsafe extern "C" fn(
            dev: ::std::os::raw::c_int,
            nameout: *mut ::std::os::raw::c_char,
            nameout_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetMixerScroll: Option<unsafe extern "C" fn() -> *mut root::MediaTrack>,
    pub GetMouseModifier: Option<
        unsafe extern "C" fn(
            context: *const ::std::os::raw::c_char,
            modifier_flag: ::std::os::raw::c_int,
            action: *mut ::std::os::raw::c_char,
            action_sz: ::std::os::raw::c_int,
        ),
    >,
    pub GetMousePosition: Option<
        unsafe extern "C" fn(xOut: *mut ::std::os::raw::c_int, yOut: *mut ::std::os::raw::c_int),
    >,
    pub GetNumAudioInputs: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetNumAudioOutputs: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetNumMIDIInputs: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetNumMIDIOutputs: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetNumTracks: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetOS: Option<unsafe extern "C" fn() -> *const ::std::os::raw::c_char>,
    pub GetOutputChannelName: Option<
        unsafe extern "C" fn(channelIndex: ::std::os::raw::c_int) -> *const ::std::os::raw::c_char,
    >,
    pub GetOutputLatency: Option<unsafe extern "C" fn() -> f64>,
    pub GetParentTrack:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack) -> *mut root::MediaTrack>,
    pub GetPeakFileName: Option<
        unsafe extern "C" fn(
            fn_: *const ::std::os::raw::c_char,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ),
    >,
    pub GetPeakFileNameEx: Option<
        unsafe extern "C" fn(
            fn_: *const ::std::os::raw::c_char,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
            forWrite: bool,
        ),
    >,
    pub GetPeakFileNameEx2: Option<
        unsafe extern "C" fn(
            fn_: *const ::std::os::raw::c_char,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
            forWrite: bool,
            peaksfileextension: *const ::std::os::raw::c_char,
        ),
    >,
    pub GetPeaksBitmap: Option<
        unsafe extern "C" fn(
            pks: *mut root::PCM_source_peaktransfer_t,
            maxamp: f64,
            w: ::std::os::raw::c_int,
            h: ::std::os::raw::c_int,
            bmp: *mut root::reaper_functions::LICE_IBitmap,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub GetPlayPosition: Option<unsafe extern "C" fn() -> f64>,
    pub GetPlayPosition2: Option<unsafe extern "C" fn() -> f64>,
    pub GetPlayPosition2Ex: Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> f64>,
    pub GetPlayPositionEx: Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> f64>,
    pub GetPlayState: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub GetPlayStateEx:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> ::std::os::raw::c_int>,
    pub GetPreferredDiskReadMode: Option<
        unsafe extern "C" fn(
            mode: *mut ::std::os::raw::c_int,
            nb: *mut ::std::os::raw::c_int,
            bs: *mut ::std::os::raw::c_int,
        ),
    >,
    pub GetPreferredDiskReadModePeak: Option<
        unsafe extern "C" fn(
            mode: *mut ::std::os::raw::c_int,
            nb: *mut ::std::os::raw::c_int,
            bs: *mut ::std::os::raw::c_int,
        ),
    >,
    pub GetPreferredDiskWriteMode: Option<
        unsafe extern "C" fn(
            mode: *mut ::std::os::raw::c_int,
            nb: *mut ::std::os::raw::c_int,
            bs: *mut ::std::os::raw::c_int,
        ),
    >,
    pub GetProjectLength: Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> f64>,
    pub GetProjectName: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ),
    >,
    pub GetProjectPath: Option<
        unsafe extern "C" fn(buf: *mut ::std::os::raw::c_char, buf_sz: ::std::os::raw::c_int),
    >,
    pub GetProjectPathEx: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ),
    >,
    pub GetProjectStateChangeCount:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> ::std::os::raw::c_int>,
    pub GetProjectTimeOffset:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject, rndframe: bool) -> f64>,
    pub GetProjectTimeSignature: Option<unsafe extern "C" fn(bpmOut: *mut f64, bpiOut: *mut f64)>,
    pub GetProjectTimeSignature2: Option<
        unsafe extern "C" fn(proj: *mut root::ReaProject, bpmOut: *mut f64, bpiOut: *mut f64),
    >,
    pub GetProjExtState: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            extname: *const ::std::os::raw::c_char,
            key: *const ::std::os::raw::c_char,
            valOutNeedBig: *mut ::std::os::raw::c_char,
            valOutNeedBig_sz: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetResourcePath: Option<unsafe extern "C" fn() -> *const ::std::os::raw::c_char>,
    pub GetSelectedEnvelope:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> *mut root::TrackEnvelope>,
    pub GetSelectedMediaItem: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            selitem: ::std::os::raw::c_int,
        ) -> *mut root::MediaItem,
    >,
    pub GetSelectedTrack: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            seltrackidx: ::std::os::raw::c_int,
        ) -> *mut root::MediaTrack,
    >,
    pub GetSelectedTrack2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            seltrackidx: ::std::os::raw::c_int,
            wantmaster: bool,
        ) -> *mut root::MediaTrack,
    >,
    pub GetSelectedTrackEnvelope:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> *mut root::TrackEnvelope>,
    pub GetSet_ArrangeView2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            isSet: bool,
            screen_x_start: ::std::os::raw::c_int,
            screen_x_end: ::std::os::raw::c_int,
            start_timeOut: *mut f64,
            end_timeOut: *mut f64,
        ),
    >,
    pub GetSet_LoopTimeRange: Option<
        unsafe extern "C" fn(
            isSet: bool,
            isLoop: bool,
            startOut: *mut f64,
            endOut: *mut f64,
            allowautoseek: bool,
        ),
    >,
    pub GetSet_LoopTimeRange2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            isSet: bool,
            isLoop: bool,
            startOut: *mut f64,
            endOut: *mut f64,
            allowautoseek: bool,
        ),
    >,
    pub GetSetAutomationItemInfo: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            autoitem_idx: ::std::os::raw::c_int,
            desc: *const ::std::os::raw::c_char,
            value: f64,
            is_set: bool,
        ) -> f64,
    >,
    pub GetSetAutomationItemInfo_String: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            autoitem_idx: ::std::os::raw::c_int,
            desc: *const ::std::os::raw::c_char,
            valuestrNeedBig: *mut ::std::os::raw::c_char,
            is_set: bool,
        ) -> bool,
    >,
    pub GetSetEnvelopeInfo_String: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            parmname: *const ::std::os::raw::c_char,
            stringNeedBig: *mut ::std::os::raw::c_char,
            setNewValue: bool,
        ) -> bool,
    >,
    pub GetSetEnvelopeState: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            str: *mut ::std::os::raw::c_char,
            str_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetSetEnvelopeState2: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            str: *mut ::std::os::raw::c_char,
            str_sz: ::std::os::raw::c_int,
            isundo: bool,
        ) -> bool,
    >,
    pub GetSetItemState: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            str: *mut ::std::os::raw::c_char,
            str_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetSetItemState2: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            str: *mut ::std::os::raw::c_char,
            str_sz: ::std::os::raw::c_int,
            isundo: bool,
        ) -> bool,
    >,
    pub GetSetMediaItemInfo: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            parmname: *const ::std::os::raw::c_char,
            setNewValue: *mut ::std::os::raw::c_void,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub GetSetMediaItemInfo_String: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            parmname: *const ::std::os::raw::c_char,
            stringNeedBig: *mut ::std::os::raw::c_char,
            setNewValue: bool,
        ) -> bool,
    >,
    pub GetSetMediaItemTakeInfo: Option<
        unsafe extern "C" fn(
            tk: *mut root::MediaItem_Take,
            parmname: *const ::std::os::raw::c_char,
            setNewValue: *mut ::std::os::raw::c_void,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub GetSetMediaItemTakeInfo_String: Option<
        unsafe extern "C" fn(
            tk: *mut root::MediaItem_Take,
            parmname: *const ::std::os::raw::c_char,
            stringNeedBig: *mut ::std::os::raw::c_char,
            setNewValue: bool,
        ) -> bool,
    >,
    pub GetSetMediaTrackInfo: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            parmname: *const ::std::os::raw::c_char,
            setNewValue: *mut ::std::os::raw::c_void,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub GetSetMediaTrackInfo_String: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            parmname: *const ::std::os::raw::c_char,
            stringNeedBig: *mut ::std::os::raw::c_char,
            setNewValue: bool,
        ) -> bool,
    >,
    pub GetSetObjectState: Option<
        unsafe extern "C" fn(
            obj: *mut ::std::os::raw::c_void,
            str: *const ::std::os::raw::c_char,
        ) -> *mut ::std::os::raw::c_char,
    >,
    pub GetSetObjectState2: Option<
        unsafe extern "C" fn(
            obj: *mut ::std::os::raw::c_void,
            str: *const ::std::os::raw::c_char,
            isundo: bool,
        ) -> *mut ::std::os::raw::c_char,
    >,
    pub GetSetProjectAuthor: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            set: bool,
            author: *mut ::std::os::raw::c_char,
            author_sz: ::std::os::raw::c_int,
        ),
    >,
    pub GetSetProjectGrid: Option<
        unsafe extern "C" fn(
            project: *mut root::ReaProject,
            set: bool,
            divisionInOutOptional: *mut f64,
            swingmodeInOutOptional: *mut ::std::os::raw::c_int,
            swingamtInOutOptional: *mut f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetSetProjectInfo: Option<
        unsafe extern "C" fn(
            project: *mut root::ReaProject,
            desc: *const ::std::os::raw::c_char,
            value: f64,
            is_set: bool,
        ) -> f64,
    >,
    pub GetSetProjectInfo_String: Option<
        unsafe extern "C" fn(
            project: *mut root::ReaProject,
            desc: *const ::std::os::raw::c_char,
            valuestrNeedBig: *mut ::std::os::raw::c_char,
            is_set: bool,
        ) -> bool,
    >,
    pub GetSetProjectNotes: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            set: bool,
            notesNeedBig: *mut ::std::os::raw::c_char,
            notesNeedBig_sz: ::std::os::raw::c_int,
        ),
    >,
    pub GetSetRepeat:
        Option<unsafe extern "C" fn(val: ::std::os::raw::c_int) -> ::std::os::raw::c_int>,
    pub GetSetRepeatEx: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            val: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetSetTrackGroupMembership: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            groupname: *const ::std::os::raw::c_char,
            setmask: ::std::os::raw::c_uint,
            setvalue: ::std::os::raw::c_uint,
        ) -> ::std::os::raw::c_uint,
    >,
    pub GetSetTrackGroupMembershipHigh: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            groupname: *const ::std::os::raw::c_char,
            setmask: ::std::os::raw::c_uint,
            setvalue: ::std::os::raw::c_uint,
        ) -> ::std::os::raw::c_uint,
    >,
    pub GetSetTrackMIDISupportFile: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            track: *mut root::MediaTrack,
            which: ::std::os::raw::c_int,
            filename: *const ::std::os::raw::c_char,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub GetSetTrackSendInfo: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            category: ::std::os::raw::c_int,
            sendidx: ::std::os::raw::c_int,
            parmname: *const ::std::os::raw::c_char,
            setNewValue: *mut ::std::os::raw::c_void,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub GetSetTrackSendInfo_String: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            category: ::std::os::raw::c_int,
            sendidx: ::std::os::raw::c_int,
            parmname: *const ::std::os::raw::c_char,
            stringNeedBig: *mut ::std::os::raw::c_char,
            setNewValue: bool,
        ) -> bool,
    >,
    pub GetSetTrackState: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            str: *mut ::std::os::raw::c_char,
            str_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetSetTrackState2: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            str: *mut ::std::os::raw::c_char,
            str_sz: ::std::os::raw::c_int,
            isundo: bool,
        ) -> bool,
    >,
    pub GetSubProjectFromSource:
        Option<unsafe extern "C" fn(src: *mut root::PCM_source) -> *mut root::ReaProject>,
    pub GetTake: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            takeidx: ::std::os::raw::c_int,
        ) -> *mut root::MediaItem_Take,
    >,
    pub GetTakeEnvelope: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            envidx: ::std::os::raw::c_int,
        ) -> *mut root::TrackEnvelope,
    >,
    pub GetTakeEnvelopeByName: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            envname: *const ::std::os::raw::c_char,
        ) -> *mut root::TrackEnvelope,
    >,
    pub GetTakeName: Option<
        unsafe extern "C" fn(take: *mut root::MediaItem_Take) -> *const ::std::os::raw::c_char,
    >,
    pub GetTakeNumStretchMarkers:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take) -> ::std::os::raw::c_int>,
    pub GetTakeStretchMarker: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            idx: ::std::os::raw::c_int,
            posOut: *mut f64,
            srcposOutOptional: *mut f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetTakeStretchMarkerSlope: Option<
        unsafe extern "C" fn(take: *mut root::MediaItem_Take, idx: ::std::os::raw::c_int) -> f64,
    >,
    pub GetTCPFXParm: Option<
        unsafe extern "C" fn(
            project: *mut root::ReaProject,
            track: *mut root::MediaTrack,
            index: ::std::os::raw::c_int,
            fxindexOut: *mut ::std::os::raw::c_int,
            parmidxOut: *mut ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetTempoMatchPlayRate: Option<
        unsafe extern "C" fn(
            source: *mut root::PCM_source,
            srcscale: f64,
            position: f64,
            mult: f64,
            rateOut: *mut f64,
            targetlenOut: *mut f64,
        ) -> bool,
    >,
    pub GetTempoTimeSigMarker: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            ptidx: ::std::os::raw::c_int,
            timeposOut: *mut f64,
            measureposOut: *mut ::std::os::raw::c_int,
            beatposOut: *mut f64,
            bpmOut: *mut f64,
            timesig_numOut: *mut ::std::os::raw::c_int,
            timesig_denomOut: *mut ::std::os::raw::c_int,
            lineartempoOut: *mut bool,
        ) -> bool,
    >,
    pub GetToggleCommandState:
        Option<unsafe extern "C" fn(command_id: ::std::os::raw::c_int) -> ::std::os::raw::c_int>,
    pub GetToggleCommandState2: Option<
        unsafe extern "C" fn(
            section: *mut root::KbdSectionInfo,
            command_id: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetToggleCommandStateEx: Option<
        unsafe extern "C" fn(
            section_id: ::std::os::raw::c_int,
            command_id: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetToggleCommandStateThroughHooks: Option<
        unsafe extern "C" fn(
            section: *mut root::KbdSectionInfo,
            command_id: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetTooltipWindow: Option<unsafe extern "C" fn() -> root::HWND>,
    pub GetTrack: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            trackidx: ::std::os::raw::c_int,
        ) -> *mut root::MediaTrack,
    >,
    pub GetTrackAutomationMode:
        Option<unsafe extern "C" fn(tr: *mut root::MediaTrack) -> ::std::os::raw::c_int>,
    pub GetTrackColor:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack) -> ::std::os::raw::c_int>,
    pub GetTrackDepth:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack) -> ::std::os::raw::c_int>,
    pub GetTrackEnvelope: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            envidx: ::std::os::raw::c_int,
        ) -> *mut root::TrackEnvelope,
    >,
    pub GetTrackEnvelopeByChunkName: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            cfgchunkname: *const ::std::os::raw::c_char,
        ) -> *mut root::TrackEnvelope,
    >,
    pub GetTrackEnvelopeByName: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            envname: *const ::std::os::raw::c_char,
        ) -> *mut root::TrackEnvelope,
    >,
    pub GetTrackFromPoint: Option<
        unsafe extern "C" fn(
            screen_x: ::std::os::raw::c_int,
            screen_y: ::std::os::raw::c_int,
            infoOutOptional: *mut ::std::os::raw::c_int,
        ) -> *mut root::MediaTrack,
    >,
    pub GetTrackGUID: Option<unsafe extern "C" fn(tr: *mut root::MediaTrack) -> *mut root::GUID>,
    pub GetTrackInfo: Option<
        unsafe extern "C" fn(
            track: root::INT_PTR,
            flags: *mut ::std::os::raw::c_int,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub GetTrackMediaItem: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            itemidx: ::std::os::raw::c_int,
        ) -> *mut root::MediaItem,
    >,
    pub GetTrackMIDILyrics: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            flag: ::std::os::raw::c_int,
            bufWantNeedBig: *mut ::std::os::raw::c_char,
            bufWantNeedBig_sz: *mut ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetTrackMIDINoteName: Option<
        unsafe extern "C" fn(
            track: ::std::os::raw::c_int,
            pitch: ::std::os::raw::c_int,
            chan: ::std::os::raw::c_int,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub GetTrackMIDINoteNameEx: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            track: *mut root::MediaTrack,
            pitch: ::std::os::raw::c_int,
            chan: ::std::os::raw::c_int,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub GetTrackMIDINoteRange: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            track: *mut root::MediaTrack,
            note_loOut: *mut ::std::os::raw::c_int,
            note_hiOut: *mut ::std::os::raw::c_int,
        ),
    >,
    pub GetTrackName: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            bufOut: *mut ::std::os::raw::c_char,
            bufOut_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetTrackNumMediaItems:
        Option<unsafe extern "C" fn(tr: *mut root::MediaTrack) -> ::std::os::raw::c_int>,
    pub GetTrackNumSends: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            category: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub GetTrackReceiveName: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            recv_index: ::std::os::raw::c_int,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetTrackReceiveUIMute: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            recv_index: ::std::os::raw::c_int,
            muteOut: *mut bool,
        ) -> bool,
    >,
    pub GetTrackReceiveUIVolPan: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            recv_index: ::std::os::raw::c_int,
            volumeOut: *mut f64,
            panOut: *mut f64,
        ) -> bool,
    >,
    pub GetTrackSendInfo_Value: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            category: ::std::os::raw::c_int,
            sendidx: ::std::os::raw::c_int,
            parmname: *const ::std::os::raw::c_char,
        ) -> f64,
    >,
    pub GetTrackSendName: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            send_index: ::std::os::raw::c_int,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetTrackSendUIMute: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            send_index: ::std::os::raw::c_int,
            muteOut: *mut bool,
        ) -> bool,
    >,
    pub GetTrackSendUIVolPan: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            send_index: ::std::os::raw::c_int,
            volumeOut: *mut f64,
            panOut: *mut f64,
        ) -> bool,
    >,
    pub GetTrackState: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            flagsOut: *mut ::std::os::raw::c_int,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub GetTrackStateChunk: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            strNeedBig: *mut ::std::os::raw::c_char,
            strNeedBig_sz: ::std::os::raw::c_int,
            isundoOptional: bool,
        ) -> bool,
    >,
    pub GetTrackUIMute:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack, muteOut: *mut bool) -> bool>,
    pub GetTrackUIPan: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            pan1Out: *mut f64,
            pan2Out: *mut f64,
            panmodeOut: *mut ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetTrackUIVolPan: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            volumeOut: *mut f64,
            panOut: *mut f64,
        ) -> bool,
    >,
    pub GetUnderrunTime: Option<
        unsafe extern "C" fn(
            audio_xrunOutOptional: *mut ::std::os::raw::c_uint,
            media_xrunOutOptional: *mut ::std::os::raw::c_uint,
            curtimeOutOptional: *mut ::std::os::raw::c_uint,
        ),
    >,
    pub GetUserFileNameForRead: Option<
        unsafe extern "C" fn(
            filenameNeed4096: *mut ::std::os::raw::c_char,
            title: *const ::std::os::raw::c_char,
            defext: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub GetUserInputs: Option<
        unsafe extern "C" fn(
            title: *const ::std::os::raw::c_char,
            num_inputs: ::std::os::raw::c_int,
            captions_csv: *const ::std::os::raw::c_char,
            retvals_csv: *mut ::std::os::raw::c_char,
            retvals_csv_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GoToMarker: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            marker_index: ::std::os::raw::c_int,
            use_timeline_order: bool,
        ),
    >,
    pub GoToRegion: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            region_index: ::std::os::raw::c_int,
            use_timeline_order: bool,
        ),
    >,
    pub GR_SelectColor: Option<
        unsafe extern "C" fn(
            hwnd: root::HWND,
            colorOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub GSC_mainwnd:
        Option<unsafe extern "C" fn(t: ::std::os::raw::c_int) -> ::std::os::raw::c_int>,
    pub guidToString:
        Option<unsafe extern "C" fn(g: *const root::GUID, destNeed64: *mut ::std::os::raw::c_char)>,
    pub HasExtState: Option<
        unsafe extern "C" fn(
            section: *const ::std::os::raw::c_char,
            key: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub HasTrackMIDIPrograms:
        Option<unsafe extern "C" fn(track: ::std::os::raw::c_int) -> *const ::std::os::raw::c_char>,
    pub HasTrackMIDIProgramsEx: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            track: *mut root::MediaTrack,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub Help_Set: Option<
        unsafe extern "C" fn(helpstring: *const ::std::os::raw::c_char, is_temporary_help: bool),
    >,
    pub HiresPeaksFromSource: Option<
        unsafe extern "C" fn(
            src: *mut root::PCM_source,
            block: *mut root::PCM_source_peaktransfer_t,
        ),
    >,
    pub image_resolve_fn: Option<
        unsafe extern "C" fn(
            in_: *const ::std::os::raw::c_char,
            out: *mut ::std::os::raw::c_char,
            out_sz: ::std::os::raw::c_int,
        ),
    >,
    pub InsertAutomationItem: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            pool_id: ::std::os::raw::c_int,
            position: f64,
            length: f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub InsertEnvelopePoint: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            time: f64,
            value: f64,
            shape: ::std::os::raw::c_int,
            tension: f64,
            selected: bool,
            noSortInOptional: *mut bool,
        ) -> bool,
    >,
    pub InsertEnvelopePointEx: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            autoitem_idx: ::std::os::raw::c_int,
            time: f64,
            value: f64,
            shape: ::std::os::raw::c_int,
            tension: f64,
            selected: bool,
            noSortInOptional: *mut bool,
        ) -> bool,
    >,
    pub InsertMedia: Option<
        unsafe extern "C" fn(
            file: *const ::std::os::raw::c_char,
            mode: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub InsertMediaSection: Option<
        unsafe extern "C" fn(
            file: *const ::std::os::raw::c_char,
            mode: ::std::os::raw::c_int,
            startpct: f64,
            endpct: f64,
            pitchshift: f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub InsertTrackAtIndex:
        Option<unsafe extern "C" fn(idx: ::std::os::raw::c_int, wantDefaults: bool)>,
    pub IsInRealTimeAudio: Option<unsafe extern "C" fn() -> ::std::os::raw::c_int>,
    pub IsItemTakeActiveForPlayback: Option<
        unsafe extern "C" fn(item: *mut root::MediaItem, take: *mut root::MediaItem_Take) -> bool,
    >,
    pub IsMediaExtension:
        Option<unsafe extern "C" fn(ext: *const ::std::os::raw::c_char, wantOthers: bool) -> bool>,
    pub IsMediaItemSelected: Option<unsafe extern "C" fn(item: *mut root::MediaItem) -> bool>,
    pub IsProjectDirty:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> ::std::os::raw::c_int>,
    pub IsREAPER: Option<unsafe extern "C" fn() -> bool>,
    pub IsTrackSelected: Option<unsafe extern "C" fn(track: *mut root::MediaTrack) -> bool>,
    pub IsTrackVisible:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack, mixer: bool) -> bool>,
    pub joystick_create: Option<
        unsafe extern "C" fn(
            guid: *const root::GUID,
        ) -> *mut root::reaper_functions::joystick_device,
    >,
    pub joystick_destroy:
        Option<unsafe extern "C" fn(device: *mut root::reaper_functions::joystick_device)>,
    pub joystick_enum: Option<
        unsafe extern "C" fn(
            index: ::std::os::raw::c_int,
            namestrOutOptional: *mut *const ::std::os::raw::c_char,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub joystick_getaxis: Option<
        unsafe extern "C" fn(
            dev: *mut root::reaper_functions::joystick_device,
            axis: ::std::os::raw::c_int,
        ) -> f64,
    >,
    pub joystick_getbuttonmask: Option<
        unsafe extern "C" fn(
            dev: *mut root::reaper_functions::joystick_device,
        ) -> ::std::os::raw::c_uint,
    >,
    pub joystick_getinfo: Option<
        unsafe extern "C" fn(
            dev: *mut root::reaper_functions::joystick_device,
            axesOutOptional: *mut ::std::os::raw::c_int,
            povsOutOptional: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub joystick_getpov: Option<
        unsafe extern "C" fn(
            dev: *mut root::reaper_functions::joystick_device,
            pov: ::std::os::raw::c_int,
        ) -> f64,
    >,
    pub joystick_update:
        Option<unsafe extern "C" fn(dev: *mut root::reaper_functions::joystick_device) -> bool>,
    pub kbd_enumerateActions: Option<
        unsafe extern "C" fn(
            section: *mut root::KbdSectionInfo,
            idx: ::std::os::raw::c_int,
            nameOut: *mut *const ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int,
    >,
    pub kbd_formatKeyName:
        Option<unsafe extern "C" fn(ac: *mut root::ACCEL, s: *mut ::std::os::raw::c_char)>,
    pub kbd_getCommandName: Option<
        unsafe extern "C" fn(
            cmd: ::std::os::raw::c_int,
            s: *mut ::std::os::raw::c_char,
            section: *mut root::KbdSectionInfo,
        ),
    >,
    pub kbd_getTextFromCmd: Option<
        unsafe extern "C" fn(
            cmd: root::DWORD,
            section: *mut root::KbdSectionInfo,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub KBD_OnMainActionEx: Option<
        unsafe extern "C" fn(
            cmd: ::std::os::raw::c_int,
            val: ::std::os::raw::c_int,
            valhw: ::std::os::raw::c_int,
            relmode: ::std::os::raw::c_int,
            hwnd: root::HWND,
            proj: *mut root::ReaProject,
        ) -> ::std::os::raw::c_int,
    >,
    pub kbd_OnMidiEvent: Option<
        unsafe extern "C" fn(evt: *mut root::MIDI_event_t, dev_index: ::std::os::raw::c_int),
    >,
    pub kbd_OnMidiList: Option<
        unsafe extern "C" fn(list: *mut root::MIDI_eventlist, dev_index: ::std::os::raw::c_int),
    >,
    pub kbd_ProcessActionsMenu:
        Option<unsafe extern "C" fn(menu: root::HMENU, section: *mut root::KbdSectionInfo)>,
    pub kbd_processMidiEventActionEx: Option<
        unsafe extern "C" fn(
            evt: *mut root::MIDI_event_t,
            section: *mut root::KbdSectionInfo,
            hwndCtx: root::HWND,
        ) -> bool,
    >,
    pub kbd_reprocessMenu:
        Option<unsafe extern "C" fn(menu: root::HMENU, section: *mut root::KbdSectionInfo)>,
    pub kbd_RunCommandThroughHooks: Option<
        unsafe extern "C" fn(
            section: *mut root::KbdSectionInfo,
            actionCommandID: *mut ::std::os::raw::c_int,
            val: *mut ::std::os::raw::c_int,
            valhw: *mut ::std::os::raw::c_int,
            relmode: *mut ::std::os::raw::c_int,
            hwnd: root::HWND,
        ) -> bool,
    >,
    pub kbd_translateAccelerator: Option<
        unsafe extern "C" fn(
            hwnd: root::HWND,
            msg: *mut root::MSG,
            section: *mut root::KbdSectionInfo,
        ) -> ::std::os::raw::c_int,
    >,
    pub kbd_translateMouse: Option<
        unsafe extern "C" fn(
            winmsg: *mut ::std::os::raw::c_void,
            midimsg: *mut ::std::os::raw::c_uchar,
        ) -> bool,
    >,
    pub LICE__Destroy: Option<unsafe extern "C" fn(bm: *mut root::reaper_functions::LICE_IBitmap)>,
    pub LICE__DestroyFont:
        Option<unsafe extern "C" fn(font: *mut root::reaper_functions::LICE_IFont)>,
    pub LICE__DrawText: Option<
        unsafe extern "C" fn(
            font: *mut root::reaper_functions::LICE_IFont,
            bm: *mut root::reaper_functions::LICE_IBitmap,
            str: *const ::std::os::raw::c_char,
            strcnt: ::std::os::raw::c_int,
            rect: *mut root::RECT,
            dtFlags: root::UINT,
        ) -> ::std::os::raw::c_int,
    >,
    pub LICE__GetBits: Option<
        unsafe extern "C" fn(
            bm: *mut root::reaper_functions::LICE_IBitmap,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub LICE__GetDC:
        Option<unsafe extern "C" fn(bm: *mut root::reaper_functions::LICE_IBitmap) -> root::HDC>,
    pub LICE__GetHeight: Option<
        unsafe extern "C" fn(
            bm: *mut root::reaper_functions::LICE_IBitmap,
        ) -> ::std::os::raw::c_int,
    >,
    pub LICE__GetRowSpan: Option<
        unsafe extern "C" fn(
            bm: *mut root::reaper_functions::LICE_IBitmap,
        ) -> ::std::os::raw::c_int,
    >,
    pub LICE__GetWidth: Option<
        unsafe extern "C" fn(
            bm: *mut root::reaper_functions::LICE_IBitmap,
        ) -> ::std::os::raw::c_int,
    >,
    pub LICE__IsFlipped:
        Option<unsafe extern "C" fn(bm: *mut root::reaper_functions::LICE_IBitmap) -> bool>,
    pub LICE__resize: Option<
        unsafe extern "C" fn(
            bm: *mut root::reaper_functions::LICE_IBitmap,
            w: ::std::os::raw::c_int,
            h: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub LICE__SetBkColor: Option<
        unsafe extern "C" fn(
            font: *mut root::reaper_functions::LICE_IFont,
            color: root::reaper_functions::LICE_pixel,
        ) -> root::reaper_functions::LICE_pixel,
    >,
    pub LICE__SetFromHFont: Option<
        unsafe extern "C" fn(
            font: *mut root::reaper_functions::LICE_IFont,
            hfont: root::HFONT,
            flags: ::std::os::raw::c_int,
        ),
    >,
    pub LICE__SetTextColor: Option<
        unsafe extern "C" fn(
            font: *mut root::reaper_functions::LICE_IFont,
            color: root::reaper_functions::LICE_pixel,
        ) -> root::reaper_functions::LICE_pixel,
    >,
    pub LICE__SetTextCombineMode: Option<
        unsafe extern "C" fn(
            ifont: *mut root::reaper_functions::LICE_IFont,
            mode: ::std::os::raw::c_int,
            alpha: f32,
        ),
    >,
    pub LICE_Arc: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            cx: f32,
            cy: f32,
            r: f32,
            minAngle: f32,
            maxAngle: f32,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
            aa: bool,
        ),
    >,
    pub LICE_Blit: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            src: *mut root::reaper_functions::LICE_IBitmap,
            dstx: ::std::os::raw::c_int,
            dsty: ::std::os::raw::c_int,
            srcx: ::std::os::raw::c_int,
            srcy: ::std::os::raw::c_int,
            srcw: ::std::os::raw::c_int,
            srch: ::std::os::raw::c_int,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_Blur: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            src: *mut root::reaper_functions::LICE_IBitmap,
            dstx: ::std::os::raw::c_int,
            dsty: ::std::os::raw::c_int,
            srcx: ::std::os::raw::c_int,
            srcy: ::std::os::raw::c_int,
            srcw: ::std::os::raw::c_int,
            srch: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_BorderedRect: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            w: ::std::os::raw::c_int,
            h: ::std::os::raw::c_int,
            bgcolor: root::reaper_functions::LICE_pixel,
            fgcolor: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_Circle: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            cx: f32,
            cy: f32,
            r: f32,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
            aa: bool,
        ),
    >,
    pub LICE_Clear: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            color: root::reaper_functions::LICE_pixel,
        ),
    >,
    pub LICE_ClearRect: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            w: ::std::os::raw::c_int,
            h: ::std::os::raw::c_int,
            mask: root::reaper_functions::LICE_pixel,
            orbits: root::reaper_functions::LICE_pixel,
        ),
    >,
    pub LICE_ClipLine: Option<
        unsafe extern "C" fn(
            pX1Out: *mut ::std::os::raw::c_int,
            pY1Out: *mut ::std::os::raw::c_int,
            pX2Out: *mut ::std::os::raw::c_int,
            pY2Out: *mut ::std::os::raw::c_int,
            xLo: ::std::os::raw::c_int,
            yLo: ::std::os::raw::c_int,
            xHi: ::std::os::raw::c_int,
            yHi: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub LICE_Copy: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            src: *mut root::reaper_functions::LICE_IBitmap,
        ),
    >,
    pub LICE_CreateBitmap: Option<
        unsafe extern "C" fn(
            mode: ::std::os::raw::c_int,
            w: ::std::os::raw::c_int,
            h: ::std::os::raw::c_int,
        ) -> *mut root::reaper_functions::LICE_IBitmap,
    >,
    pub LICE_CreateFont: Option<unsafe extern "C" fn() -> *mut root::reaper_functions::LICE_IFont>,
    pub LICE_DrawCBezier: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            xstart: f64,
            ystart: f64,
            xctl1: f64,
            yctl1: f64,
            xctl2: f64,
            yctl2: f64,
            xend: f64,
            yend: f64,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
            aa: bool,
            tol: f64,
        ),
    >,
    pub LICE_DrawChar: Option<
        unsafe extern "C" fn(
            bm: *mut root::reaper_functions::LICE_IBitmap,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            c: ::std::os::raw::c_char,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_DrawGlyph: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            color: root::reaper_functions::LICE_pixel,
            alphas: *mut root::reaper_functions::LICE_pixel_chan,
            glyph_w: ::std::os::raw::c_int,
            glyph_h: ::std::os::raw::c_int,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_DrawRect: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            w: ::std::os::raw::c_int,
            h: ::std::os::raw::c_int,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_DrawText: Option<
        unsafe extern "C" fn(
            bm: *mut root::reaper_functions::LICE_IBitmap,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            string: *const ::std::os::raw::c_char,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_FillCBezier: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            xstart: f64,
            ystart: f64,
            xctl1: f64,
            yctl1: f64,
            xctl2: f64,
            yctl2: f64,
            xend: f64,
            yend: f64,
            yfill: ::std::os::raw::c_int,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
            aa: bool,
            tol: f64,
        ),
    >,
    pub LICE_FillCircle: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            cx: f32,
            cy: f32,
            r: f32,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
            aa: bool,
        ),
    >,
    pub LICE_FillConvexPolygon: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x: *mut ::std::os::raw::c_int,
            y: *mut ::std::os::raw::c_int,
            npoints: ::std::os::raw::c_int,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_FillRect: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            w: ::std::os::raw::c_int,
            h: ::std::os::raw::c_int,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_FillTrapezoid: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x1a: ::std::os::raw::c_int,
            x1b: ::std::os::raw::c_int,
            y1: ::std::os::raw::c_int,
            x2a: ::std::os::raw::c_int,
            x2b: ::std::os::raw::c_int,
            y2: ::std::os::raw::c_int,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_FillTriangle: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x1: ::std::os::raw::c_int,
            y1: ::std::os::raw::c_int,
            x2: ::std::os::raw::c_int,
            y2: ::std::os::raw::c_int,
            x3: ::std::os::raw::c_int,
            y3: ::std::os::raw::c_int,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_GetPixel: Option<
        unsafe extern "C" fn(
            bm: *mut root::reaper_functions::LICE_IBitmap,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
        ) -> root::reaper_functions::LICE_pixel,
    >,
    pub LICE_GradRect: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            dstx: ::std::os::raw::c_int,
            dsty: ::std::os::raw::c_int,
            dstw: ::std::os::raw::c_int,
            dsth: ::std::os::raw::c_int,
            ir: f32,
            ig: f32,
            ib: f32,
            ia: f32,
            drdx: f32,
            dgdx: f32,
            dbdx: f32,
            dadx: f32,
            drdy: f32,
            dgdy: f32,
            dbdy: f32,
            dady: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_Line: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x1: f32,
            y1: f32,
            x2: f32,
            y2: f32,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
            aa: bool,
        ),
    >,
    pub LICE_LineInt: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x1: ::std::os::raw::c_int,
            y1: ::std::os::raw::c_int,
            x2: ::std::os::raw::c_int,
            y2: ::std::os::raw::c_int,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
            aa: bool,
        ),
    >,
    pub LICE_LoadPNG: Option<
        unsafe extern "C" fn(
            filename: *const ::std::os::raw::c_char,
            bmp: *mut root::reaper_functions::LICE_IBitmap,
        ) -> *mut root::reaper_functions::LICE_IBitmap,
    >,
    pub LICE_LoadPNGFromResource: Option<
        unsafe extern "C" fn(
            hInst: root::HINSTANCE,
            resid: *const ::std::os::raw::c_char,
            bmp: *mut root::reaper_functions::LICE_IBitmap,
        ) -> *mut root::reaper_functions::LICE_IBitmap,
    >,
    pub LICE_MeasureText: Option<
        unsafe extern "C" fn(
            string: *const ::std::os::raw::c_char,
            w: *mut ::std::os::raw::c_int,
            h: *mut ::std::os::raw::c_int,
        ),
    >,
    pub LICE_MultiplyAddRect: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            w: ::std::os::raw::c_int,
            h: ::std::os::raw::c_int,
            rsc: f32,
            gsc: f32,
            bsc: f32,
            asc: f32,
            radd: f32,
            gadd: f32,
            badd: f32,
            aadd: f32,
        ),
    >,
    pub LICE_PutPixel: Option<
        unsafe extern "C" fn(
            bm: *mut root::reaper_functions::LICE_IBitmap,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            color: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_RotatedBlit: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            src: *mut root::reaper_functions::LICE_IBitmap,
            dstx: ::std::os::raw::c_int,
            dsty: ::std::os::raw::c_int,
            dstw: ::std::os::raw::c_int,
            dsth: ::std::os::raw::c_int,
            srcx: f32,
            srcy: f32,
            srcw: f32,
            srch: f32,
            angle: f32,
            cliptosourcerect: bool,
            alpha: f32,
            mode: ::std::os::raw::c_int,
            rotxcent: f32,
            rotycent: f32,
        ),
    >,
    pub LICE_RoundRect: Option<
        unsafe extern "C" fn(
            drawbm: *mut root::reaper_functions::LICE_IBitmap,
            xpos: f32,
            ypos: f32,
            w: f32,
            h: f32,
            cornerradius: ::std::os::raw::c_int,
            col: root::reaper_functions::LICE_pixel,
            alpha: f32,
            mode: ::std::os::raw::c_int,
            aa: bool,
        ),
    >,
    pub LICE_ScaledBlit: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            src: *mut root::reaper_functions::LICE_IBitmap,
            dstx: ::std::os::raw::c_int,
            dsty: ::std::os::raw::c_int,
            dstw: ::std::os::raw::c_int,
            dsth: ::std::os::raw::c_int,
            srcx: f32,
            srcy: f32,
            srcw: f32,
            srch: f32,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ),
    >,
    pub LICE_SimpleFill: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            newcolor: root::reaper_functions::LICE_pixel,
            comparemask: root::reaper_functions::LICE_pixel,
            keepmask: root::reaper_functions::LICE_pixel,
        ),
    >,
    pub Loop_OnArrow: Option<
        unsafe extern "C" fn(
            project: *mut root::ReaProject,
            direction: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub Main_OnCommand:
        Option<unsafe extern "C" fn(command: ::std::os::raw::c_int, flag: ::std::os::raw::c_int)>,
    pub Main_OnCommandEx: Option<
        unsafe extern "C" fn(
            command: ::std::os::raw::c_int,
            flag: ::std::os::raw::c_int,
            proj: *mut root::ReaProject,
        ),
    >,
    pub Main_openProject: Option<unsafe extern "C" fn(name: *const ::std::os::raw::c_char)>,
    pub Main_SaveProject:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject, forceSaveAsInOptional: bool)>,
    pub Main_UpdateLoopInfo: Option<unsafe extern "C" fn(ignoremask: ::std::os::raw::c_int)>,
    pub MarkProjectDirty: Option<unsafe extern "C" fn(proj: *mut root::ReaProject)>,
    pub MarkTrackItemsDirty:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack, item: *mut root::MediaItem)>,
    pub Master_GetPlayRate: Option<unsafe extern "C" fn(project: *mut root::ReaProject) -> f64>,
    pub Master_GetPlayRateAtTime:
        Option<unsafe extern "C" fn(time_s: f64, proj: *mut root::ReaProject) -> f64>,
    pub Master_GetTempo: Option<unsafe extern "C" fn() -> f64>,
    pub Master_NormalizePlayRate:
        Option<unsafe extern "C" fn(playrate: f64, isnormalized: bool) -> f64>,
    pub Master_NormalizeTempo: Option<unsafe extern "C" fn(bpm: f64, isnormalized: bool) -> f64>,
    pub MB: Option<
        unsafe extern "C" fn(
            msg: *const ::std::os::raw::c_char,
            title: *const ::std::os::raw::c_char,
            type_: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub MediaItemDescendsFromTrack: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            track: *mut root::MediaTrack,
        ) -> ::std::os::raw::c_int,
    >,
    pub MIDI_CountEvts: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            notecntOut: *mut ::std::os::raw::c_int,
            ccevtcntOut: *mut ::std::os::raw::c_int,
            textsyxevtcntOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub MIDI_DeleteCC: Option<
        unsafe extern "C" fn(take: *mut root::MediaItem_Take, ccidx: ::std::os::raw::c_int) -> bool,
    >,
    pub MIDI_DeleteEvt: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            evtidx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_DeleteNote: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            noteidx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_DeleteTextSysexEvt: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            textsyxevtidx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_DisableSort: Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take)>,
    pub MIDI_EnumSelCC: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            ccidx: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub MIDI_EnumSelEvts: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            evtidx: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub MIDI_EnumSelNotes: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            noteidx: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub MIDI_EnumSelTextSysexEvts: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            textsyxidx: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub MIDI_eventlist_Create: Option<unsafe extern "C" fn() -> *mut root::MIDI_eventlist>,
    pub MIDI_eventlist_Destroy: Option<unsafe extern "C" fn(evtlist: *mut root::MIDI_eventlist)>,
    pub MIDI_GetAllEvts: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            bufNeedBig: *mut ::std::os::raw::c_char,
            bufNeedBig_sz: *mut ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_GetCC: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            ccidx: ::std::os::raw::c_int,
            selectedOut: *mut bool,
            mutedOut: *mut bool,
            ppqposOut: *mut f64,
            chanmsgOut: *mut ::std::os::raw::c_int,
            chanOut: *mut ::std::os::raw::c_int,
            msg2Out: *mut ::std::os::raw::c_int,
            msg3Out: *mut ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_GetCCShape: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            ccidx: ::std::os::raw::c_int,
            shapeOut: *mut ::std::os::raw::c_int,
            beztensionOut: *mut f64,
        ) -> bool,
    >,
    pub MIDI_GetEvt: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            evtidx: ::std::os::raw::c_int,
            selectedOut: *mut bool,
            mutedOut: *mut bool,
            ppqposOut: *mut f64,
            msg: *mut ::std::os::raw::c_char,
            msg_sz: *mut ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_GetGrid: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            swingOutOptional: *mut f64,
            noteLenOutOptional: *mut f64,
        ) -> f64,
    >,
    pub MIDI_GetHash: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            notesonly: bool,
            hash: *mut ::std::os::raw::c_char,
            hash_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_GetNote: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            noteidx: ::std::os::raw::c_int,
            selectedOut: *mut bool,
            mutedOut: *mut bool,
            startppqposOut: *mut f64,
            endppqposOut: *mut f64,
            chanOut: *mut ::std::os::raw::c_int,
            pitchOut: *mut ::std::os::raw::c_int,
            velOut: *mut ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_GetPPQPos_EndOfMeasure:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take, ppqpos: f64) -> f64>,
    pub MIDI_GetPPQPos_StartOfMeasure:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take, ppqpos: f64) -> f64>,
    pub MIDI_GetPPQPosFromProjQN:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take, projqn: f64) -> f64>,
    pub MIDI_GetPPQPosFromProjTime:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take, projtime: f64) -> f64>,
    pub MIDI_GetProjQNFromPPQPos:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take, ppqpos: f64) -> f64>,
    pub MIDI_GetProjTimeFromPPQPos:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take, ppqpos: f64) -> f64>,
    pub MIDI_GetScale: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            rootOut: *mut ::std::os::raw::c_int,
            scaleOut: *mut ::std::os::raw::c_int,
            name: *mut ::std::os::raw::c_char,
            name_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_GetTextSysexEvt: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            textsyxevtidx: ::std::os::raw::c_int,
            selectedOutOptional: *mut bool,
            mutedOutOptional: *mut bool,
            ppqposOutOptional: *mut f64,
            typeOutOptional: *mut ::std::os::raw::c_int,
            msgOptional: *mut ::std::os::raw::c_char,
            msgOptional_sz: *mut ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_GetTrackHash: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            notesonly: bool,
            hash: *mut ::std::os::raw::c_char,
            hash_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_InsertCC: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            selected: bool,
            muted: bool,
            ppqpos: f64,
            chanmsg: ::std::os::raw::c_int,
            chan: ::std::os::raw::c_int,
            msg2: ::std::os::raw::c_int,
            msg3: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_InsertEvt: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            selected: bool,
            muted: bool,
            ppqpos: f64,
            bytestr: *const ::std::os::raw::c_char,
            bytestr_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_InsertNote: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            selected: bool,
            muted: bool,
            startppqpos: f64,
            endppqpos: f64,
            chan: ::std::os::raw::c_int,
            pitch: ::std::os::raw::c_int,
            vel: ::std::os::raw::c_int,
            noSortInOptional: *const bool,
        ) -> bool,
    >,
    pub MIDI_InsertTextSysexEvt: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            selected: bool,
            muted: bool,
            ppqpos: f64,
            type_: ::std::os::raw::c_int,
            bytestr: *const ::std::os::raw::c_char,
            bytestr_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub midi_reinit: Option<unsafe extern "C" fn()>,
    pub MIDI_SelectAll: Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take, select: bool)>,
    pub MIDI_SetAllEvts: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            buf: *const ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDI_SetCC: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            ccidx: ::std::os::raw::c_int,
            selectedInOptional: *const bool,
            mutedInOptional: *const bool,
            ppqposInOptional: *const f64,
            chanmsgInOptional: *const ::std::os::raw::c_int,
            chanInOptional: *const ::std::os::raw::c_int,
            msg2InOptional: *const ::std::os::raw::c_int,
            msg3InOptional: *const ::std::os::raw::c_int,
            noSortInOptional: *const bool,
        ) -> bool,
    >,
    pub MIDI_SetCCShape: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            ccidx: ::std::os::raw::c_int,
            shape: ::std::os::raw::c_int,
            beztension: f64,
            noSortInOptional: *const bool,
        ) -> bool,
    >,
    pub MIDI_SetEvt: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            evtidx: ::std::os::raw::c_int,
            selectedInOptional: *const bool,
            mutedInOptional: *const bool,
            ppqposInOptional: *const f64,
            msgOptional: *const ::std::os::raw::c_char,
            msgOptional_sz: ::std::os::raw::c_int,
            noSortInOptional: *const bool,
        ) -> bool,
    >,
    pub MIDI_SetItemExtents:
        Option<unsafe extern "C" fn(item: *mut root::MediaItem, startQN: f64, endQN: f64) -> bool>,
    pub MIDI_SetNote: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            noteidx: ::std::os::raw::c_int,
            selectedInOptional: *const bool,
            mutedInOptional: *const bool,
            startppqposInOptional: *const f64,
            endppqposInOptional: *const f64,
            chanInOptional: *const ::std::os::raw::c_int,
            pitchInOptional: *const ::std::os::raw::c_int,
            velInOptional: *const ::std::os::raw::c_int,
            noSortInOptional: *const bool,
        ) -> bool,
    >,
    pub MIDI_SetTextSysexEvt: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            textsyxevtidx: ::std::os::raw::c_int,
            selectedInOptional: *const bool,
            mutedInOptional: *const bool,
            ppqposInOptional: *const f64,
            typeInOptional: *const ::std::os::raw::c_int,
            msgOptional: *const ::std::os::raw::c_char,
            msgOptional_sz: ::std::os::raw::c_int,
            noSortInOptional: *const bool,
        ) -> bool,
    >,
    pub MIDI_Sort: Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take)>,
    pub MIDIEditor_GetActive: Option<unsafe extern "C" fn() -> root::HWND>,
    pub MIDIEditor_GetMode:
        Option<unsafe extern "C" fn(midieditor: root::HWND) -> ::std::os::raw::c_int>,
    pub MIDIEditor_GetSetting_int: Option<
        unsafe extern "C" fn(
            midieditor: root::HWND,
            setting_desc: *const ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int,
    >,
    pub MIDIEditor_GetSetting_str: Option<
        unsafe extern "C" fn(
            midieditor: root::HWND,
            setting_desc: *const ::std::os::raw::c_char,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub MIDIEditor_GetTake:
        Option<unsafe extern "C" fn(midieditor: root::HWND) -> *mut root::MediaItem_Take>,
    pub MIDIEditor_LastFocused_OnCommand: Option<
        unsafe extern "C" fn(command_id: ::std::os::raw::c_int, islistviewcommand: bool) -> bool,
    >,
    pub MIDIEditor_OnCommand: Option<
        unsafe extern "C" fn(midieditor: root::HWND, command_id: ::std::os::raw::c_int) -> bool,
    >,
    pub MIDIEditor_SetSetting_int: Option<
        unsafe extern "C" fn(
            midieditor: root::HWND,
            setting_desc: *const ::std::os::raw::c_char,
            setting: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub mkpanstr: Option<unsafe extern "C" fn(strNeed64: *mut ::std::os::raw::c_char, pan: f64)>,
    pub mkvolpanstr:
        Option<unsafe extern "C" fn(strNeed64: *mut ::std::os::raw::c_char, vol: f64, pan: f64)>,
    pub mkvolstr: Option<unsafe extern "C" fn(strNeed64: *mut ::std::os::raw::c_char, vol: f64)>,
    pub MoveEditCursor: Option<unsafe extern "C" fn(adjamt: f64, dosel: bool)>,
    pub MoveMediaItemToTrack: Option<
        unsafe extern "C" fn(item: *mut root::MediaItem, desttr: *mut root::MediaTrack) -> bool,
    >,
    pub MuteAllTracks: Option<unsafe extern "C" fn(mute: bool)>,
    pub my_getViewport:
        Option<unsafe extern "C" fn(r: *mut root::RECT, sr: *const root::RECT, wantWorkArea: bool)>,
    pub NamedCommandLookup: Option<
        unsafe extern "C" fn(command_name: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int,
    >,
    pub OnPauseButton: Option<unsafe extern "C" fn()>,
    pub OnPauseButtonEx: Option<unsafe extern "C" fn(proj: *mut root::ReaProject)>,
    pub OnPlayButton: Option<unsafe extern "C" fn()>,
    pub OnPlayButtonEx: Option<unsafe extern "C" fn(proj: *mut root::ReaProject)>,
    pub OnStopButton: Option<unsafe extern "C" fn()>,
    pub OnStopButtonEx: Option<unsafe extern "C" fn(proj: *mut root::ReaProject)>,
    pub OpenColorThemeFile:
        Option<unsafe extern "C" fn(fn_: *const ::std::os::raw::c_char) -> bool>,
    pub OpenMediaExplorer: Option<
        unsafe extern "C" fn(mediafn: *const ::std::os::raw::c_char, play: bool) -> root::HWND,
    >,
    pub OscLocalMessageToHost: Option<
        unsafe extern "C" fn(message: *const ::std::os::raw::c_char, valueInOptional: *const f64),
    >,
    pub parse_timestr: Option<unsafe extern "C" fn(buf: *const ::std::os::raw::c_char) -> f64>,
    pub parse_timestr_len: Option<
        unsafe extern "C" fn(
            buf: *const ::std::os::raw::c_char,
            offset: f64,
            modeoverride: ::std::os::raw::c_int,
        ) -> f64,
    >,
    pub parse_timestr_pos: Option<
        unsafe extern "C" fn(
            buf: *const ::std::os::raw::c_char,
            modeoverride: ::std::os::raw::c_int,
        ) -> f64,
    >,
    pub parsepanstr: Option<unsafe extern "C" fn(str: *const ::std::os::raw::c_char) -> f64>,
    pub PCM_Sink_Create: Option<
        unsafe extern "C" fn(
            filename: *const ::std::os::raw::c_char,
            cfg: *const ::std::os::raw::c_char,
            cfg_sz: ::std::os::raw::c_int,
            nch: ::std::os::raw::c_int,
            srate: ::std::os::raw::c_int,
            buildpeaks: bool,
        ) -> *mut root::PCM_sink,
    >,
    pub PCM_Sink_CreateEx: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            filename: *const ::std::os::raw::c_char,
            cfg: *const ::std::os::raw::c_char,
            cfg_sz: ::std::os::raw::c_int,
            nch: ::std::os::raw::c_int,
            srate: ::std::os::raw::c_int,
            buildpeaks: bool,
        ) -> *mut root::PCM_sink,
    >,
    pub PCM_Sink_CreateMIDIFile: Option<
        unsafe extern "C" fn(
            filename: *const ::std::os::raw::c_char,
            cfg: *const ::std::os::raw::c_char,
            cfg_sz: ::std::os::raw::c_int,
            bpm: f64,
            div: ::std::os::raw::c_int,
        ) -> *mut root::PCM_sink,
    >,
    pub PCM_Sink_CreateMIDIFileEx: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            filename: *const ::std::os::raw::c_char,
            cfg: *const ::std::os::raw::c_char,
            cfg_sz: ::std::os::raw::c_int,
            bpm: f64,
            div: ::std::os::raw::c_int,
        ) -> *mut root::PCM_sink,
    >,
    pub PCM_Sink_Enum: Option<
        unsafe extern "C" fn(
            idx: ::std::os::raw::c_int,
            descstrOut: *mut *const ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_uint,
    >,
    pub PCM_Sink_GetExtension: Option<
        unsafe extern "C" fn(
            data: *const ::std::os::raw::c_char,
            data_sz: ::std::os::raw::c_int,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub PCM_Sink_ShowConfig: Option<
        unsafe extern "C" fn(
            cfg: *const ::std::os::raw::c_char,
            cfg_sz: ::std::os::raw::c_int,
            hwndParent: root::HWND,
        ) -> root::HWND,
    >,
    pub PCM_Source_CreateFromFile: Option<
        unsafe extern "C" fn(filename: *const ::std::os::raw::c_char) -> *mut root::PCM_source,
    >,
    pub PCM_Source_CreateFromFileEx: Option<
        unsafe extern "C" fn(
            filename: *const ::std::os::raw::c_char,
            forcenoMidiImp: bool,
        ) -> *mut root::PCM_source,
    >,
    pub PCM_Source_CreateFromSimple: Option<
        unsafe extern "C" fn(
            dec: *mut root::ISimpleMediaDecoder,
            fn_: *const ::std::os::raw::c_char,
        ) -> *mut root::PCM_source,
    >,
    pub PCM_Source_CreateFromType: Option<
        unsafe extern "C" fn(sourcetype: *const ::std::os::raw::c_char) -> *mut root::PCM_source,
    >,
    pub PCM_Source_Destroy: Option<unsafe extern "C" fn(src: *mut root::PCM_source)>,
    pub PCM_Source_GetPeaks: Option<
        unsafe extern "C" fn(
            src: *mut root::PCM_source,
            peakrate: f64,
            starttime: f64,
            numchannels: ::std::os::raw::c_int,
            numsamplesperchannel: ::std::os::raw::c_int,
            want_extra_type: ::std::os::raw::c_int,
            buf: *mut f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub PCM_Source_GetSectionInfo: Option<
        unsafe extern "C" fn(
            src: *mut root::PCM_source,
            offsOut: *mut f64,
            lenOut: *mut f64,
            revOut: *mut bool,
        ) -> bool,
    >,
    pub PeakBuild_Create: Option<
        unsafe extern "C" fn(
            src: *mut root::PCM_source,
            fn_: *const ::std::os::raw::c_char,
            srate: ::std::os::raw::c_int,
            nch: ::std::os::raw::c_int,
        ) -> *mut root::REAPER_PeakBuild_Interface,
    >,
    pub PeakBuild_CreateEx: Option<
        unsafe extern "C" fn(
            src: *mut root::PCM_source,
            fn_: *const ::std::os::raw::c_char,
            srate: ::std::os::raw::c_int,
            nch: ::std::os::raw::c_int,
            flags: ::std::os::raw::c_int,
        ) -> *mut root::REAPER_PeakBuild_Interface,
    >,
    pub PeakGet_Create: Option<
        unsafe extern "C" fn(
            fn_: *const ::std::os::raw::c_char,
            srate: ::std::os::raw::c_int,
            nch: ::std::os::raw::c_int,
        ) -> *mut root::REAPER_PeakGet_Interface,
    >,
    pub PitchShiftSubModeMenu: Option<
        unsafe extern "C" fn(
            hwnd: root::HWND,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            mode: ::std::os::raw::c_int,
            submode_sel: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub PlayPreview: Option<
        unsafe extern "C" fn(preview: *mut root::preview_register_t) -> ::std::os::raw::c_int,
    >,
    pub PlayPreviewEx: Option<
        unsafe extern "C" fn(
            preview: *mut root::preview_register_t,
            bufflags: ::std::os::raw::c_int,
            MSI: f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub PlayTrackPreview: Option<
        unsafe extern "C" fn(preview: *mut root::preview_register_t) -> ::std::os::raw::c_int,
    >,
    pub PlayTrackPreview2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            preview: *mut root::preview_register_t,
        ) -> ::std::os::raw::c_int,
    >,
    pub PlayTrackPreview2Ex: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            preview: *mut root::preview_register_t,
            flags: ::std::os::raw::c_int,
            msi: f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub plugin_getapi: Option<
        unsafe extern "C" fn(name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_void,
    >,
    pub plugin_getFilterList: Option<unsafe extern "C" fn() -> *const ::std::os::raw::c_char>,
    pub plugin_getImportableProjectFilterList:
        Option<unsafe extern "C" fn() -> *const ::std::os::raw::c_char>,
    pub plugin_register: Option<
        unsafe extern "C" fn(
            name: *const ::std::os::raw::c_char,
            infostruct: *mut ::std::os::raw::c_void,
        ) -> ::std::os::raw::c_int,
    >,
    pub PluginWantsAlwaysRunFx: Option<unsafe extern "C" fn(amt: ::std::os::raw::c_int)>,
    pub PreventUIRefresh: Option<unsafe extern "C" fn(prevent_count: ::std::os::raw::c_int)>,
    pub projectconfig_var_addr: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            idx: ::std::os::raw::c_int,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub projectconfig_var_getoffs: Option<
        unsafe extern "C" fn(
            name: *const ::std::os::raw::c_char,
            szOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub realloc_cmd_ptr: Option<
        unsafe extern "C" fn(
            ptr: *mut *mut ::std::os::raw::c_char,
            ptr_size: *mut ::std::os::raw::c_int,
            new_size: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub ReaperGetPitchShiftAPI: Option<
        unsafe extern "C" fn(version: ::std::os::raw::c_int) -> *mut root::IReaperPitchShift,
    >,
    pub ReaScriptError: Option<unsafe extern "C" fn(errmsg: *const ::std::os::raw::c_char)>,
    pub RecursiveCreateDirectory: Option<
        unsafe extern "C" fn(
            path: *const ::std::os::raw::c_char,
            ignored: usize,
        ) -> ::std::os::raw::c_int,
    >,
    pub reduce_open_files:
        Option<unsafe extern "C" fn(flags: ::std::os::raw::c_int) -> ::std::os::raw::c_int>,
    pub RefreshToolbar: Option<unsafe extern "C" fn(command_id: ::std::os::raw::c_int)>,
    pub RefreshToolbar2: Option<
        unsafe extern "C" fn(section_id: ::std::os::raw::c_int, command_id: ::std::os::raw::c_int),
    >,
    pub relative_fn: Option<
        unsafe extern "C" fn(
            in_: *const ::std::os::raw::c_char,
            out: *mut ::std::os::raw::c_char,
            out_sz: ::std::os::raw::c_int,
        ),
    >,
    pub RemoveTrackSend: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            category: ::std::os::raw::c_int,
            sendidx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub RenderFileSection: Option<
        unsafe extern "C" fn(
            source_filename: *const ::std::os::raw::c_char,
            target_filename: *const ::std::os::raw::c_char,
            start_percent: f64,
            end_percent: f64,
            playrate: f64,
        ) -> bool,
    >,
    pub ReorderSelectedTracks: Option<
        unsafe extern "C" fn(
            beforeTrackIdx: ::std::os::raw::c_int,
            makePrevFolder: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub Resample_EnumModes:
        Option<unsafe extern "C" fn(mode: ::std::os::raw::c_int) -> *const ::std::os::raw::c_char>,
    pub Resampler_Create: Option<unsafe extern "C" fn() -> *mut root::REAPER_Resample_Interface>,
    pub resolve_fn: Option<
        unsafe extern "C" fn(
            in_: *const ::std::os::raw::c_char,
            out: *mut ::std::os::raw::c_char,
            out_sz: ::std::os::raw::c_int,
        ),
    >,
    pub resolve_fn2: Option<
        unsafe extern "C" fn(
            in_: *const ::std::os::raw::c_char,
            out: *mut ::std::os::raw::c_char,
            out_sz: ::std::os::raw::c_int,
            checkSubDirOptional: *const ::std::os::raw::c_char,
        ),
    >,
    pub ReverseNamedCommandLookup: Option<
        unsafe extern "C" fn(command_id: ::std::os::raw::c_int) -> *const ::std::os::raw::c_char,
    >,
    pub ScaleFromEnvelopeMode:
        Option<unsafe extern "C" fn(scaling_mode: ::std::os::raw::c_int, val: f64) -> f64>,
    pub ScaleToEnvelopeMode:
        Option<unsafe extern "C" fn(scaling_mode: ::std::os::raw::c_int, val: f64) -> f64>,
    pub screenset_register: Option<
        unsafe extern "C" fn(
            id: *mut ::std::os::raw::c_char,
            callbackFunc: *mut ::std::os::raw::c_void,
            param: *mut ::std::os::raw::c_void,
        ),
    >,
    pub screenset_registerNew: Option<
        unsafe extern "C" fn(
            id: *mut ::std::os::raw::c_char,
            callbackFunc: root::screensetNewCallbackFunc,
            param: *mut ::std::os::raw::c_void,
        ),
    >,
    pub screenset_unregister: Option<unsafe extern "C" fn(id: *mut ::std::os::raw::c_char)>,
    pub screenset_unregisterByParam:
        Option<unsafe extern "C" fn(param: *mut ::std::os::raw::c_void)>,
    pub screenset_updateLastFocus: Option<unsafe extern "C" fn(prevWin: root::HWND)>,
    pub SectionFromUniqueID:
        Option<unsafe extern "C" fn(uniqueID: ::std::os::raw::c_int) -> *mut root::KbdSectionInfo>,
    pub SelectAllMediaItems:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject, selected: bool)>,
    pub SelectProjectInstance: Option<unsafe extern "C" fn(proj: *mut root::ReaProject)>,
    pub SendLocalOscMessage: Option<
        unsafe extern "C" fn(
            local_osc_handler: *mut ::std::os::raw::c_void,
            msg: *const ::std::os::raw::c_char,
            msglen: ::std::os::raw::c_int,
        ),
    >,
    pub SetActiveTake: Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take)>,
    pub SetAutomationMode: Option<unsafe extern "C" fn(mode: ::std::os::raw::c_int, onlySel: bool)>,
    pub SetCurrentBPM:
        Option<unsafe extern "C" fn(__proj: *mut root::ReaProject, bpm: f64, wantUndo: bool)>,
    pub SetCursorContext: Option<
        unsafe extern "C" fn(mode: ::std::os::raw::c_int, envInOptional: *mut root::TrackEnvelope),
    >,
    pub SetEditCurPos: Option<unsafe extern "C" fn(time: f64, moveview: bool, seekplay: bool)>,
    pub SetEditCurPos2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            time: f64,
            moveview: bool,
            seekplay: bool,
        ),
    >,
    pub SetEnvelopePoint: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            ptidx: ::std::os::raw::c_int,
            timeInOptional: *mut f64,
            valueInOptional: *mut f64,
            shapeInOptional: *mut ::std::os::raw::c_int,
            tensionInOptional: *mut f64,
            selectedInOptional: *mut bool,
            noSortInOptional: *mut bool,
        ) -> bool,
    >,
    pub SetEnvelopePointEx: Option<
        unsafe extern "C" fn(
            envelope: *mut root::TrackEnvelope,
            autoitem_idx: ::std::os::raw::c_int,
            ptidx: ::std::os::raw::c_int,
            timeInOptional: *mut f64,
            valueInOptional: *mut f64,
            shapeInOptional: *mut ::std::os::raw::c_int,
            tensionInOptional: *mut f64,
            selectedInOptional: *mut bool,
            noSortInOptional: *mut bool,
        ) -> bool,
    >,
    pub SetEnvelopeStateChunk: Option<
        unsafe extern "C" fn(
            env: *mut root::TrackEnvelope,
            str: *const ::std::os::raw::c_char,
            isundoOptional: bool,
        ) -> bool,
    >,
    pub SetExtState: Option<
        unsafe extern "C" fn(
            section: *const ::std::os::raw::c_char,
            key: *const ::std::os::raw::c_char,
            value: *const ::std::os::raw::c_char,
            persist: bool,
        ),
    >,
    pub SetGlobalAutomationOverride: Option<unsafe extern "C" fn(mode: ::std::os::raw::c_int)>,
    pub SetItemStateChunk: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            str: *const ::std::os::raw::c_char,
            isundoOptional: bool,
        ) -> bool,
    >,
    pub SetMasterTrackVisibility:
        Option<unsafe extern "C" fn(flag: ::std::os::raw::c_int) -> ::std::os::raw::c_int>,
    pub SetMediaItemInfo_Value: Option<
        unsafe extern "C" fn(
            item: *mut root::MediaItem,
            parmname: *const ::std::os::raw::c_char,
            newvalue: f64,
        ) -> bool,
    >,
    pub SetMediaItemLength: Option<
        unsafe extern "C" fn(item: *mut root::MediaItem, length: f64, refreshUI: bool) -> bool,
    >,
    pub SetMediaItemPosition: Option<
        unsafe extern "C" fn(item: *mut root::MediaItem, position: f64, refreshUI: bool) -> bool,
    >,
    pub SetMediaItemSelected:
        Option<unsafe extern "C" fn(item: *mut root::MediaItem, selected: bool)>,
    pub SetMediaItemTake_Source: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            source: *mut root::PCM_source,
        ) -> bool,
    >,
    pub SetMediaItemTakeInfo_Value: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            parmname: *const ::std::os::raw::c_char,
            newvalue: f64,
        ) -> bool,
    >,
    pub SetMediaTrackInfo_Value: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            parmname: *const ::std::os::raw::c_char,
            newvalue: f64,
        ) -> bool,
    >,
    pub SetMIDIEditorGrid:
        Option<unsafe extern "C" fn(project: *mut root::ReaProject, division: f64)>,
    pub SetMixerScroll:
        Option<unsafe extern "C" fn(leftmosttrack: *mut root::MediaTrack) -> *mut root::MediaTrack>,
    pub SetMouseModifier: Option<
        unsafe extern "C" fn(
            context: *const ::std::os::raw::c_char,
            modifier_flag: ::std::os::raw::c_int,
            action: *const ::std::os::raw::c_char,
        ),
    >,
    pub SetOnlyTrackSelected: Option<unsafe extern "C" fn(track: *mut root::MediaTrack)>,
    pub SetProjectGrid: Option<unsafe extern "C" fn(project: *mut root::ReaProject, division: f64)>,
    pub SetProjectMarker: Option<
        unsafe extern "C" fn(
            markrgnindexnumber: ::std::os::raw::c_int,
            isrgn: bool,
            pos: f64,
            rgnend: f64,
            name: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub SetProjectMarker2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            markrgnindexnumber: ::std::os::raw::c_int,
            isrgn: bool,
            pos: f64,
            rgnend: f64,
            name: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub SetProjectMarker3: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            markrgnindexnumber: ::std::os::raw::c_int,
            isrgn: bool,
            pos: f64,
            rgnend: f64,
            name: *const ::std::os::raw::c_char,
            color: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub SetProjectMarker4: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            markrgnindexnumber: ::std::os::raw::c_int,
            isrgn: bool,
            pos: f64,
            rgnend: f64,
            name: *const ::std::os::raw::c_char,
            color: ::std::os::raw::c_int,
            flags: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub SetProjectMarkerByIndex: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            markrgnidx: ::std::os::raw::c_int,
            isrgn: bool,
            pos: f64,
            rgnend: f64,
            IDnumber: ::std::os::raw::c_int,
            name: *const ::std::os::raw::c_char,
            color: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub SetProjectMarkerByIndex2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            markrgnidx: ::std::os::raw::c_int,
            isrgn: bool,
            pos: f64,
            rgnend: f64,
            IDnumber: ::std::os::raw::c_int,
            name: *const ::std::os::raw::c_char,
            color: ::std::os::raw::c_int,
            flags: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub SetProjExtState: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            extname: *const ::std::os::raw::c_char,
            key: *const ::std::os::raw::c_char,
            value: *const ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int,
    >,
    pub SetRegionRenderMatrix: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            regionindex: ::std::os::raw::c_int,
            track: *mut root::MediaTrack,
            addorremove: ::std::os::raw::c_int,
        ),
    >,
    pub SetRenderLastError: Option<unsafe extern "C" fn(errorstr: *const ::std::os::raw::c_char)>,
    pub SetTakeStretchMarker: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            idx: ::std::os::raw::c_int,
            pos: f64,
            srcposInOptional: *const f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub SetTakeStretchMarkerSlope: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            idx: ::std::os::raw::c_int,
            slope: f64,
        ) -> bool,
    >,
    pub SetTempoTimeSigMarker: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            ptidx: ::std::os::raw::c_int,
            timepos: f64,
            measurepos: ::std::os::raw::c_int,
            beatpos: f64,
            bpm: f64,
            timesig_num: ::std::os::raw::c_int,
            timesig_denom: ::std::os::raw::c_int,
            lineartempo: bool,
        ) -> bool,
    >,
    pub SetToggleCommandState: Option<
        unsafe extern "C" fn(
            section_id: ::std::os::raw::c_int,
            command_id: ::std::os::raw::c_int,
            state: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub SetTrackAutomationMode:
        Option<unsafe extern "C" fn(tr: *mut root::MediaTrack, mode: ::std::os::raw::c_int)>,
    pub SetTrackColor:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack, color: ::std::os::raw::c_int)>,
    pub SetTrackMIDILyrics: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            flag: ::std::os::raw::c_int,
            str: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub SetTrackMIDINoteName: Option<
        unsafe extern "C" fn(
            track: ::std::os::raw::c_int,
            pitch: ::std::os::raw::c_int,
            chan: ::std::os::raw::c_int,
            name: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub SetTrackMIDINoteNameEx: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            track: *mut root::MediaTrack,
            pitch: ::std::os::raw::c_int,
            chan: ::std::os::raw::c_int,
            name: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub SetTrackSelected:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack, selected: bool)>,
    pub SetTrackSendInfo_Value: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            category: ::std::os::raw::c_int,
            sendidx: ::std::os::raw::c_int,
            parmname: *const ::std::os::raw::c_char,
            newvalue: f64,
        ) -> bool,
    >,
    pub SetTrackSendUIPan: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            send_idx: ::std::os::raw::c_int,
            pan: f64,
            isend: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub SetTrackSendUIVol: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            send_idx: ::std::os::raw::c_int,
            vol: f64,
            isend: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub SetTrackStateChunk: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            str: *const ::std::os::raw::c_char,
            isundoOptional: bool,
        ) -> bool,
    >,
    pub ShowActionList:
        Option<unsafe extern "C" fn(caller: *mut root::KbdSectionInfo, callerWnd: root::HWND)>,
    pub ShowConsoleMsg: Option<unsafe extern "C" fn(msg: *const ::std::os::raw::c_char)>,
    pub ShowMessageBox: Option<
        unsafe extern "C" fn(
            msg: *const ::std::os::raw::c_char,
            title: *const ::std::os::raw::c_char,
            type_: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub ShowPopupMenu: Option<
        unsafe extern "C" fn(
            name: *const ::std::os::raw::c_char,
            x: ::std::os::raw::c_int,
            y: ::std::os::raw::c_int,
            hwndParentOptional: root::HWND,
            ctxOptional: *mut ::std::os::raw::c_void,
            ctx2Optional: ::std::os::raw::c_int,
            ctx3Optional: ::std::os::raw::c_int,
        ),
    >,
    pub SLIDER2DB: Option<unsafe extern "C" fn(y: f64) -> f64>,
    pub SnapToGrid:
        Option<unsafe extern "C" fn(project: *mut root::ReaProject, time_pos: f64) -> f64>,
    pub SoloAllTracks: Option<unsafe extern "C" fn(solo: ::std::os::raw::c_int)>,
    pub Splash_GetWnd: Option<unsafe extern "C" fn() -> root::HWND>,
    pub SplitMediaItem: Option<
        unsafe extern "C" fn(item: *mut root::MediaItem, position: f64) -> *mut root::MediaItem,
    >,
    pub StopPreview: Option<
        unsafe extern "C" fn(preview: *mut root::preview_register_t) -> ::std::os::raw::c_int,
    >,
    pub StopTrackPreview: Option<
        unsafe extern "C" fn(preview: *mut root::preview_register_t) -> ::std::os::raw::c_int,
    >,
    pub StopTrackPreview2: Option<
        unsafe extern "C" fn(
            proj: *mut ::std::os::raw::c_void,
            preview: *mut root::preview_register_t,
        ) -> ::std::os::raw::c_int,
    >,
    pub stringToGuid:
        Option<unsafe extern "C" fn(str: *const ::std::os::raw::c_char, g: *mut root::GUID)>,
    pub StuffMIDIMessage: Option<
        unsafe extern "C" fn(
            mode: ::std::os::raw::c_int,
            msg1: ::std::os::raw::c_int,
            msg2: ::std::os::raw::c_int,
            msg3: ::std::os::raw::c_int,
        ),
    >,
    pub TakeFX_AddByName: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fxname: *const ::std::os::raw::c_char,
            instantiate: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub TakeFX_CopyToTake: Option<
        unsafe extern "C" fn(
            src_take: *mut root::MediaItem_Take,
            src_fx: ::std::os::raw::c_int,
            dest_take: *mut root::MediaItem_Take,
            dest_fx: ::std::os::raw::c_int,
            is_move: bool,
        ),
    >,
    pub TakeFX_CopyToTrack: Option<
        unsafe extern "C" fn(
            src_take: *mut root::MediaItem_Take,
            src_fx: ::std::os::raw::c_int,
            dest_track: *mut root::MediaTrack,
            dest_fx: ::std::os::raw::c_int,
            is_move: bool,
        ),
    >,
    pub TakeFX_Delete: Option<
        unsafe extern "C" fn(take: *mut root::MediaItem_Take, fx: ::std::os::raw::c_int) -> bool,
    >,
    pub TakeFX_EndParamEdit: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TakeFX_FormatParamValue: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            val: f64,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TakeFX_FormatParamValueNormalized: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            value: f64,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TakeFX_GetChainVisible:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take) -> ::std::os::raw::c_int>,
    pub TakeFX_GetCount:
        Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take) -> ::std::os::raw::c_int>,
    pub TakeFX_GetEnabled: Option<
        unsafe extern "C" fn(take: *mut root::MediaItem_Take, fx: ::std::os::raw::c_int) -> bool,
    >,
    pub TakeFX_GetEnvelope: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fxindex: ::std::os::raw::c_int,
            parameterindex: ::std::os::raw::c_int,
            create: bool,
        ) -> *mut root::TrackEnvelope,
    >,
    pub TakeFX_GetFloatingWindow: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            index: ::std::os::raw::c_int,
        ) -> root::HWND,
    >,
    pub TakeFX_GetFormattedParamValue: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TakeFX_GetFXGUID: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
        ) -> *mut root::GUID,
    >,
    pub TakeFX_GetFXName: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TakeFX_GetIOSize: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            inputPinsOutOptional: *mut ::std::os::raw::c_int,
            outputPinsOutOptional: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub TakeFX_GetNamedConfigParm: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            parmname: *const ::std::os::raw::c_char,
            bufOut: *mut ::std::os::raw::c_char,
            bufOut_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TakeFX_GetNumParams: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub TakeFX_GetOffline: Option<
        unsafe extern "C" fn(take: *mut root::MediaItem_Take, fx: ::std::os::raw::c_int) -> bool,
    >,
    pub TakeFX_GetOpen: Option<
        unsafe extern "C" fn(take: *mut root::MediaItem_Take, fx: ::std::os::raw::c_int) -> bool,
    >,
    pub TakeFX_GetParam: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            minvalOut: *mut f64,
            maxvalOut: *mut f64,
        ) -> f64,
    >,
    pub TakeFX_GetParameterStepSizes: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            stepOut: *mut f64,
            smallstepOut: *mut f64,
            largestepOut: *mut f64,
            istoggleOut: *mut bool,
        ) -> bool,
    >,
    pub TakeFX_GetParamEx: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            minvalOut: *mut f64,
            maxvalOut: *mut f64,
            midvalOut: *mut f64,
        ) -> f64,
    >,
    pub TakeFX_GetParamName: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TakeFX_GetParamNormalized: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
        ) -> f64,
    >,
    pub TakeFX_GetPinMappings: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            isoutput: ::std::os::raw::c_int,
            pin: ::std::os::raw::c_int,
            high32OutOptional: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub TakeFX_GetPreset: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            presetname: *mut ::std::os::raw::c_char,
            presetname_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TakeFX_GetPresetIndex: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            numberOfPresetsOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub TakeFX_GetUserPresetFilename: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            fn_: *mut ::std::os::raw::c_char,
            fn_sz: ::std::os::raw::c_int,
        ),
    >,
    pub TakeFX_NavigatePresets: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            presetmove: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TakeFX_SetEnabled: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            enabled: bool,
        ),
    >,
    pub TakeFX_SetNamedConfigParm: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            parmname: *const ::std::os::raw::c_char,
            value: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub TakeFX_SetOffline: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            offline: bool,
        ),
    >,
    pub TakeFX_SetOpen: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            open: bool,
        ),
    >,
    pub TakeFX_SetParam: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            val: f64,
        ) -> bool,
    >,
    pub TakeFX_SetParamNormalized: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            value: f64,
        ) -> bool,
    >,
    pub TakeFX_SetPinMappings: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            isoutput: ::std::os::raw::c_int,
            pin: ::std::os::raw::c_int,
            low32bits: ::std::os::raw::c_int,
            hi32bits: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TakeFX_SetPreset: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            presetname: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub TakeFX_SetPresetByIndex: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            fx: ::std::os::raw::c_int,
            idx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TakeFX_Show: Option<
        unsafe extern "C" fn(
            take: *mut root::MediaItem_Take,
            index: ::std::os::raw::c_int,
            showFlag: ::std::os::raw::c_int,
        ),
    >,
    pub TakeIsMIDI: Option<unsafe extern "C" fn(take: *mut root::MediaItem_Take) -> bool>,
    pub ThemeLayout_GetLayout: Option<
        unsafe extern "C" fn(
            section: *const ::std::os::raw::c_char,
            idx: ::std::os::raw::c_int,
            nameOut: *mut ::std::os::raw::c_char,
            nameOut_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub ThemeLayout_GetParameter: Option<
        unsafe extern "C" fn(
            wp: ::std::os::raw::c_int,
            descOutOptional: *mut *const ::std::os::raw::c_char,
            valueOutOptional: *mut ::std::os::raw::c_int,
            defValueOutOptional: *mut ::std::os::raw::c_int,
            minValueOutOptional: *mut ::std::os::raw::c_int,
            maxValueOutOptional: *mut ::std::os::raw::c_int,
        ) -> *const ::std::os::raw::c_char,
    >,
    pub ThemeLayout_RefreshAll: Option<unsafe extern "C" fn()>,
    pub ThemeLayout_SetLayout: Option<
        unsafe extern "C" fn(
            section: *const ::std::os::raw::c_char,
            layout: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub ThemeLayout_SetParameter: Option<
        unsafe extern "C" fn(
            wp: ::std::os::raw::c_int,
            value: ::std::os::raw::c_int,
            persist: bool,
        ) -> bool,
    >,
    pub time_precise: Option<unsafe extern "C" fn() -> f64>,
    pub TimeMap2_beatsToTime: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            tpos: f64,
            measuresInOptional: *const ::std::os::raw::c_int,
        ) -> f64,
    >,
    pub TimeMap2_GetDividedBpmAtTime:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject, time: f64) -> f64>,
    pub TimeMap2_GetNextChangeTime:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject, time: f64) -> f64>,
    pub TimeMap2_QNToTime:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject, qn: f64) -> f64>,
    pub TimeMap2_timeToBeats: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            tpos: f64,
            measuresOutOptional: *mut ::std::os::raw::c_int,
            cmlOutOptional: *mut ::std::os::raw::c_int,
            fullbeatsOutOptional: *mut f64,
            cdenomOutOptional: *mut ::std::os::raw::c_int,
        ) -> f64,
    >,
    pub TimeMap2_timeToQN:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject, tpos: f64) -> f64>,
    pub TimeMap_curFrameRate: Option<
        unsafe extern "C" fn(proj: *mut root::ReaProject, dropFrameOutOptional: *mut bool) -> f64,
    >,
    pub TimeMap_GetDividedBpmAtTime: Option<unsafe extern "C" fn(time: f64) -> f64>,
    pub TimeMap_GetMeasureInfo: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            measure: ::std::os::raw::c_int,
            qn_startOut: *mut f64,
            qn_endOut: *mut f64,
            timesig_numOut: *mut ::std::os::raw::c_int,
            timesig_denomOut: *mut ::std::os::raw::c_int,
            tempoOut: *mut f64,
        ) -> f64,
    >,
    pub TimeMap_GetMetronomePattern: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            time: f64,
            pattern: *mut ::std::os::raw::c_char,
            pattern_sz: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub TimeMap_GetTimeSigAtTime: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            time: f64,
            timesig_numOut: *mut ::std::os::raw::c_int,
            timesig_denomOut: *mut ::std::os::raw::c_int,
            tempoOut: *mut f64,
        ),
    >,
    pub TimeMap_QNToMeasures: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            qn: f64,
            qnMeasureStartOutOptional: *mut f64,
            qnMeasureEndOutOptional: *mut f64,
        ) -> ::std::os::raw::c_int,
    >,
    pub TimeMap_QNToTime: Option<unsafe extern "C" fn(qn: f64) -> f64>,
    pub TimeMap_QNToTime_abs:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject, qn: f64) -> f64>,
    pub TimeMap_timeToQN: Option<unsafe extern "C" fn(tpos: f64) -> f64>,
    pub TimeMap_timeToQN_abs:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject, tpos: f64) -> f64>,
    pub ToggleTrackSendUIMute: Option<
        unsafe extern "C" fn(track: *mut root::MediaTrack, send_idx: ::std::os::raw::c_int) -> bool,
    >,
    pub Track_GetPeakHoldDB: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            channel: ::std::os::raw::c_int,
            clear: bool,
        ) -> f64,
    >,
    pub Track_GetPeakInfo: Option<
        unsafe extern "C" fn(track: *mut root::MediaTrack, channel: ::std::os::raw::c_int) -> f64,
    >,
    pub TrackCtl_SetToolTip: Option<
        unsafe extern "C" fn(
            fmt: *const ::std::os::raw::c_char,
            xpos: ::std::os::raw::c_int,
            ypos: ::std::os::raw::c_int,
            topmost: bool,
        ),
    >,
    pub TrackFX_AddByName: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fxname: *const ::std::os::raw::c_char,
            recFX: bool,
            instantiate: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub TrackFX_CopyToTake: Option<
        unsafe extern "C" fn(
            src_track: *mut root::MediaTrack,
            src_fx: ::std::os::raw::c_int,
            dest_take: *mut root::MediaItem_Take,
            dest_fx: ::std::os::raw::c_int,
            is_move: bool,
        ),
    >,
    pub TrackFX_CopyToTrack: Option<
        unsafe extern "C" fn(
            src_track: *mut root::MediaTrack,
            src_fx: ::std::os::raw::c_int,
            dest_track: *mut root::MediaTrack,
            dest_fx: ::std::os::raw::c_int,
            is_move: bool,
        ),
    >,
    pub TrackFX_Delete: Option<
        unsafe extern "C" fn(track: *mut root::MediaTrack, fx: ::std::os::raw::c_int) -> bool,
    >,
    pub TrackFX_EndParamEdit: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_FormatParamValue: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            val: f64,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_FormatParamValueNormalized: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            value: f64,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_GetByName: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fxname: *const ::std::os::raw::c_char,
            instantiate: bool,
        ) -> ::std::os::raw::c_int,
    >,
    pub TrackFX_GetChainVisible:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack) -> ::std::os::raw::c_int>,
    pub TrackFX_GetCount:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack) -> ::std::os::raw::c_int>,
    pub TrackFX_GetEnabled: Option<
        unsafe extern "C" fn(track: *mut root::MediaTrack, fx: ::std::os::raw::c_int) -> bool,
    >,
    pub TrackFX_GetEQ: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            instantiate: bool,
        ) -> ::std::os::raw::c_int,
    >,
    pub TrackFX_GetEQBandEnabled: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fxidx: ::std::os::raw::c_int,
            bandtype: ::std::os::raw::c_int,
            bandidx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_GetEQParam: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fxidx: ::std::os::raw::c_int,
            paramidx: ::std::os::raw::c_int,
            bandtypeOut: *mut ::std::os::raw::c_int,
            bandidxOut: *mut ::std::os::raw::c_int,
            paramtypeOut: *mut ::std::os::raw::c_int,
            normvalOut: *mut f64,
        ) -> bool,
    >,
    pub TrackFX_GetFloatingWindow: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            index: ::std::os::raw::c_int,
        ) -> root::HWND,
    >,
    pub TrackFX_GetFormattedParamValue: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_GetFXGUID: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
        ) -> *mut root::GUID,
    >,
    pub TrackFX_GetFXName: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_GetInstrument:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack) -> ::std::os::raw::c_int>,
    pub TrackFX_GetIOSize: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            inputPinsOutOptional: *mut ::std::os::raw::c_int,
            outputPinsOutOptional: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub TrackFX_GetNamedConfigParm: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            parmname: *const ::std::os::raw::c_char,
            bufOut: *mut ::std::os::raw::c_char,
            bufOut_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_GetNumParams: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub TrackFX_GetOffline: Option<
        unsafe extern "C" fn(track: *mut root::MediaTrack, fx: ::std::os::raw::c_int) -> bool,
    >,
    pub TrackFX_GetOpen: Option<
        unsafe extern "C" fn(track: *mut root::MediaTrack, fx: ::std::os::raw::c_int) -> bool,
    >,
    pub TrackFX_GetParam: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            minvalOut: *mut f64,
            maxvalOut: *mut f64,
        ) -> f64,
    >,
    pub TrackFX_GetParameterStepSizes: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            stepOut: *mut f64,
            smallstepOut: *mut f64,
            largestepOut: *mut f64,
            istoggleOut: *mut bool,
        ) -> bool,
    >,
    pub TrackFX_GetParamEx: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            minvalOut: *mut f64,
            maxvalOut: *mut f64,
            midvalOut: *mut f64,
        ) -> f64,
    >,
    pub TrackFX_GetParamName: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            buf: *mut ::std::os::raw::c_char,
            buf_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_GetParamNormalized: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
        ) -> f64,
    >,
    pub TrackFX_GetPinMappings: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            isoutput: ::std::os::raw::c_int,
            pin: ::std::os::raw::c_int,
            high32OutOptional: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub TrackFX_GetPreset: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            presetname: *mut ::std::os::raw::c_char,
            presetname_sz: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_GetPresetIndex: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            numberOfPresetsOut: *mut ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int,
    >,
    pub TrackFX_GetRecChainVisible:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack) -> ::std::os::raw::c_int>,
    pub TrackFX_GetRecCount:
        Option<unsafe extern "C" fn(track: *mut root::MediaTrack) -> ::std::os::raw::c_int>,
    pub TrackFX_GetUserPresetFilename: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            fn_: *mut ::std::os::raw::c_char,
            fn_sz: ::std::os::raw::c_int,
        ),
    >,
    pub TrackFX_NavigatePresets: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            presetmove: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_SetEnabled: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            enabled: bool,
        ),
    >,
    pub TrackFX_SetEQBandEnabled: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fxidx: ::std::os::raw::c_int,
            bandtype: ::std::os::raw::c_int,
            bandidx: ::std::os::raw::c_int,
            enable: bool,
        ) -> bool,
    >,
    pub TrackFX_SetEQParam: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fxidx: ::std::os::raw::c_int,
            bandtype: ::std::os::raw::c_int,
            bandidx: ::std::os::raw::c_int,
            paramtype: ::std::os::raw::c_int,
            val: f64,
            isnorm: bool,
        ) -> bool,
    >,
    pub TrackFX_SetNamedConfigParm: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            parmname: *const ::std::os::raw::c_char,
            value: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub TrackFX_SetOffline: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            offline: bool,
        ),
    >,
    pub TrackFX_SetOpen: Option<
        unsafe extern "C" fn(track: *mut root::MediaTrack, fx: ::std::os::raw::c_int, open: bool),
    >,
    pub TrackFX_SetParam: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            val: f64,
        ) -> bool,
    >,
    pub TrackFX_SetParamNormalized: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            param: ::std::os::raw::c_int,
            value: f64,
        ) -> bool,
    >,
    pub TrackFX_SetPinMappings: Option<
        unsafe extern "C" fn(
            tr: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            isoutput: ::std::os::raw::c_int,
            pin: ::std::os::raw::c_int,
            low32bits: ::std::os::raw::c_int,
            hi32bits: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_SetPreset: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            presetname: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub TrackFX_SetPresetByIndex: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            fx: ::std::os::raw::c_int,
            idx: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub TrackFX_Show: Option<
        unsafe extern "C" fn(
            track: *mut root::MediaTrack,
            index: ::std::os::raw::c_int,
            showFlag: ::std::os::raw::c_int,
        ),
    >,
    pub TrackList_AdjustWindows: Option<unsafe extern "C" fn(isMinor: bool)>,
    pub TrackList_UpdateAllExternalSurfaces: Option<unsafe extern "C" fn()>,
    pub Undo_BeginBlock: Option<unsafe extern "C" fn()>,
    pub Undo_BeginBlock2: Option<unsafe extern "C" fn(proj: *mut root::ReaProject)>,
    pub Undo_CanRedo2:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> *const ::std::os::raw::c_char>,
    pub Undo_CanUndo2:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> *const ::std::os::raw::c_char>,
    pub Undo_DoRedo2:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> ::std::os::raw::c_int>,
    pub Undo_DoUndo2:
        Option<unsafe extern "C" fn(proj: *mut root::ReaProject) -> ::std::os::raw::c_int>,
    pub Undo_EndBlock: Option<
        unsafe extern "C" fn(
            descchange: *const ::std::os::raw::c_char,
            extraflags: ::std::os::raw::c_int,
        ),
    >,
    pub Undo_EndBlock2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            descchange: *const ::std::os::raw::c_char,
            extraflags: ::std::os::raw::c_int,
        ),
    >,
    pub Undo_OnStateChange: Option<unsafe extern "C" fn(descchange: *const ::std::os::raw::c_char)>,
    pub Undo_OnStateChange2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            descchange: *const ::std::os::raw::c_char,
        ),
    >,
    pub Undo_OnStateChange_Item: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            name: *const ::std::os::raw::c_char,
            item: *mut root::MediaItem,
        ),
    >,
    pub Undo_OnStateChangeEx: Option<
        unsafe extern "C" fn(
            descchange: *const ::std::os::raw::c_char,
            whichStates: ::std::os::raw::c_int,
            trackparm: ::std::os::raw::c_int,
        ),
    >,
    pub Undo_OnStateChangeEx2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            descchange: *const ::std::os::raw::c_char,
            whichStates: ::std::os::raw::c_int,
            trackparm: ::std::os::raw::c_int,
        ),
    >,
    pub update_disk_counters: Option<
        unsafe extern "C" fn(readamt: ::std::os::raw::c_int, writeamt: ::std::os::raw::c_int),
    >,
    pub UpdateArrange: Option<unsafe extern "C" fn()>,
    pub UpdateItemInProject: Option<unsafe extern "C" fn(item: *mut root::MediaItem)>,
    pub UpdateTimeline: Option<unsafe extern "C" fn()>,
    pub ValidatePtr: Option<
        unsafe extern "C" fn(
            pointer: *mut ::std::os::raw::c_void,
            ctypename: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub ValidatePtr2: Option<
        unsafe extern "C" fn(
            proj: *mut root::ReaProject,
            pointer: *mut ::std::os::raw::c_void,
            ctypename: *const ::std::os::raw::c_char,
        ) -> bool,
    >,
    pub ViewPrefs: Option<
        unsafe extern "C" fn(
            page: ::std::os::raw::c_int,
            pageByName: *const ::std::os::raw::c_char,
        ),
    >,
    pub WDL_VirtualWnd_ScaledBlitBG: Option<
        unsafe extern "C" fn(
            dest: *mut root::reaper_functions::LICE_IBitmap,
            src: *mut root::reaper_functions::WDL_VirtualWnd_BGCfg,
            destx: ::std::os::raw::c_int,
            desty: ::std::os::raw::c_int,
            destw: ::std::os::raw::c_int,
            desth: ::std::os::raw::c_int,
            clipx: ::std::os::raw::c_int,
            clipy: ::std::os::raw::c_int,
            clipw: ::std::os::raw::c_int,
            cliph: ::std::os::raw::c_int,
            alpha: f32,
            mode: ::std::os::raw::c_int,
        ) -> bool,
    >,
    pub GetMidiInput:
        Option<unsafe extern "C" fn(idx: ::std::os::raw::c_int) -> *mut root::midi_Input>,
    pub GetMidiOutput:
        Option<unsafe extern "C" fn(idx: ::std::os::raw::c_int) -> *mut root::midi_Output>,
}
impl Reaper {
    #[doc = r" Loads all available REAPER functions plug-in context and returns a `Reaper` instance"]
    #[doc = r" which allows you to call these functions."]
    pub fn load(context: &ReaperPluginContext) -> Reaper {
        let get_func = &context.function_provider;
        unsafe {
            Reaper {
                __mergesort: std::mem::transmute(get_func(c_str!(stringify!(__mergesort)))),
                AddCustomizableMenu: std::mem::transmute(get_func(c_str!(stringify!(
                    AddCustomizableMenu
                )))),
                AddExtensionsMainMenu: std::mem::transmute(get_func(c_str!(stringify!(
                    AddExtensionsMainMenu
                )))),
                AddMediaItemToTrack: std::mem::transmute(get_func(c_str!(stringify!(
                    AddMediaItemToTrack
                )))),
                AddProjectMarker: std::mem::transmute(get_func(c_str!(stringify!(
                    AddProjectMarker
                )))),
                AddProjectMarker2: std::mem::transmute(get_func(c_str!(stringify!(
                    AddProjectMarker2
                )))),
                AddRemoveReaScript: std::mem::transmute(get_func(c_str!(stringify!(
                    AddRemoveReaScript
                )))),
                AddTakeToMediaItem: std::mem::transmute(get_func(c_str!(stringify!(
                    AddTakeToMediaItem
                )))),
                AddTempoTimeSigMarker: std::mem::transmute(get_func(c_str!(stringify!(
                    AddTempoTimeSigMarker
                )))),
                adjustZoom: std::mem::transmute(get_func(c_str!(stringify!(adjustZoom)))),
                AnyTrackSolo: std::mem::transmute(get_func(c_str!(stringify!(AnyTrackSolo)))),
                APIExists: std::mem::transmute(get_func(c_str!(stringify!(APIExists)))),
                APITest: std::mem::transmute(get_func(c_str!(stringify!(APITest)))),
                ApplyNudge: std::mem::transmute(get_func(c_str!(stringify!(ApplyNudge)))),
                ArmCommand: std::mem::transmute(get_func(c_str!(stringify!(ArmCommand)))),
                Audio_Init: std::mem::transmute(get_func(c_str!(stringify!(Audio_Init)))),
                Audio_IsPreBuffer: std::mem::transmute(get_func(c_str!(stringify!(
                    Audio_IsPreBuffer
                )))),
                Audio_IsRunning: std::mem::transmute(get_func(c_str!(stringify!(Audio_IsRunning)))),
                Audio_Quit: std::mem::transmute(get_func(c_str!(stringify!(Audio_Quit)))),
                Audio_RegHardwareHook: std::mem::transmute(get_func(c_str!(stringify!(
                    Audio_RegHardwareHook
                )))),
                AudioAccessorStateChanged: std::mem::transmute(get_func(c_str!(stringify!(
                    AudioAccessorStateChanged
                )))),
                AudioAccessorUpdate: std::mem::transmute(get_func(c_str!(stringify!(
                    AudioAccessorUpdate
                )))),
                AudioAccessorValidateState: std::mem::transmute(get_func(c_str!(stringify!(
                    AudioAccessorValidateState
                )))),
                BypassFxAllTracks: std::mem::transmute(get_func(c_str!(stringify!(
                    BypassFxAllTracks
                )))),
                CalculatePeaks: std::mem::transmute(get_func(c_str!(stringify!(CalculatePeaks)))),
                CalculatePeaksFloatSrcPtr: std::mem::transmute(get_func(c_str!(stringify!(
                    CalculatePeaksFloatSrcPtr
                )))),
                ClearAllRecArmed: std::mem::transmute(get_func(c_str!(stringify!(
                    ClearAllRecArmed
                )))),
                ClearConsole: std::mem::transmute(get_func(c_str!(stringify!(ClearConsole)))),
                ClearPeakCache: std::mem::transmute(get_func(c_str!(stringify!(ClearPeakCache)))),
                ColorFromNative: std::mem::transmute(get_func(c_str!(stringify!(ColorFromNative)))),
                ColorToNative: std::mem::transmute(get_func(c_str!(stringify!(ColorToNative)))),
                CountActionShortcuts: std::mem::transmute(get_func(c_str!(stringify!(
                    CountActionShortcuts
                )))),
                CountAutomationItems: std::mem::transmute(get_func(c_str!(stringify!(
                    CountAutomationItems
                )))),
                CountEnvelopePoints: std::mem::transmute(get_func(c_str!(stringify!(
                    CountEnvelopePoints
                )))),
                CountEnvelopePointsEx: std::mem::transmute(get_func(c_str!(stringify!(
                    CountEnvelopePointsEx
                )))),
                CountMediaItems: std::mem::transmute(get_func(c_str!(stringify!(CountMediaItems)))),
                CountProjectMarkers: std::mem::transmute(get_func(c_str!(stringify!(
                    CountProjectMarkers
                )))),
                CountSelectedMediaItems: std::mem::transmute(get_func(c_str!(stringify!(
                    CountSelectedMediaItems
                )))),
                CountSelectedTracks: std::mem::transmute(get_func(c_str!(stringify!(
                    CountSelectedTracks
                )))),
                CountSelectedTracks2: std::mem::transmute(get_func(c_str!(stringify!(
                    CountSelectedTracks2
                )))),
                CountTakeEnvelopes: std::mem::transmute(get_func(c_str!(stringify!(
                    CountTakeEnvelopes
                )))),
                CountTakes: std::mem::transmute(get_func(c_str!(stringify!(CountTakes)))),
                CountTCPFXParms: std::mem::transmute(get_func(c_str!(stringify!(CountTCPFXParms)))),
                CountTempoTimeSigMarkers: std::mem::transmute(get_func(c_str!(stringify!(
                    CountTempoTimeSigMarkers
                )))),
                CountTrackEnvelopes: std::mem::transmute(get_func(c_str!(stringify!(
                    CountTrackEnvelopes
                )))),
                CountTrackMediaItems: std::mem::transmute(get_func(c_str!(stringify!(
                    CountTrackMediaItems
                )))),
                CountTracks: std::mem::transmute(get_func(c_str!(stringify!(CountTracks)))),
                CreateLocalOscHandler: std::mem::transmute(get_func(c_str!(stringify!(
                    CreateLocalOscHandler
                )))),
                CreateMIDIInput: std::mem::transmute(get_func(c_str!(stringify!(CreateMIDIInput)))),
                CreateMIDIOutput: std::mem::transmute(get_func(c_str!(stringify!(
                    CreateMIDIOutput
                )))),
                CreateNewMIDIItemInProj: std::mem::transmute(get_func(c_str!(stringify!(
                    CreateNewMIDIItemInProj
                )))),
                CreateTakeAudioAccessor: std::mem::transmute(get_func(c_str!(stringify!(
                    CreateTakeAudioAccessor
                )))),
                CreateTrackAudioAccessor: std::mem::transmute(get_func(c_str!(stringify!(
                    CreateTrackAudioAccessor
                )))),
                CreateTrackSend: std::mem::transmute(get_func(c_str!(stringify!(CreateTrackSend)))),
                CSurf_FlushUndo: std::mem::transmute(get_func(c_str!(stringify!(CSurf_FlushUndo)))),
                CSurf_GetTouchState: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_GetTouchState
                )))),
                CSurf_GoEnd: std::mem::transmute(get_func(c_str!(stringify!(CSurf_GoEnd)))),
                CSurf_GoStart: std::mem::transmute(get_func(c_str!(stringify!(CSurf_GoStart)))),
                CSurf_NumTracks: std::mem::transmute(get_func(c_str!(stringify!(CSurf_NumTracks)))),
                CSurf_OnArrow: std::mem::transmute(get_func(c_str!(stringify!(CSurf_OnArrow)))),
                CSurf_OnFwd: std::mem::transmute(get_func(c_str!(stringify!(CSurf_OnFwd)))),
                CSurf_OnFXChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnFXChange
                )))),
                CSurf_OnInputMonitorChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnInputMonitorChange
                )))),
                CSurf_OnInputMonitorChangeEx: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnInputMonitorChangeEx
                )))),
                CSurf_OnMuteChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnMuteChange
                )))),
                CSurf_OnMuteChangeEx: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnMuteChangeEx
                )))),
                CSurf_OnOscControlMessage: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnOscControlMessage
                )))),
                CSurf_OnPanChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnPanChange
                )))),
                CSurf_OnPanChangeEx: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnPanChangeEx
                )))),
                CSurf_OnPause: std::mem::transmute(get_func(c_str!(stringify!(CSurf_OnPause)))),
                CSurf_OnPlay: std::mem::transmute(get_func(c_str!(stringify!(CSurf_OnPlay)))),
                CSurf_OnPlayRateChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnPlayRateChange
                )))),
                CSurf_OnRecArmChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnRecArmChange
                )))),
                CSurf_OnRecArmChangeEx: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnRecArmChangeEx
                )))),
                CSurf_OnRecord: std::mem::transmute(get_func(c_str!(stringify!(CSurf_OnRecord)))),
                CSurf_OnRecvPanChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnRecvPanChange
                )))),
                CSurf_OnRecvVolumeChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnRecvVolumeChange
                )))),
                CSurf_OnRew: std::mem::transmute(get_func(c_str!(stringify!(CSurf_OnRew)))),
                CSurf_OnRewFwd: std::mem::transmute(get_func(c_str!(stringify!(CSurf_OnRewFwd)))),
                CSurf_OnScroll: std::mem::transmute(get_func(c_str!(stringify!(CSurf_OnScroll)))),
                CSurf_OnSelectedChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnSelectedChange
                )))),
                CSurf_OnSendPanChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnSendPanChange
                )))),
                CSurf_OnSendVolumeChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnSendVolumeChange
                )))),
                CSurf_OnSoloChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnSoloChange
                )))),
                CSurf_OnSoloChangeEx: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnSoloChangeEx
                )))),
                CSurf_OnStop: std::mem::transmute(get_func(c_str!(stringify!(CSurf_OnStop)))),
                CSurf_OnTempoChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnTempoChange
                )))),
                CSurf_OnTrackSelection: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnTrackSelection
                )))),
                CSurf_OnVolumeChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnVolumeChange
                )))),
                CSurf_OnVolumeChangeEx: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnVolumeChangeEx
                )))),
                CSurf_OnWidthChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnWidthChange
                )))),
                CSurf_OnWidthChangeEx: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_OnWidthChangeEx
                )))),
                CSurf_OnZoom: std::mem::transmute(get_func(c_str!(stringify!(CSurf_OnZoom)))),
                CSurf_ResetAllCachedVolPanStates: std::mem::transmute(get_func(c_str!(
                    stringify!(CSurf_ResetAllCachedVolPanStates)
                ))),
                CSurf_ScrubAmt: std::mem::transmute(get_func(c_str!(stringify!(CSurf_ScrubAmt)))),
                CSurf_SetAutoMode: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_SetAutoMode
                )))),
                CSurf_SetPlayState: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_SetPlayState
                )))),
                CSurf_SetRepeatState: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_SetRepeatState
                )))),
                CSurf_SetSurfaceMute: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_SetSurfaceMute
                )))),
                CSurf_SetSurfacePan: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_SetSurfacePan
                )))),
                CSurf_SetSurfaceRecArm: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_SetSurfaceRecArm
                )))),
                CSurf_SetSurfaceSelected: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_SetSurfaceSelected
                )))),
                CSurf_SetSurfaceSolo: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_SetSurfaceSolo
                )))),
                CSurf_SetSurfaceVolume: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_SetSurfaceVolume
                )))),
                CSurf_SetTrackListChange: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_SetTrackListChange
                )))),
                CSurf_TrackFromID: std::mem::transmute(get_func(c_str!(stringify!(
                    CSurf_TrackFromID
                )))),
                CSurf_TrackToID: std::mem::transmute(get_func(c_str!(stringify!(CSurf_TrackToID)))),
                DB2SLIDER: std::mem::transmute(get_func(c_str!(stringify!(DB2SLIDER)))),
                DeleteActionShortcut: std::mem::transmute(get_func(c_str!(stringify!(
                    DeleteActionShortcut
                )))),
                DeleteEnvelopePointEx: std::mem::transmute(get_func(c_str!(stringify!(
                    DeleteEnvelopePointEx
                )))),
                DeleteEnvelopePointRange: std::mem::transmute(get_func(c_str!(stringify!(
                    DeleteEnvelopePointRange
                )))),
                DeleteEnvelopePointRangeEx: std::mem::transmute(get_func(c_str!(stringify!(
                    DeleteEnvelopePointRangeEx
                )))),
                DeleteExtState: std::mem::transmute(get_func(c_str!(stringify!(DeleteExtState)))),
                DeleteProjectMarker: std::mem::transmute(get_func(c_str!(stringify!(
                    DeleteProjectMarker
                )))),
                DeleteProjectMarkerByIndex: std::mem::transmute(get_func(c_str!(stringify!(
                    DeleteProjectMarkerByIndex
                )))),
                DeleteTakeStretchMarkers: std::mem::transmute(get_func(c_str!(stringify!(
                    DeleteTakeStretchMarkers
                )))),
                DeleteTempoTimeSigMarker: std::mem::transmute(get_func(c_str!(stringify!(
                    DeleteTempoTimeSigMarker
                )))),
                DeleteTrack: std::mem::transmute(get_func(c_str!(stringify!(DeleteTrack)))),
                DeleteTrackMediaItem: std::mem::transmute(get_func(c_str!(stringify!(
                    DeleteTrackMediaItem
                )))),
                DestroyAudioAccessor: std::mem::transmute(get_func(c_str!(stringify!(
                    DestroyAudioAccessor
                )))),
                DestroyLocalOscHandler: std::mem::transmute(get_func(c_str!(stringify!(
                    DestroyLocalOscHandler
                )))),
                DoActionShortcutDialog: std::mem::transmute(get_func(c_str!(stringify!(
                    DoActionShortcutDialog
                )))),
                Dock_UpdateDockID: std::mem::transmute(get_func(c_str!(stringify!(
                    Dock_UpdateDockID
                )))),
                DockGetPosition: std::mem::transmute(get_func(c_str!(stringify!(DockGetPosition)))),
                DockIsChildOfDock: std::mem::transmute(get_func(c_str!(stringify!(
                    DockIsChildOfDock
                )))),
                DockWindowActivate: std::mem::transmute(get_func(c_str!(stringify!(
                    DockWindowActivate
                )))),
                DockWindowAdd: std::mem::transmute(get_func(c_str!(stringify!(DockWindowAdd)))),
                DockWindowAddEx: std::mem::transmute(get_func(c_str!(stringify!(DockWindowAddEx)))),
                DockWindowRefresh: std::mem::transmute(get_func(c_str!(stringify!(
                    DockWindowRefresh
                )))),
                DockWindowRefreshForHWND: std::mem::transmute(get_func(c_str!(stringify!(
                    DockWindowRefreshForHWND
                )))),
                DockWindowRemove: std::mem::transmute(get_func(c_str!(stringify!(
                    DockWindowRemove
                )))),
                DuplicateCustomizableMenu: std::mem::transmute(get_func(c_str!(stringify!(
                    DuplicateCustomizableMenu
                )))),
                EditTempoTimeSigMarker: std::mem::transmute(get_func(c_str!(stringify!(
                    EditTempoTimeSigMarker
                )))),
                EnsureNotCompletelyOffscreen: std::mem::transmute(get_func(c_str!(stringify!(
                    EnsureNotCompletelyOffscreen
                )))),
                EnumerateFiles: std::mem::transmute(get_func(c_str!(stringify!(EnumerateFiles)))),
                EnumerateSubdirectories: std::mem::transmute(get_func(c_str!(stringify!(
                    EnumerateSubdirectories
                )))),
                EnumPitchShiftModes: std::mem::transmute(get_func(c_str!(stringify!(
                    EnumPitchShiftModes
                )))),
                EnumPitchShiftSubModes: std::mem::transmute(get_func(c_str!(stringify!(
                    EnumPitchShiftSubModes
                )))),
                EnumProjectMarkers: std::mem::transmute(get_func(c_str!(stringify!(
                    EnumProjectMarkers
                )))),
                EnumProjectMarkers2: std::mem::transmute(get_func(c_str!(stringify!(
                    EnumProjectMarkers2
                )))),
                EnumProjectMarkers3: std::mem::transmute(get_func(c_str!(stringify!(
                    EnumProjectMarkers3
                )))),
                EnumProjects: std::mem::transmute(get_func(c_str!(stringify!(EnumProjects)))),
                EnumProjExtState: std::mem::transmute(get_func(c_str!(stringify!(
                    EnumProjExtState
                )))),
                EnumRegionRenderMatrix: std::mem::transmute(get_func(c_str!(stringify!(
                    EnumRegionRenderMatrix
                )))),
                EnumTrackMIDIProgramNames: std::mem::transmute(get_func(c_str!(stringify!(
                    EnumTrackMIDIProgramNames
                )))),
                EnumTrackMIDIProgramNamesEx: std::mem::transmute(get_func(c_str!(stringify!(
                    EnumTrackMIDIProgramNamesEx
                )))),
                Envelope_Evaluate: std::mem::transmute(get_func(c_str!(stringify!(
                    Envelope_Evaluate
                )))),
                Envelope_FormatValue: std::mem::transmute(get_func(c_str!(stringify!(
                    Envelope_FormatValue
                )))),
                Envelope_GetParentTake: std::mem::transmute(get_func(c_str!(stringify!(
                    Envelope_GetParentTake
                )))),
                Envelope_GetParentTrack: std::mem::transmute(get_func(c_str!(stringify!(
                    Envelope_GetParentTrack
                )))),
                Envelope_SortPoints: std::mem::transmute(get_func(c_str!(stringify!(
                    Envelope_SortPoints
                )))),
                Envelope_SortPointsEx: std::mem::transmute(get_func(c_str!(stringify!(
                    Envelope_SortPointsEx
                )))),
                ExecProcess: std::mem::transmute(get_func(c_str!(stringify!(ExecProcess)))),
                file_exists: std::mem::transmute(get_func(c_str!(stringify!(file_exists)))),
                FindTempoTimeSigMarker: std::mem::transmute(get_func(c_str!(stringify!(
                    FindTempoTimeSigMarker
                )))),
                format_timestr: std::mem::transmute(get_func(c_str!(stringify!(format_timestr)))),
                format_timestr_len: std::mem::transmute(get_func(c_str!(stringify!(
                    format_timestr_len
                )))),
                format_timestr_pos: std::mem::transmute(get_func(c_str!(stringify!(
                    format_timestr_pos
                )))),
                FreeHeapPtr: std::mem::transmute(get_func(c_str!(stringify!(FreeHeapPtr)))),
                genGuid: std::mem::transmute(get_func(c_str!(stringify!(genGuid)))),
                get_config_var: std::mem::transmute(get_func(c_str!(stringify!(get_config_var)))),
                get_config_var_string: std::mem::transmute(get_func(c_str!(stringify!(
                    get_config_var_string
                )))),
                get_ini_file: std::mem::transmute(get_func(c_str!(stringify!(get_ini_file)))),
                get_midi_config_var: std::mem::transmute(get_func(c_str!(stringify!(
                    get_midi_config_var
                )))),
                GetActionShortcutDesc: std::mem::transmute(get_func(c_str!(stringify!(
                    GetActionShortcutDesc
                )))),
                GetActiveTake: std::mem::transmute(get_func(c_str!(stringify!(GetActiveTake)))),
                GetAllProjectPlayStates: std::mem::transmute(get_func(c_str!(stringify!(
                    GetAllProjectPlayStates
                )))),
                GetAppVersion: std::mem::transmute(get_func(c_str!(stringify!(GetAppVersion)))),
                GetArmedCommand: std::mem::transmute(get_func(c_str!(stringify!(GetArmedCommand)))),
                GetAudioAccessorEndTime: std::mem::transmute(get_func(c_str!(stringify!(
                    GetAudioAccessorEndTime
                )))),
                GetAudioAccessorHash: std::mem::transmute(get_func(c_str!(stringify!(
                    GetAudioAccessorHash
                )))),
                GetAudioAccessorSamples: std::mem::transmute(get_func(c_str!(stringify!(
                    GetAudioAccessorSamples
                )))),
                GetAudioAccessorStartTime: std::mem::transmute(get_func(c_str!(stringify!(
                    GetAudioAccessorStartTime
                )))),
                GetAudioDeviceInfo: std::mem::transmute(get_func(c_str!(stringify!(
                    GetAudioDeviceInfo
                )))),
                GetColorTheme: std::mem::transmute(get_func(c_str!(stringify!(GetColorTheme)))),
                GetColorThemeStruct: std::mem::transmute(get_func(c_str!(stringify!(
                    GetColorThemeStruct
                )))),
                GetConfigWantsDock: std::mem::transmute(get_func(c_str!(stringify!(
                    GetConfigWantsDock
                )))),
                GetContextMenu: std::mem::transmute(get_func(c_str!(stringify!(GetContextMenu)))),
                GetCurrentProjectInLoadSave: std::mem::transmute(get_func(c_str!(stringify!(
                    GetCurrentProjectInLoadSave
                )))),
                GetCursorContext: std::mem::transmute(get_func(c_str!(stringify!(
                    GetCursorContext
                )))),
                GetCursorContext2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetCursorContext2
                )))),
                GetCursorPosition: std::mem::transmute(get_func(c_str!(stringify!(
                    GetCursorPosition
                )))),
                GetCursorPositionEx: std::mem::transmute(get_func(c_str!(stringify!(
                    GetCursorPositionEx
                )))),
                GetDisplayedMediaItemColor: std::mem::transmute(get_func(c_str!(stringify!(
                    GetDisplayedMediaItemColor
                )))),
                GetDisplayedMediaItemColor2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetDisplayedMediaItemColor2
                )))),
                GetEnvelopeInfo_Value: std::mem::transmute(get_func(c_str!(stringify!(
                    GetEnvelopeInfo_Value
                )))),
                GetEnvelopeName: std::mem::transmute(get_func(c_str!(stringify!(GetEnvelopeName)))),
                GetEnvelopePoint: std::mem::transmute(get_func(c_str!(stringify!(
                    GetEnvelopePoint
                )))),
                GetEnvelopePointByTime: std::mem::transmute(get_func(c_str!(stringify!(
                    GetEnvelopePointByTime
                )))),
                GetEnvelopePointByTimeEx: std::mem::transmute(get_func(c_str!(stringify!(
                    GetEnvelopePointByTimeEx
                )))),
                GetEnvelopePointEx: std::mem::transmute(get_func(c_str!(stringify!(
                    GetEnvelopePointEx
                )))),
                GetEnvelopeScalingMode: std::mem::transmute(get_func(c_str!(stringify!(
                    GetEnvelopeScalingMode
                )))),
                GetEnvelopeStateChunk: std::mem::transmute(get_func(c_str!(stringify!(
                    GetEnvelopeStateChunk
                )))),
                GetExePath: std::mem::transmute(get_func(c_str!(stringify!(GetExePath)))),
                GetExtState: std::mem::transmute(get_func(c_str!(stringify!(GetExtState)))),
                GetFocusedFX: std::mem::transmute(get_func(c_str!(stringify!(GetFocusedFX)))),
                GetFreeDiskSpaceForRecordPath: std::mem::transmute(get_func(c_str!(stringify!(
                    GetFreeDiskSpaceForRecordPath
                )))),
                GetFXEnvelope: std::mem::transmute(get_func(c_str!(stringify!(GetFXEnvelope)))),
                GetGlobalAutomationOverride: std::mem::transmute(get_func(c_str!(stringify!(
                    GetGlobalAutomationOverride
                )))),
                GetHZoomLevel: std::mem::transmute(get_func(c_str!(stringify!(GetHZoomLevel)))),
                GetIconThemePointer: std::mem::transmute(get_func(c_str!(stringify!(
                    GetIconThemePointer
                )))),
                GetIconThemePointerForDPI: std::mem::transmute(get_func(c_str!(stringify!(
                    GetIconThemePointerForDPI
                )))),
                GetIconThemeStruct: std::mem::transmute(get_func(c_str!(stringify!(
                    GetIconThemeStruct
                )))),
                GetInputChannelName: std::mem::transmute(get_func(c_str!(stringify!(
                    GetInputChannelName
                )))),
                GetInputOutputLatency: std::mem::transmute(get_func(c_str!(stringify!(
                    GetInputOutputLatency
                )))),
                GetItemEditingTime2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetItemEditingTime2
                )))),
                GetItemFromPoint: std::mem::transmute(get_func(c_str!(stringify!(
                    GetItemFromPoint
                )))),
                GetItemProjectContext: std::mem::transmute(get_func(c_str!(stringify!(
                    GetItemProjectContext
                )))),
                GetItemStateChunk: std::mem::transmute(get_func(c_str!(stringify!(
                    GetItemStateChunk
                )))),
                GetLastColorThemeFile: std::mem::transmute(get_func(c_str!(stringify!(
                    GetLastColorThemeFile
                )))),
                GetLastMarkerAndCurRegion: std::mem::transmute(get_func(c_str!(stringify!(
                    GetLastMarkerAndCurRegion
                )))),
                GetLastTouchedFX: std::mem::transmute(get_func(c_str!(stringify!(
                    GetLastTouchedFX
                )))),
                GetLastTouchedTrack: std::mem::transmute(get_func(c_str!(stringify!(
                    GetLastTouchedTrack
                )))),
                GetMainHwnd: std::mem::transmute(get_func(c_str!(stringify!(GetMainHwnd)))),
                GetMasterMuteSoloFlags: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMasterMuteSoloFlags
                )))),
                GetMasterTrack: std::mem::transmute(get_func(c_str!(stringify!(GetMasterTrack)))),
                GetMasterTrackVisibility: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMasterTrackVisibility
                )))),
                GetMaxMidiInputs: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMaxMidiInputs
                )))),
                GetMaxMidiOutputs: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMaxMidiOutputs
                )))),
                GetMediaItem: std::mem::transmute(get_func(c_str!(stringify!(GetMediaItem)))),
                GetMediaItem_Track: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaItem_Track
                )))),
                GetMediaItemInfo_Value: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaItemInfo_Value
                )))),
                GetMediaItemNumTakes: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaItemNumTakes
                )))),
                GetMediaItemTake: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaItemTake
                )))),
                GetMediaItemTake_Item: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaItemTake_Item
                )))),
                GetMediaItemTake_Peaks: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaItemTake_Peaks
                )))),
                GetMediaItemTake_Source: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaItemTake_Source
                )))),
                GetMediaItemTake_Track: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaItemTake_Track
                )))),
                GetMediaItemTakeByGUID: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaItemTakeByGUID
                )))),
                GetMediaItemTakeInfo_Value: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaItemTakeInfo_Value
                )))),
                GetMediaItemTrack: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaItemTrack
                )))),
                GetMediaSourceFileName: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaSourceFileName
                )))),
                GetMediaSourceLength: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaSourceLength
                )))),
                GetMediaSourceNumChannels: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaSourceNumChannels
                )))),
                GetMediaSourceParent: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaSourceParent
                )))),
                GetMediaSourceSampleRate: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaSourceSampleRate
                )))),
                GetMediaSourceType: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaSourceType
                )))),
                GetMediaTrackInfo_Value: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMediaTrackInfo_Value
                )))),
                GetMIDIInputName: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMIDIInputName
                )))),
                GetMIDIOutputName: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMIDIOutputName
                )))),
                GetMixerScroll: std::mem::transmute(get_func(c_str!(stringify!(GetMixerScroll)))),
                GetMouseModifier: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMouseModifier
                )))),
                GetMousePosition: std::mem::transmute(get_func(c_str!(stringify!(
                    GetMousePosition
                )))),
                GetNumAudioInputs: std::mem::transmute(get_func(c_str!(stringify!(
                    GetNumAudioInputs
                )))),
                GetNumAudioOutputs: std::mem::transmute(get_func(c_str!(stringify!(
                    GetNumAudioOutputs
                )))),
                GetNumMIDIInputs: std::mem::transmute(get_func(c_str!(stringify!(
                    GetNumMIDIInputs
                )))),
                GetNumMIDIOutputs: std::mem::transmute(get_func(c_str!(stringify!(
                    GetNumMIDIOutputs
                )))),
                GetNumTracks: std::mem::transmute(get_func(c_str!(stringify!(GetNumTracks)))),
                GetOS: std::mem::transmute(get_func(c_str!(stringify!(GetOS)))),
                GetOutputChannelName: std::mem::transmute(get_func(c_str!(stringify!(
                    GetOutputChannelName
                )))),
                GetOutputLatency: std::mem::transmute(get_func(c_str!(stringify!(
                    GetOutputLatency
                )))),
                GetParentTrack: std::mem::transmute(get_func(c_str!(stringify!(GetParentTrack)))),
                GetPeakFileName: std::mem::transmute(get_func(c_str!(stringify!(GetPeakFileName)))),
                GetPeakFileNameEx: std::mem::transmute(get_func(c_str!(stringify!(
                    GetPeakFileNameEx
                )))),
                GetPeakFileNameEx2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetPeakFileNameEx2
                )))),
                GetPeaksBitmap: std::mem::transmute(get_func(c_str!(stringify!(GetPeaksBitmap)))),
                GetPlayPosition: std::mem::transmute(get_func(c_str!(stringify!(GetPlayPosition)))),
                GetPlayPosition2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetPlayPosition2
                )))),
                GetPlayPosition2Ex: std::mem::transmute(get_func(c_str!(stringify!(
                    GetPlayPosition2Ex
                )))),
                GetPlayPositionEx: std::mem::transmute(get_func(c_str!(stringify!(
                    GetPlayPositionEx
                )))),
                GetPlayState: std::mem::transmute(get_func(c_str!(stringify!(GetPlayState)))),
                GetPlayStateEx: std::mem::transmute(get_func(c_str!(stringify!(GetPlayStateEx)))),
                GetPreferredDiskReadMode: std::mem::transmute(get_func(c_str!(stringify!(
                    GetPreferredDiskReadMode
                )))),
                GetPreferredDiskReadModePeak: std::mem::transmute(get_func(c_str!(stringify!(
                    GetPreferredDiskReadModePeak
                )))),
                GetPreferredDiskWriteMode: std::mem::transmute(get_func(c_str!(stringify!(
                    GetPreferredDiskWriteMode
                )))),
                GetProjectLength: std::mem::transmute(get_func(c_str!(stringify!(
                    GetProjectLength
                )))),
                GetProjectName: std::mem::transmute(get_func(c_str!(stringify!(GetProjectName)))),
                GetProjectPath: std::mem::transmute(get_func(c_str!(stringify!(GetProjectPath)))),
                GetProjectPathEx: std::mem::transmute(get_func(c_str!(stringify!(
                    GetProjectPathEx
                )))),
                GetProjectStateChangeCount: std::mem::transmute(get_func(c_str!(stringify!(
                    GetProjectStateChangeCount
                )))),
                GetProjectTimeOffset: std::mem::transmute(get_func(c_str!(stringify!(
                    GetProjectTimeOffset
                )))),
                GetProjectTimeSignature: std::mem::transmute(get_func(c_str!(stringify!(
                    GetProjectTimeSignature
                )))),
                GetProjectTimeSignature2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetProjectTimeSignature2
                )))),
                GetProjExtState: std::mem::transmute(get_func(c_str!(stringify!(GetProjExtState)))),
                GetResourcePath: std::mem::transmute(get_func(c_str!(stringify!(GetResourcePath)))),
                GetSelectedEnvelope: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSelectedEnvelope
                )))),
                GetSelectedMediaItem: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSelectedMediaItem
                )))),
                GetSelectedTrack: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSelectedTrack
                )))),
                GetSelectedTrack2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSelectedTrack2
                )))),
                GetSelectedTrackEnvelope: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSelectedTrackEnvelope
                )))),
                GetSet_ArrangeView2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSet_ArrangeView2
                )))),
                GetSet_LoopTimeRange: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSet_LoopTimeRange
                )))),
                GetSet_LoopTimeRange2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSet_LoopTimeRange2
                )))),
                GetSetAutomationItemInfo: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetAutomationItemInfo
                )))),
                GetSetAutomationItemInfo_String: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetAutomationItemInfo_String
                )))),
                GetSetEnvelopeInfo_String: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetEnvelopeInfo_String
                )))),
                GetSetEnvelopeState: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetEnvelopeState
                )))),
                GetSetEnvelopeState2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetEnvelopeState2
                )))),
                GetSetItemState: std::mem::transmute(get_func(c_str!(stringify!(GetSetItemState)))),
                GetSetItemState2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetItemState2
                )))),
                GetSetMediaItemInfo: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetMediaItemInfo
                )))),
                GetSetMediaItemInfo_String: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetMediaItemInfo_String
                )))),
                GetSetMediaItemTakeInfo: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetMediaItemTakeInfo
                )))),
                GetSetMediaItemTakeInfo_String: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetMediaItemTakeInfo_String
                )))),
                GetSetMediaTrackInfo: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetMediaTrackInfo
                )))),
                GetSetMediaTrackInfo_String: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetMediaTrackInfo_String
                )))),
                GetSetObjectState: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetObjectState
                )))),
                GetSetObjectState2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetObjectState2
                )))),
                GetSetProjectAuthor: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetProjectAuthor
                )))),
                GetSetProjectGrid: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetProjectGrid
                )))),
                GetSetProjectInfo: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetProjectInfo
                )))),
                GetSetProjectInfo_String: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetProjectInfo_String
                )))),
                GetSetProjectNotes: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetProjectNotes
                )))),
                GetSetRepeat: std::mem::transmute(get_func(c_str!(stringify!(GetSetRepeat)))),
                GetSetRepeatEx: std::mem::transmute(get_func(c_str!(stringify!(GetSetRepeatEx)))),
                GetSetTrackGroupMembership: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetTrackGroupMembership
                )))),
                GetSetTrackGroupMembershipHigh: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetTrackGroupMembershipHigh
                )))),
                GetSetTrackMIDISupportFile: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetTrackMIDISupportFile
                )))),
                GetSetTrackSendInfo: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetTrackSendInfo
                )))),
                GetSetTrackSendInfo_String: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetTrackSendInfo_String
                )))),
                GetSetTrackState: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetTrackState
                )))),
                GetSetTrackState2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSetTrackState2
                )))),
                GetSubProjectFromSource: std::mem::transmute(get_func(c_str!(stringify!(
                    GetSubProjectFromSource
                )))),
                GetTake: std::mem::transmute(get_func(c_str!(stringify!(GetTake)))),
                GetTakeEnvelope: std::mem::transmute(get_func(c_str!(stringify!(GetTakeEnvelope)))),
                GetTakeEnvelopeByName: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTakeEnvelopeByName
                )))),
                GetTakeName: std::mem::transmute(get_func(c_str!(stringify!(GetTakeName)))),
                GetTakeNumStretchMarkers: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTakeNumStretchMarkers
                )))),
                GetTakeStretchMarker: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTakeStretchMarker
                )))),
                GetTakeStretchMarkerSlope: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTakeStretchMarkerSlope
                )))),
                GetTCPFXParm: std::mem::transmute(get_func(c_str!(stringify!(GetTCPFXParm)))),
                GetTempoMatchPlayRate: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTempoMatchPlayRate
                )))),
                GetTempoTimeSigMarker: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTempoTimeSigMarker
                )))),
                GetToggleCommandState: std::mem::transmute(get_func(c_str!(stringify!(
                    GetToggleCommandState
                )))),
                GetToggleCommandState2: std::mem::transmute(get_func(c_str!(stringify!(
                    GetToggleCommandState2
                )))),
                GetToggleCommandStateEx: std::mem::transmute(get_func(c_str!(stringify!(
                    GetToggleCommandStateEx
                )))),
                GetToggleCommandStateThroughHooks: std::mem::transmute(get_func(c_str!(
                    stringify!(GetToggleCommandStateThroughHooks)
                ))),
                GetTooltipWindow: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTooltipWindow
                )))),
                GetTrack: std::mem::transmute(get_func(c_str!(stringify!(GetTrack)))),
                GetTrackAutomationMode: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackAutomationMode
                )))),
                GetTrackColor: std::mem::transmute(get_func(c_str!(stringify!(GetTrackColor)))),
                GetTrackDepth: std::mem::transmute(get_func(c_str!(stringify!(GetTrackDepth)))),
                GetTrackEnvelope: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackEnvelope
                )))),
                GetTrackEnvelopeByChunkName: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackEnvelopeByChunkName
                )))),
                GetTrackEnvelopeByName: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackEnvelopeByName
                )))),
                GetTrackFromPoint: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackFromPoint
                )))),
                GetTrackGUID: std::mem::transmute(get_func(c_str!(stringify!(GetTrackGUID)))),
                GetTrackInfo: std::mem::transmute(get_func(c_str!(stringify!(GetTrackInfo)))),
                GetTrackMediaItem: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackMediaItem
                )))),
                GetTrackMIDILyrics: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackMIDILyrics
                )))),
                GetTrackMIDINoteName: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackMIDINoteName
                )))),
                GetTrackMIDINoteNameEx: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackMIDINoteNameEx
                )))),
                GetTrackMIDINoteRange: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackMIDINoteRange
                )))),
                GetTrackName: std::mem::transmute(get_func(c_str!(stringify!(GetTrackName)))),
                GetTrackNumMediaItems: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackNumMediaItems
                )))),
                GetTrackNumSends: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackNumSends
                )))),
                GetTrackReceiveName: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackReceiveName
                )))),
                GetTrackReceiveUIMute: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackReceiveUIMute
                )))),
                GetTrackReceiveUIVolPan: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackReceiveUIVolPan
                )))),
                GetTrackSendInfo_Value: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackSendInfo_Value
                )))),
                GetTrackSendName: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackSendName
                )))),
                GetTrackSendUIMute: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackSendUIMute
                )))),
                GetTrackSendUIVolPan: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackSendUIVolPan
                )))),
                GetTrackState: std::mem::transmute(get_func(c_str!(stringify!(GetTrackState)))),
                GetTrackStateChunk: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackStateChunk
                )))),
                GetTrackUIMute: std::mem::transmute(get_func(c_str!(stringify!(GetTrackUIMute)))),
                GetTrackUIPan: std::mem::transmute(get_func(c_str!(stringify!(GetTrackUIPan)))),
                GetTrackUIVolPan: std::mem::transmute(get_func(c_str!(stringify!(
                    GetTrackUIVolPan
                )))),
                GetUnderrunTime: std::mem::transmute(get_func(c_str!(stringify!(GetUnderrunTime)))),
                GetUserFileNameForRead: std::mem::transmute(get_func(c_str!(stringify!(
                    GetUserFileNameForRead
                )))),
                GetUserInputs: std::mem::transmute(get_func(c_str!(stringify!(GetUserInputs)))),
                GoToMarker: std::mem::transmute(get_func(c_str!(stringify!(GoToMarker)))),
                GoToRegion: std::mem::transmute(get_func(c_str!(stringify!(GoToRegion)))),
                GR_SelectColor: std::mem::transmute(get_func(c_str!(stringify!(GR_SelectColor)))),
                GSC_mainwnd: std::mem::transmute(get_func(c_str!(stringify!(GSC_mainwnd)))),
                guidToString: std::mem::transmute(get_func(c_str!(stringify!(guidToString)))),
                HasExtState: std::mem::transmute(get_func(c_str!(stringify!(HasExtState)))),
                HasTrackMIDIPrograms: std::mem::transmute(get_func(c_str!(stringify!(
                    HasTrackMIDIPrograms
                )))),
                HasTrackMIDIProgramsEx: std::mem::transmute(get_func(c_str!(stringify!(
                    HasTrackMIDIProgramsEx
                )))),
                Help_Set: std::mem::transmute(get_func(c_str!(stringify!(Help_Set)))),
                HiresPeaksFromSource: std::mem::transmute(get_func(c_str!(stringify!(
                    HiresPeaksFromSource
                )))),
                image_resolve_fn: std::mem::transmute(get_func(c_str!(stringify!(
                    image_resolve_fn
                )))),
                InsertAutomationItem: std::mem::transmute(get_func(c_str!(stringify!(
                    InsertAutomationItem
                )))),
                InsertEnvelopePoint: std::mem::transmute(get_func(c_str!(stringify!(
                    InsertEnvelopePoint
                )))),
                InsertEnvelopePointEx: std::mem::transmute(get_func(c_str!(stringify!(
                    InsertEnvelopePointEx
                )))),
                InsertMedia: std::mem::transmute(get_func(c_str!(stringify!(InsertMedia)))),
                InsertMediaSection: std::mem::transmute(get_func(c_str!(stringify!(
                    InsertMediaSection
                )))),
                InsertTrackAtIndex: std::mem::transmute(get_func(c_str!(stringify!(
                    InsertTrackAtIndex
                )))),
                IsInRealTimeAudio: std::mem::transmute(get_func(c_str!(stringify!(
                    IsInRealTimeAudio
                )))),
                IsItemTakeActiveForPlayback: std::mem::transmute(get_func(c_str!(stringify!(
                    IsItemTakeActiveForPlayback
                )))),
                IsMediaExtension: std::mem::transmute(get_func(c_str!(stringify!(
                    IsMediaExtension
                )))),
                IsMediaItemSelected: std::mem::transmute(get_func(c_str!(stringify!(
                    IsMediaItemSelected
                )))),
                IsProjectDirty: std::mem::transmute(get_func(c_str!(stringify!(IsProjectDirty)))),
                IsREAPER: std::mem::transmute(get_func(c_str!(stringify!(IsREAPER)))),
                IsTrackSelected: std::mem::transmute(get_func(c_str!(stringify!(IsTrackSelected)))),
                IsTrackVisible: std::mem::transmute(get_func(c_str!(stringify!(IsTrackVisible)))),
                joystick_create: std::mem::transmute(get_func(c_str!(stringify!(joystick_create)))),
                joystick_destroy: std::mem::transmute(get_func(c_str!(stringify!(
                    joystick_destroy
                )))),
                joystick_enum: std::mem::transmute(get_func(c_str!(stringify!(joystick_enum)))),
                joystick_getaxis: std::mem::transmute(get_func(c_str!(stringify!(
                    joystick_getaxis
                )))),
                joystick_getbuttonmask: std::mem::transmute(get_func(c_str!(stringify!(
                    joystick_getbuttonmask
                )))),
                joystick_getinfo: std::mem::transmute(get_func(c_str!(stringify!(
                    joystick_getinfo
                )))),
                joystick_getpov: std::mem::transmute(get_func(c_str!(stringify!(joystick_getpov)))),
                joystick_update: std::mem::transmute(get_func(c_str!(stringify!(joystick_update)))),
                kbd_enumerateActions: std::mem::transmute(get_func(c_str!(stringify!(
                    kbd_enumerateActions
                )))),
                kbd_formatKeyName: std::mem::transmute(get_func(c_str!(stringify!(
                    kbd_formatKeyName
                )))),
                kbd_getCommandName: std::mem::transmute(get_func(c_str!(stringify!(
                    kbd_getCommandName
                )))),
                kbd_getTextFromCmd: std::mem::transmute(get_func(c_str!(stringify!(
                    kbd_getTextFromCmd
                )))),
                KBD_OnMainActionEx: std::mem::transmute(get_func(c_str!(stringify!(
                    KBD_OnMainActionEx
                )))),
                kbd_OnMidiEvent: std::mem::transmute(get_func(c_str!(stringify!(kbd_OnMidiEvent)))),
                kbd_OnMidiList: std::mem::transmute(get_func(c_str!(stringify!(kbd_OnMidiList)))),
                kbd_ProcessActionsMenu: std::mem::transmute(get_func(c_str!(stringify!(
                    kbd_ProcessActionsMenu
                )))),
                kbd_processMidiEventActionEx: std::mem::transmute(get_func(c_str!(stringify!(
                    kbd_processMidiEventActionEx
                )))),
                kbd_reprocessMenu: std::mem::transmute(get_func(c_str!(stringify!(
                    kbd_reprocessMenu
                )))),
                kbd_RunCommandThroughHooks: std::mem::transmute(get_func(c_str!(stringify!(
                    kbd_RunCommandThroughHooks
                )))),
                kbd_translateAccelerator: std::mem::transmute(get_func(c_str!(stringify!(
                    kbd_translateAccelerator
                )))),
                kbd_translateMouse: std::mem::transmute(get_func(c_str!(stringify!(
                    kbd_translateMouse
                )))),
                LICE__Destroy: std::mem::transmute(get_func(c_str!(stringify!(LICE__Destroy)))),
                LICE__DestroyFont: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE__DestroyFont
                )))),
                LICE__DrawText: std::mem::transmute(get_func(c_str!(stringify!(LICE__DrawText)))),
                LICE__GetBits: std::mem::transmute(get_func(c_str!(stringify!(LICE__GetBits)))),
                LICE__GetDC: std::mem::transmute(get_func(c_str!(stringify!(LICE__GetDC)))),
                LICE__GetHeight: std::mem::transmute(get_func(c_str!(stringify!(LICE__GetHeight)))),
                LICE__GetRowSpan: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE__GetRowSpan
                )))),
                LICE__GetWidth: std::mem::transmute(get_func(c_str!(stringify!(LICE__GetWidth)))),
                LICE__IsFlipped: std::mem::transmute(get_func(c_str!(stringify!(LICE__IsFlipped)))),
                LICE__resize: std::mem::transmute(get_func(c_str!(stringify!(LICE__resize)))),
                LICE__SetBkColor: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE__SetBkColor
                )))),
                LICE__SetFromHFont: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE__SetFromHFont
                )))),
                LICE__SetTextColor: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE__SetTextColor
                )))),
                LICE__SetTextCombineMode: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE__SetTextCombineMode
                )))),
                LICE_Arc: std::mem::transmute(get_func(c_str!(stringify!(LICE_Arc)))),
                LICE_Blit: std::mem::transmute(get_func(c_str!(stringify!(LICE_Blit)))),
                LICE_Blur: std::mem::transmute(get_func(c_str!(stringify!(LICE_Blur)))),
                LICE_BorderedRect: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE_BorderedRect
                )))),
                LICE_Circle: std::mem::transmute(get_func(c_str!(stringify!(LICE_Circle)))),
                LICE_Clear: std::mem::transmute(get_func(c_str!(stringify!(LICE_Clear)))),
                LICE_ClearRect: std::mem::transmute(get_func(c_str!(stringify!(LICE_ClearRect)))),
                LICE_ClipLine: std::mem::transmute(get_func(c_str!(stringify!(LICE_ClipLine)))),
                LICE_Copy: std::mem::transmute(get_func(c_str!(stringify!(LICE_Copy)))),
                LICE_CreateBitmap: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE_CreateBitmap
                )))),
                LICE_CreateFont: std::mem::transmute(get_func(c_str!(stringify!(LICE_CreateFont)))),
                LICE_DrawCBezier: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE_DrawCBezier
                )))),
                LICE_DrawChar: std::mem::transmute(get_func(c_str!(stringify!(LICE_DrawChar)))),
                LICE_DrawGlyph: std::mem::transmute(get_func(c_str!(stringify!(LICE_DrawGlyph)))),
                LICE_DrawRect: std::mem::transmute(get_func(c_str!(stringify!(LICE_DrawRect)))),
                LICE_DrawText: std::mem::transmute(get_func(c_str!(stringify!(LICE_DrawText)))),
                LICE_FillCBezier: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE_FillCBezier
                )))),
                LICE_FillCircle: std::mem::transmute(get_func(c_str!(stringify!(LICE_FillCircle)))),
                LICE_FillConvexPolygon: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE_FillConvexPolygon
                )))),
                LICE_FillRect: std::mem::transmute(get_func(c_str!(stringify!(LICE_FillRect)))),
                LICE_FillTrapezoid: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE_FillTrapezoid
                )))),
                LICE_FillTriangle: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE_FillTriangle
                )))),
                LICE_GetPixel: std::mem::transmute(get_func(c_str!(stringify!(LICE_GetPixel)))),
                LICE_GradRect: std::mem::transmute(get_func(c_str!(stringify!(LICE_GradRect)))),
                LICE_Line: std::mem::transmute(get_func(c_str!(stringify!(LICE_Line)))),
                LICE_LineInt: std::mem::transmute(get_func(c_str!(stringify!(LICE_LineInt)))),
                LICE_LoadPNG: std::mem::transmute(get_func(c_str!(stringify!(LICE_LoadPNG)))),
                LICE_LoadPNGFromResource: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE_LoadPNGFromResource
                )))),
                LICE_MeasureText: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE_MeasureText
                )))),
                LICE_MultiplyAddRect: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE_MultiplyAddRect
                )))),
                LICE_PutPixel: std::mem::transmute(get_func(c_str!(stringify!(LICE_PutPixel)))),
                LICE_RotatedBlit: std::mem::transmute(get_func(c_str!(stringify!(
                    LICE_RotatedBlit
                )))),
                LICE_RoundRect: std::mem::transmute(get_func(c_str!(stringify!(LICE_RoundRect)))),
                LICE_ScaledBlit: std::mem::transmute(get_func(c_str!(stringify!(LICE_ScaledBlit)))),
                LICE_SimpleFill: std::mem::transmute(get_func(c_str!(stringify!(LICE_SimpleFill)))),
                Loop_OnArrow: std::mem::transmute(get_func(c_str!(stringify!(Loop_OnArrow)))),
                Main_OnCommand: std::mem::transmute(get_func(c_str!(stringify!(Main_OnCommand)))),
                Main_OnCommandEx: std::mem::transmute(get_func(c_str!(stringify!(
                    Main_OnCommandEx
                )))),
                Main_openProject: std::mem::transmute(get_func(c_str!(stringify!(
                    Main_openProject
                )))),
                Main_SaveProject: std::mem::transmute(get_func(c_str!(stringify!(
                    Main_SaveProject
                )))),
                Main_UpdateLoopInfo: std::mem::transmute(get_func(c_str!(stringify!(
                    Main_UpdateLoopInfo
                )))),
                MarkProjectDirty: std::mem::transmute(get_func(c_str!(stringify!(
                    MarkProjectDirty
                )))),
                MarkTrackItemsDirty: std::mem::transmute(get_func(c_str!(stringify!(
                    MarkTrackItemsDirty
                )))),
                Master_GetPlayRate: std::mem::transmute(get_func(c_str!(stringify!(
                    Master_GetPlayRate
                )))),
                Master_GetPlayRateAtTime: std::mem::transmute(get_func(c_str!(stringify!(
                    Master_GetPlayRateAtTime
                )))),
                Master_GetTempo: std::mem::transmute(get_func(c_str!(stringify!(Master_GetTempo)))),
                Master_NormalizePlayRate: std::mem::transmute(get_func(c_str!(stringify!(
                    Master_NormalizePlayRate
                )))),
                Master_NormalizeTempo: std::mem::transmute(get_func(c_str!(stringify!(
                    Master_NormalizeTempo
                )))),
                MB: std::mem::transmute(get_func(c_str!(stringify!(MB)))),
                MediaItemDescendsFromTrack: std::mem::transmute(get_func(c_str!(stringify!(
                    MediaItemDescendsFromTrack
                )))),
                MIDI_CountEvts: std::mem::transmute(get_func(c_str!(stringify!(MIDI_CountEvts)))),
                MIDI_DeleteCC: std::mem::transmute(get_func(c_str!(stringify!(MIDI_DeleteCC)))),
                MIDI_DeleteEvt: std::mem::transmute(get_func(c_str!(stringify!(MIDI_DeleteEvt)))),
                MIDI_DeleteNote: std::mem::transmute(get_func(c_str!(stringify!(MIDI_DeleteNote)))),
                MIDI_DeleteTextSysexEvt: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_DeleteTextSysexEvt
                )))),
                MIDI_DisableSort: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_DisableSort
                )))),
                MIDI_EnumSelCC: std::mem::transmute(get_func(c_str!(stringify!(MIDI_EnumSelCC)))),
                MIDI_EnumSelEvts: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_EnumSelEvts
                )))),
                MIDI_EnumSelNotes: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_EnumSelNotes
                )))),
                MIDI_EnumSelTextSysexEvts: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_EnumSelTextSysexEvts
                )))),
                MIDI_eventlist_Create: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_eventlist_Create
                )))),
                MIDI_eventlist_Destroy: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_eventlist_Destroy
                )))),
                MIDI_GetAllEvts: std::mem::transmute(get_func(c_str!(stringify!(MIDI_GetAllEvts)))),
                MIDI_GetCC: std::mem::transmute(get_func(c_str!(stringify!(MIDI_GetCC)))),
                MIDI_GetCCShape: std::mem::transmute(get_func(c_str!(stringify!(MIDI_GetCCShape)))),
                MIDI_GetEvt: std::mem::transmute(get_func(c_str!(stringify!(MIDI_GetEvt)))),
                MIDI_GetGrid: std::mem::transmute(get_func(c_str!(stringify!(MIDI_GetGrid)))),
                MIDI_GetHash: std::mem::transmute(get_func(c_str!(stringify!(MIDI_GetHash)))),
                MIDI_GetNote: std::mem::transmute(get_func(c_str!(stringify!(MIDI_GetNote)))),
                MIDI_GetPPQPos_EndOfMeasure: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_GetPPQPos_EndOfMeasure
                )))),
                MIDI_GetPPQPos_StartOfMeasure: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_GetPPQPos_StartOfMeasure
                )))),
                MIDI_GetPPQPosFromProjQN: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_GetPPQPosFromProjQN
                )))),
                MIDI_GetPPQPosFromProjTime: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_GetPPQPosFromProjTime
                )))),
                MIDI_GetProjQNFromPPQPos: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_GetProjQNFromPPQPos
                )))),
                MIDI_GetProjTimeFromPPQPos: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_GetProjTimeFromPPQPos
                )))),
                MIDI_GetScale: std::mem::transmute(get_func(c_str!(stringify!(MIDI_GetScale)))),
                MIDI_GetTextSysexEvt: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_GetTextSysexEvt
                )))),
                MIDI_GetTrackHash: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_GetTrackHash
                )))),
                MIDI_InsertCC: std::mem::transmute(get_func(c_str!(stringify!(MIDI_InsertCC)))),
                MIDI_InsertEvt: std::mem::transmute(get_func(c_str!(stringify!(MIDI_InsertEvt)))),
                MIDI_InsertNote: std::mem::transmute(get_func(c_str!(stringify!(MIDI_InsertNote)))),
                MIDI_InsertTextSysexEvt: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_InsertTextSysexEvt
                )))),
                midi_reinit: std::mem::transmute(get_func(c_str!(stringify!(midi_reinit)))),
                MIDI_SelectAll: std::mem::transmute(get_func(c_str!(stringify!(MIDI_SelectAll)))),
                MIDI_SetAllEvts: std::mem::transmute(get_func(c_str!(stringify!(MIDI_SetAllEvts)))),
                MIDI_SetCC: std::mem::transmute(get_func(c_str!(stringify!(MIDI_SetCC)))),
                MIDI_SetCCShape: std::mem::transmute(get_func(c_str!(stringify!(MIDI_SetCCShape)))),
                MIDI_SetEvt: std::mem::transmute(get_func(c_str!(stringify!(MIDI_SetEvt)))),
                MIDI_SetItemExtents: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_SetItemExtents
                )))),
                MIDI_SetNote: std::mem::transmute(get_func(c_str!(stringify!(MIDI_SetNote)))),
                MIDI_SetTextSysexEvt: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDI_SetTextSysexEvt
                )))),
                MIDI_Sort: std::mem::transmute(get_func(c_str!(stringify!(MIDI_Sort)))),
                MIDIEditor_GetActive: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDIEditor_GetActive
                )))),
                MIDIEditor_GetMode: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDIEditor_GetMode
                )))),
                MIDIEditor_GetSetting_int: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDIEditor_GetSetting_int
                )))),
                MIDIEditor_GetSetting_str: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDIEditor_GetSetting_str
                )))),
                MIDIEditor_GetTake: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDIEditor_GetTake
                )))),
                MIDIEditor_LastFocused_OnCommand: std::mem::transmute(get_func(c_str!(
                    stringify!(MIDIEditor_LastFocused_OnCommand)
                ))),
                MIDIEditor_OnCommand: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDIEditor_OnCommand
                )))),
                MIDIEditor_SetSetting_int: std::mem::transmute(get_func(c_str!(stringify!(
                    MIDIEditor_SetSetting_int
                )))),
                mkpanstr: std::mem::transmute(get_func(c_str!(stringify!(mkpanstr)))),
                mkvolpanstr: std::mem::transmute(get_func(c_str!(stringify!(mkvolpanstr)))),
                mkvolstr: std::mem::transmute(get_func(c_str!(stringify!(mkvolstr)))),
                MoveEditCursor: std::mem::transmute(get_func(c_str!(stringify!(MoveEditCursor)))),
                MoveMediaItemToTrack: std::mem::transmute(get_func(c_str!(stringify!(
                    MoveMediaItemToTrack
                )))),
                MuteAllTracks: std::mem::transmute(get_func(c_str!(stringify!(MuteAllTracks)))),
                my_getViewport: std::mem::transmute(get_func(c_str!(stringify!(my_getViewport)))),
                NamedCommandLookup: std::mem::transmute(get_func(c_str!(stringify!(
                    NamedCommandLookup
                )))),
                OnPauseButton: std::mem::transmute(get_func(c_str!(stringify!(OnPauseButton)))),
                OnPauseButtonEx: std::mem::transmute(get_func(c_str!(stringify!(OnPauseButtonEx)))),
                OnPlayButton: std::mem::transmute(get_func(c_str!(stringify!(OnPlayButton)))),
                OnPlayButtonEx: std::mem::transmute(get_func(c_str!(stringify!(OnPlayButtonEx)))),
                OnStopButton: std::mem::transmute(get_func(c_str!(stringify!(OnStopButton)))),
                OnStopButtonEx: std::mem::transmute(get_func(c_str!(stringify!(OnStopButtonEx)))),
                OpenColorThemeFile: std::mem::transmute(get_func(c_str!(stringify!(
                    OpenColorThemeFile
                )))),
                OpenMediaExplorer: std::mem::transmute(get_func(c_str!(stringify!(
                    OpenMediaExplorer
                )))),
                OscLocalMessageToHost: std::mem::transmute(get_func(c_str!(stringify!(
                    OscLocalMessageToHost
                )))),
                parse_timestr: std::mem::transmute(get_func(c_str!(stringify!(parse_timestr)))),
                parse_timestr_len: std::mem::transmute(get_func(c_str!(stringify!(
                    parse_timestr_len
                )))),
                parse_timestr_pos: std::mem::transmute(get_func(c_str!(stringify!(
                    parse_timestr_pos
                )))),
                parsepanstr: std::mem::transmute(get_func(c_str!(stringify!(parsepanstr)))),
                PCM_Sink_Create: std::mem::transmute(get_func(c_str!(stringify!(PCM_Sink_Create)))),
                PCM_Sink_CreateEx: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Sink_CreateEx
                )))),
                PCM_Sink_CreateMIDIFile: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Sink_CreateMIDIFile
                )))),
                PCM_Sink_CreateMIDIFileEx: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Sink_CreateMIDIFileEx
                )))),
                PCM_Sink_Enum: std::mem::transmute(get_func(c_str!(stringify!(PCM_Sink_Enum)))),
                PCM_Sink_GetExtension: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Sink_GetExtension
                )))),
                PCM_Sink_ShowConfig: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Sink_ShowConfig
                )))),
                PCM_Source_CreateFromFile: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Source_CreateFromFile
                )))),
                PCM_Source_CreateFromFileEx: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Source_CreateFromFileEx
                )))),
                PCM_Source_CreateFromSimple: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Source_CreateFromSimple
                )))),
                PCM_Source_CreateFromType: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Source_CreateFromType
                )))),
                PCM_Source_Destroy: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Source_Destroy
                )))),
                PCM_Source_GetPeaks: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Source_GetPeaks
                )))),
                PCM_Source_GetSectionInfo: std::mem::transmute(get_func(c_str!(stringify!(
                    PCM_Source_GetSectionInfo
                )))),
                PeakBuild_Create: std::mem::transmute(get_func(c_str!(stringify!(
                    PeakBuild_Create
                )))),
                PeakBuild_CreateEx: std::mem::transmute(get_func(c_str!(stringify!(
                    PeakBuild_CreateEx
                )))),
                PeakGet_Create: std::mem::transmute(get_func(c_str!(stringify!(PeakGet_Create)))),
                PitchShiftSubModeMenu: std::mem::transmute(get_func(c_str!(stringify!(
                    PitchShiftSubModeMenu
                )))),
                PlayPreview: std::mem::transmute(get_func(c_str!(stringify!(PlayPreview)))),
                PlayPreviewEx: std::mem::transmute(get_func(c_str!(stringify!(PlayPreviewEx)))),
                PlayTrackPreview: std::mem::transmute(get_func(c_str!(stringify!(
                    PlayTrackPreview
                )))),
                PlayTrackPreview2: std::mem::transmute(get_func(c_str!(stringify!(
                    PlayTrackPreview2
                )))),
                PlayTrackPreview2Ex: std::mem::transmute(get_func(c_str!(stringify!(
                    PlayTrackPreview2Ex
                )))),
                plugin_getapi: std::mem::transmute(get_func(c_str!(stringify!(plugin_getapi)))),
                plugin_getFilterList: std::mem::transmute(get_func(c_str!(stringify!(
                    plugin_getFilterList
                )))),
                plugin_getImportableProjectFilterList: std::mem::transmute(get_func(c_str!(
                    stringify!(plugin_getImportableProjectFilterList)
                ))),
                plugin_register: std::mem::transmute(get_func(c_str!(stringify!(plugin_register)))),
                PluginWantsAlwaysRunFx: std::mem::transmute(get_func(c_str!(stringify!(
                    PluginWantsAlwaysRunFx
                )))),
                PreventUIRefresh: std::mem::transmute(get_func(c_str!(stringify!(
                    PreventUIRefresh
                )))),
                projectconfig_var_addr: std::mem::transmute(get_func(c_str!(stringify!(
                    projectconfig_var_addr
                )))),
                projectconfig_var_getoffs: std::mem::transmute(get_func(c_str!(stringify!(
                    projectconfig_var_getoffs
                )))),
                realloc_cmd_ptr: std::mem::transmute(get_func(c_str!(stringify!(realloc_cmd_ptr)))),
                ReaperGetPitchShiftAPI: std::mem::transmute(get_func(c_str!(stringify!(
                    ReaperGetPitchShiftAPI
                )))),
                ReaScriptError: std::mem::transmute(get_func(c_str!(stringify!(ReaScriptError)))),
                RecursiveCreateDirectory: std::mem::transmute(get_func(c_str!(stringify!(
                    RecursiveCreateDirectory
                )))),
                reduce_open_files: std::mem::transmute(get_func(c_str!(stringify!(
                    reduce_open_files
                )))),
                RefreshToolbar: std::mem::transmute(get_func(c_str!(stringify!(RefreshToolbar)))),
                RefreshToolbar2: std::mem::transmute(get_func(c_str!(stringify!(RefreshToolbar2)))),
                relative_fn: std::mem::transmute(get_func(c_str!(stringify!(relative_fn)))),
                RemoveTrackSend: std::mem::transmute(get_func(c_str!(stringify!(RemoveTrackSend)))),
                RenderFileSection: std::mem::transmute(get_func(c_str!(stringify!(
                    RenderFileSection
                )))),
                ReorderSelectedTracks: std::mem::transmute(get_func(c_str!(stringify!(
                    ReorderSelectedTracks
                )))),
                Resample_EnumModes: std::mem::transmute(get_func(c_str!(stringify!(
                    Resample_EnumModes
                )))),
                Resampler_Create: std::mem::transmute(get_func(c_str!(stringify!(
                    Resampler_Create
                )))),
                resolve_fn: std::mem::transmute(get_func(c_str!(stringify!(resolve_fn)))),
                resolve_fn2: std::mem::transmute(get_func(c_str!(stringify!(resolve_fn2)))),
                ReverseNamedCommandLookup: std::mem::transmute(get_func(c_str!(stringify!(
                    ReverseNamedCommandLookup
                )))),
                ScaleFromEnvelopeMode: std::mem::transmute(get_func(c_str!(stringify!(
                    ScaleFromEnvelopeMode
                )))),
                ScaleToEnvelopeMode: std::mem::transmute(get_func(c_str!(stringify!(
                    ScaleToEnvelopeMode
                )))),
                screenset_register: std::mem::transmute(get_func(c_str!(stringify!(
                    screenset_register
                )))),
                screenset_registerNew: std::mem::transmute(get_func(c_str!(stringify!(
                    screenset_registerNew
                )))),
                screenset_unregister: std::mem::transmute(get_func(c_str!(stringify!(
                    screenset_unregister
                )))),
                screenset_unregisterByParam: std::mem::transmute(get_func(c_str!(stringify!(
                    screenset_unregisterByParam
                )))),
                screenset_updateLastFocus: std::mem::transmute(get_func(c_str!(stringify!(
                    screenset_updateLastFocus
                )))),
                SectionFromUniqueID: std::mem::transmute(get_func(c_str!(stringify!(
                    SectionFromUniqueID
                )))),
                SelectAllMediaItems: std::mem::transmute(get_func(c_str!(stringify!(
                    SelectAllMediaItems
                )))),
                SelectProjectInstance: std::mem::transmute(get_func(c_str!(stringify!(
                    SelectProjectInstance
                )))),
                SendLocalOscMessage: std::mem::transmute(get_func(c_str!(stringify!(
                    SendLocalOscMessage
                )))),
                SetActiveTake: std::mem::transmute(get_func(c_str!(stringify!(SetActiveTake)))),
                SetAutomationMode: std::mem::transmute(get_func(c_str!(stringify!(
                    SetAutomationMode
                )))),
                SetCurrentBPM: std::mem::transmute(get_func(c_str!(stringify!(SetCurrentBPM)))),
                SetCursorContext: std::mem::transmute(get_func(c_str!(stringify!(
                    SetCursorContext
                )))),
                SetEditCurPos: std::mem::transmute(get_func(c_str!(stringify!(SetEditCurPos)))),
                SetEditCurPos2: std::mem::transmute(get_func(c_str!(stringify!(SetEditCurPos2)))),
                SetEnvelopePoint: std::mem::transmute(get_func(c_str!(stringify!(
                    SetEnvelopePoint
                )))),
                SetEnvelopePointEx: std::mem::transmute(get_func(c_str!(stringify!(
                    SetEnvelopePointEx
                )))),
                SetEnvelopeStateChunk: std::mem::transmute(get_func(c_str!(stringify!(
                    SetEnvelopeStateChunk
                )))),
                SetExtState: std::mem::transmute(get_func(c_str!(stringify!(SetExtState)))),
                SetGlobalAutomationOverride: std::mem::transmute(get_func(c_str!(stringify!(
                    SetGlobalAutomationOverride
                )))),
                SetItemStateChunk: std::mem::transmute(get_func(c_str!(stringify!(
                    SetItemStateChunk
                )))),
                SetMasterTrackVisibility: std::mem::transmute(get_func(c_str!(stringify!(
                    SetMasterTrackVisibility
                )))),
                SetMediaItemInfo_Value: std::mem::transmute(get_func(c_str!(stringify!(
                    SetMediaItemInfo_Value
                )))),
                SetMediaItemLength: std::mem::transmute(get_func(c_str!(stringify!(
                    SetMediaItemLength
                )))),
                SetMediaItemPosition: std::mem::transmute(get_func(c_str!(stringify!(
                    SetMediaItemPosition
                )))),
                SetMediaItemSelected: std::mem::transmute(get_func(c_str!(stringify!(
                    SetMediaItemSelected
                )))),
                SetMediaItemTake_Source: std::mem::transmute(get_func(c_str!(stringify!(
                    SetMediaItemTake_Source
                )))),
                SetMediaItemTakeInfo_Value: std::mem::transmute(get_func(c_str!(stringify!(
                    SetMediaItemTakeInfo_Value
                )))),
                SetMediaTrackInfo_Value: std::mem::transmute(get_func(c_str!(stringify!(
                    SetMediaTrackInfo_Value
                )))),
                SetMIDIEditorGrid: std::mem::transmute(get_func(c_str!(stringify!(
                    SetMIDIEditorGrid
                )))),
                SetMixerScroll: std::mem::transmute(get_func(c_str!(stringify!(SetMixerScroll)))),
                SetMouseModifier: std::mem::transmute(get_func(c_str!(stringify!(
                    SetMouseModifier
                )))),
                SetOnlyTrackSelected: std::mem::transmute(get_func(c_str!(stringify!(
                    SetOnlyTrackSelected
                )))),
                SetProjectGrid: std::mem::transmute(get_func(c_str!(stringify!(SetProjectGrid)))),
                SetProjectMarker: std::mem::transmute(get_func(c_str!(stringify!(
                    SetProjectMarker
                )))),
                SetProjectMarker2: std::mem::transmute(get_func(c_str!(stringify!(
                    SetProjectMarker2
                )))),
                SetProjectMarker3: std::mem::transmute(get_func(c_str!(stringify!(
                    SetProjectMarker3
                )))),
                SetProjectMarker4: std::mem::transmute(get_func(c_str!(stringify!(
                    SetProjectMarker4
                )))),
                SetProjectMarkerByIndex: std::mem::transmute(get_func(c_str!(stringify!(
                    SetProjectMarkerByIndex
                )))),
                SetProjectMarkerByIndex2: std::mem::transmute(get_func(c_str!(stringify!(
                    SetProjectMarkerByIndex2
                )))),
                SetProjExtState: std::mem::transmute(get_func(c_str!(stringify!(SetProjExtState)))),
                SetRegionRenderMatrix: std::mem::transmute(get_func(c_str!(stringify!(
                    SetRegionRenderMatrix
                )))),
                SetRenderLastError: std::mem::transmute(get_func(c_str!(stringify!(
                    SetRenderLastError
                )))),
                SetTakeStretchMarker: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTakeStretchMarker
                )))),
                SetTakeStretchMarkerSlope: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTakeStretchMarkerSlope
                )))),
                SetTempoTimeSigMarker: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTempoTimeSigMarker
                )))),
                SetToggleCommandState: std::mem::transmute(get_func(c_str!(stringify!(
                    SetToggleCommandState
                )))),
                SetTrackAutomationMode: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTrackAutomationMode
                )))),
                SetTrackColor: std::mem::transmute(get_func(c_str!(stringify!(SetTrackColor)))),
                SetTrackMIDILyrics: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTrackMIDILyrics
                )))),
                SetTrackMIDINoteName: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTrackMIDINoteName
                )))),
                SetTrackMIDINoteNameEx: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTrackMIDINoteNameEx
                )))),
                SetTrackSelected: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTrackSelected
                )))),
                SetTrackSendInfo_Value: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTrackSendInfo_Value
                )))),
                SetTrackSendUIPan: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTrackSendUIPan
                )))),
                SetTrackSendUIVol: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTrackSendUIVol
                )))),
                SetTrackStateChunk: std::mem::transmute(get_func(c_str!(stringify!(
                    SetTrackStateChunk
                )))),
                ShowActionList: std::mem::transmute(get_func(c_str!(stringify!(ShowActionList)))),
                ShowConsoleMsg: std::mem::transmute(get_func(c_str!(stringify!(ShowConsoleMsg)))),
                ShowMessageBox: std::mem::transmute(get_func(c_str!(stringify!(ShowMessageBox)))),
                ShowPopupMenu: std::mem::transmute(get_func(c_str!(stringify!(ShowPopupMenu)))),
                SLIDER2DB: std::mem::transmute(get_func(c_str!(stringify!(SLIDER2DB)))),
                SnapToGrid: std::mem::transmute(get_func(c_str!(stringify!(SnapToGrid)))),
                SoloAllTracks: std::mem::transmute(get_func(c_str!(stringify!(SoloAllTracks)))),
                Splash_GetWnd: std::mem::transmute(get_func(c_str!(stringify!(Splash_GetWnd)))),
                SplitMediaItem: std::mem::transmute(get_func(c_str!(stringify!(SplitMediaItem)))),
                StopPreview: std::mem::transmute(get_func(c_str!(stringify!(StopPreview)))),
                StopTrackPreview: std::mem::transmute(get_func(c_str!(stringify!(
                    StopTrackPreview
                )))),
                StopTrackPreview2: std::mem::transmute(get_func(c_str!(stringify!(
                    StopTrackPreview2
                )))),
                stringToGuid: std::mem::transmute(get_func(c_str!(stringify!(stringToGuid)))),
                StuffMIDIMessage: std::mem::transmute(get_func(c_str!(stringify!(
                    StuffMIDIMessage
                )))),
                TakeFX_AddByName: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_AddByName
                )))),
                TakeFX_CopyToTake: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_CopyToTake
                )))),
                TakeFX_CopyToTrack: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_CopyToTrack
                )))),
                TakeFX_Delete: std::mem::transmute(get_func(c_str!(stringify!(TakeFX_Delete)))),
                TakeFX_EndParamEdit: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_EndParamEdit
                )))),
                TakeFX_FormatParamValue: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_FormatParamValue
                )))),
                TakeFX_FormatParamValueNormalized: std::mem::transmute(get_func(c_str!(
                    stringify!(TakeFX_FormatParamValueNormalized)
                ))),
                TakeFX_GetChainVisible: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetChainVisible
                )))),
                TakeFX_GetCount: std::mem::transmute(get_func(c_str!(stringify!(TakeFX_GetCount)))),
                TakeFX_GetEnabled: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetEnabled
                )))),
                TakeFX_GetEnvelope: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetEnvelope
                )))),
                TakeFX_GetFloatingWindow: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetFloatingWindow
                )))),
                TakeFX_GetFormattedParamValue: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetFormattedParamValue
                )))),
                TakeFX_GetFXGUID: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetFXGUID
                )))),
                TakeFX_GetFXName: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetFXName
                )))),
                TakeFX_GetIOSize: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetIOSize
                )))),
                TakeFX_GetNamedConfigParm: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetNamedConfigParm
                )))),
                TakeFX_GetNumParams: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetNumParams
                )))),
                TakeFX_GetOffline: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetOffline
                )))),
                TakeFX_GetOpen: std::mem::transmute(get_func(c_str!(stringify!(TakeFX_GetOpen)))),
                TakeFX_GetParam: std::mem::transmute(get_func(c_str!(stringify!(TakeFX_GetParam)))),
                TakeFX_GetParameterStepSizes: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetParameterStepSizes
                )))),
                TakeFX_GetParamEx: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetParamEx
                )))),
                TakeFX_GetParamName: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetParamName
                )))),
                TakeFX_GetParamNormalized: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetParamNormalized
                )))),
                TakeFX_GetPinMappings: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetPinMappings
                )))),
                TakeFX_GetPreset: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetPreset
                )))),
                TakeFX_GetPresetIndex: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetPresetIndex
                )))),
                TakeFX_GetUserPresetFilename: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_GetUserPresetFilename
                )))),
                TakeFX_NavigatePresets: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_NavigatePresets
                )))),
                TakeFX_SetEnabled: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_SetEnabled
                )))),
                TakeFX_SetNamedConfigParm: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_SetNamedConfigParm
                )))),
                TakeFX_SetOffline: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_SetOffline
                )))),
                TakeFX_SetOpen: std::mem::transmute(get_func(c_str!(stringify!(TakeFX_SetOpen)))),
                TakeFX_SetParam: std::mem::transmute(get_func(c_str!(stringify!(TakeFX_SetParam)))),
                TakeFX_SetParamNormalized: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_SetParamNormalized
                )))),
                TakeFX_SetPinMappings: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_SetPinMappings
                )))),
                TakeFX_SetPreset: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_SetPreset
                )))),
                TakeFX_SetPresetByIndex: std::mem::transmute(get_func(c_str!(stringify!(
                    TakeFX_SetPresetByIndex
                )))),
                TakeFX_Show: std::mem::transmute(get_func(c_str!(stringify!(TakeFX_Show)))),
                TakeIsMIDI: std::mem::transmute(get_func(c_str!(stringify!(TakeIsMIDI)))),
                ThemeLayout_GetLayout: std::mem::transmute(get_func(c_str!(stringify!(
                    ThemeLayout_GetLayout
                )))),
                ThemeLayout_GetParameter: std::mem::transmute(get_func(c_str!(stringify!(
                    ThemeLayout_GetParameter
                )))),
                ThemeLayout_RefreshAll: std::mem::transmute(get_func(c_str!(stringify!(
                    ThemeLayout_RefreshAll
                )))),
                ThemeLayout_SetLayout: std::mem::transmute(get_func(c_str!(stringify!(
                    ThemeLayout_SetLayout
                )))),
                ThemeLayout_SetParameter: std::mem::transmute(get_func(c_str!(stringify!(
                    ThemeLayout_SetParameter
                )))),
                time_precise: std::mem::transmute(get_func(c_str!(stringify!(time_precise)))),
                TimeMap2_beatsToTime: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap2_beatsToTime
                )))),
                TimeMap2_GetDividedBpmAtTime: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap2_GetDividedBpmAtTime
                )))),
                TimeMap2_GetNextChangeTime: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap2_GetNextChangeTime
                )))),
                TimeMap2_QNToTime: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap2_QNToTime
                )))),
                TimeMap2_timeToBeats: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap2_timeToBeats
                )))),
                TimeMap2_timeToQN: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap2_timeToQN
                )))),
                TimeMap_curFrameRate: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap_curFrameRate
                )))),
                TimeMap_GetDividedBpmAtTime: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap_GetDividedBpmAtTime
                )))),
                TimeMap_GetMeasureInfo: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap_GetMeasureInfo
                )))),
                TimeMap_GetMetronomePattern: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap_GetMetronomePattern
                )))),
                TimeMap_GetTimeSigAtTime: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap_GetTimeSigAtTime
                )))),
                TimeMap_QNToMeasures: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap_QNToMeasures
                )))),
                TimeMap_QNToTime: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap_QNToTime
                )))),
                TimeMap_QNToTime_abs: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap_QNToTime_abs
                )))),
                TimeMap_timeToQN: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap_timeToQN
                )))),
                TimeMap_timeToQN_abs: std::mem::transmute(get_func(c_str!(stringify!(
                    TimeMap_timeToQN_abs
                )))),
                ToggleTrackSendUIMute: std::mem::transmute(get_func(c_str!(stringify!(
                    ToggleTrackSendUIMute
                )))),
                Track_GetPeakHoldDB: std::mem::transmute(get_func(c_str!(stringify!(
                    Track_GetPeakHoldDB
                )))),
                Track_GetPeakInfo: std::mem::transmute(get_func(c_str!(stringify!(
                    Track_GetPeakInfo
                )))),
                TrackCtl_SetToolTip: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackCtl_SetToolTip
                )))),
                TrackFX_AddByName: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_AddByName
                )))),
                TrackFX_CopyToTake: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_CopyToTake
                )))),
                TrackFX_CopyToTrack: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_CopyToTrack
                )))),
                TrackFX_Delete: std::mem::transmute(get_func(c_str!(stringify!(TrackFX_Delete)))),
                TrackFX_EndParamEdit: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_EndParamEdit
                )))),
                TrackFX_FormatParamValue: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_FormatParamValue
                )))),
                TrackFX_FormatParamValueNormalized: std::mem::transmute(get_func(c_str!(
                    stringify!(TrackFX_FormatParamValueNormalized)
                ))),
                TrackFX_GetByName: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetByName
                )))),
                TrackFX_GetChainVisible: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetChainVisible
                )))),
                TrackFX_GetCount: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetCount
                )))),
                TrackFX_GetEnabled: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetEnabled
                )))),
                TrackFX_GetEQ: std::mem::transmute(get_func(c_str!(stringify!(TrackFX_GetEQ)))),
                TrackFX_GetEQBandEnabled: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetEQBandEnabled
                )))),
                TrackFX_GetEQParam: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetEQParam
                )))),
                TrackFX_GetFloatingWindow: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetFloatingWindow
                )))),
                TrackFX_GetFormattedParamValue: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetFormattedParamValue
                )))),
                TrackFX_GetFXGUID: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetFXGUID
                )))),
                TrackFX_GetFXName: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetFXName
                )))),
                TrackFX_GetInstrument: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetInstrument
                )))),
                TrackFX_GetIOSize: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetIOSize
                )))),
                TrackFX_GetNamedConfigParm: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetNamedConfigParm
                )))),
                TrackFX_GetNumParams: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetNumParams
                )))),
                TrackFX_GetOffline: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetOffline
                )))),
                TrackFX_GetOpen: std::mem::transmute(get_func(c_str!(stringify!(TrackFX_GetOpen)))),
                TrackFX_GetParam: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetParam
                )))),
                TrackFX_GetParameterStepSizes: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetParameterStepSizes
                )))),
                TrackFX_GetParamEx: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetParamEx
                )))),
                TrackFX_GetParamName: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetParamName
                )))),
                TrackFX_GetParamNormalized: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetParamNormalized
                )))),
                TrackFX_GetPinMappings: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetPinMappings
                )))),
                TrackFX_GetPreset: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetPreset
                )))),
                TrackFX_GetPresetIndex: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetPresetIndex
                )))),
                TrackFX_GetRecChainVisible: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetRecChainVisible
                )))),
                TrackFX_GetRecCount: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetRecCount
                )))),
                TrackFX_GetUserPresetFilename: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_GetUserPresetFilename
                )))),
                TrackFX_NavigatePresets: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_NavigatePresets
                )))),
                TrackFX_SetEnabled: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_SetEnabled
                )))),
                TrackFX_SetEQBandEnabled: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_SetEQBandEnabled
                )))),
                TrackFX_SetEQParam: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_SetEQParam
                )))),
                TrackFX_SetNamedConfigParm: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_SetNamedConfigParm
                )))),
                TrackFX_SetOffline: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_SetOffline
                )))),
                TrackFX_SetOpen: std::mem::transmute(get_func(c_str!(stringify!(TrackFX_SetOpen)))),
                TrackFX_SetParam: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_SetParam
                )))),
                TrackFX_SetParamNormalized: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_SetParamNormalized
                )))),
                TrackFX_SetPinMappings: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_SetPinMappings
                )))),
                TrackFX_SetPreset: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_SetPreset
                )))),
                TrackFX_SetPresetByIndex: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackFX_SetPresetByIndex
                )))),
                TrackFX_Show: std::mem::transmute(get_func(c_str!(stringify!(TrackFX_Show)))),
                TrackList_AdjustWindows: std::mem::transmute(get_func(c_str!(stringify!(
                    TrackList_AdjustWindows
                )))),
                TrackList_UpdateAllExternalSurfaces: std::mem::transmute(get_func(c_str!(
                    stringify!(TrackList_UpdateAllExternalSurfaces)
                ))),
                Undo_BeginBlock: std::mem::transmute(get_func(c_str!(stringify!(Undo_BeginBlock)))),
                Undo_BeginBlock2: std::mem::transmute(get_func(c_str!(stringify!(
                    Undo_BeginBlock2
                )))),
                Undo_CanRedo2: std::mem::transmute(get_func(c_str!(stringify!(Undo_CanRedo2)))),
                Undo_CanUndo2: std::mem::transmute(get_func(c_str!(stringify!(Undo_CanUndo2)))),
                Undo_DoRedo2: std::mem::transmute(get_func(c_str!(stringify!(Undo_DoRedo2)))),
                Undo_DoUndo2: std::mem::transmute(get_func(c_str!(stringify!(Undo_DoUndo2)))),
                Undo_EndBlock: std::mem::transmute(get_func(c_str!(stringify!(Undo_EndBlock)))),
                Undo_EndBlock2: std::mem::transmute(get_func(c_str!(stringify!(Undo_EndBlock2)))),
                Undo_OnStateChange: std::mem::transmute(get_func(c_str!(stringify!(
                    Undo_OnStateChange
                )))),
                Undo_OnStateChange2: std::mem::transmute(get_func(c_str!(stringify!(
                    Undo_OnStateChange2
                )))),
                Undo_OnStateChange_Item: std::mem::transmute(get_func(c_str!(stringify!(
                    Undo_OnStateChange_Item
                )))),
                Undo_OnStateChangeEx: std::mem::transmute(get_func(c_str!(stringify!(
                    Undo_OnStateChangeEx
                )))),
                Undo_OnStateChangeEx2: std::mem::transmute(get_func(c_str!(stringify!(
                    Undo_OnStateChangeEx2
                )))),
                update_disk_counters: std::mem::transmute(get_func(c_str!(stringify!(
                    update_disk_counters
                )))),
                UpdateArrange: std::mem::transmute(get_func(c_str!(stringify!(UpdateArrange)))),
                UpdateItemInProject: std::mem::transmute(get_func(c_str!(stringify!(
                    UpdateItemInProject
                )))),
                UpdateTimeline: std::mem::transmute(get_func(c_str!(stringify!(UpdateTimeline)))),
                ValidatePtr: std::mem::transmute(get_func(c_str!(stringify!(ValidatePtr)))),
                ValidatePtr2: std::mem::transmute(get_func(c_str!(stringify!(ValidatePtr2)))),
                ViewPrefs: std::mem::transmute(get_func(c_str!(stringify!(ViewPrefs)))),
                WDL_VirtualWnd_ScaledBlitBG: std::mem::transmute(get_func(c_str!(stringify!(
                    WDL_VirtualWnd_ScaledBlitBG
                )))),
                GetMidiInput: std::mem::transmute(get_func(c_str!(stringify!(GetMidiInput)))),
                GetMidiOutput: std::mem::transmute(get_func(c_str!(stringify!(GetMidiOutput)))),
            }
        }
    }
}
