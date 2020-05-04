use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr::{null_mut, NonNull};

use reaper_rs_low::{
    add_cpp_control_surface, firewall, raw, remove_cpp_control_surface, IReaperControlSurface,
    Reaper,
};

use crate::ProjectContext::CurrentProject;
use crate::{
    concat_c_strs, delegating_hook_command, delegating_hook_post_command, delegating_toggle_action,
    require_non_null, require_non_null_panic, ActionValueChange, AddFxBehavior, AddFxFailed,
    AudioHookRegister, AutomationMode, Bpm, ChunkCacheHint, CommandId, CreateTrackSendFailed, Db,
    DelegatingControlSurface, EnvChunkName, FxAddByNameBehavior, FxNotFound, FxOrParameterNotFound,
    FxOrParameterNotFoundOrCockosExtNotSupported, FxPresetRef, FxShowInstruction, GangBehavior,
    GlobalAutomationModeOverride, GuidExpressionInvalid, Hwnd, InputMonitoringMode,
    InvalidTrackInfoKey, KbdSectionInfo, MasterTrackBehavior, MediaTrack, MediumAudioHookRegister,
    MediumGaccelRegister, MediumHookCommand, MediumHookPostCommand, MediumReaperControlSurface,
    MediumToggleAction, MessageBoxResult, MessageBoxType, MidiInput, MidiInputDeviceId,
    MidiOutputDeviceId, NotificationBehavior, PlaybackSpeedFactor, PluginRegistration,
    ProjectContext, ProjectPart, ProjectRef, ReaProject, ReaperFunctionFailed,
    ReaperNormalizedFxParamValue, ReaperPanValue, ReaperPointer, ReaperStringArg, ReaperVersion,
    ReaperVolumeValue, RecordArmState, RecordingInput, SectionContext, SectionId, SendTarget,
    StuffMidiMessageTarget, TrackDefaultsBehavior, TrackEnvelope, TrackFxChainType,
    TrackFxLocation, TrackInfoKey, TrackRef, TrackSendCategory, TrackSendDirection,
    TrackSendInfoKey, TransferBehavior, UndoBehavior, UndoScope, ValueChange, VolumeSliderValue,
    WindowContext,
};

use helgoboss_midi::ShortMessage;
use reaper_rs_low;
use reaper_rs_low::raw::{
    audio_hook_register_t, gaccel_register_t, midi_Input, GUID, UNDO_STATE_ALL,
};

use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::path::PathBuf;

/// Parent marker trait representing a thread type.
///
/// See *Design* section of [`ReaperFunctions`] for more information.
///
/// [`ReaperFunctions`]: struct.ReaperFunctions.html
pub trait ThreadScope: Debug {}

/// Marker thread representing the main thread.
pub trait MainThread: ThreadScope {}

/// Marker thread representing the audio thread.
pub trait AudioThread: ThreadScope {}

/// This is the main access point for most REAPER functions.
///
/// # Basics
///
/// You can obtain an instance of this struct by calling [`Reaper::functions()`]. This unlocks all
/// functions which are safe to execute in the main thread. If you want access to the functions
/// which are safe to execute in the audio thread, call [`Reaper::create_real_time_functions()`]
/// instead. REAPER functions which are related to registering/unregistering things are located in
/// [`Reaper`].
///
/// Please note that this struct contains nothing but function pointers, so you are free to clone
/// it, e.g. in order to make all functions accessible somewhere else. This is sometimes easier than
/// passing references around. Don't do it too often though. It's just a bitwise copy of all
/// function pointers, but there are around 800 of them, so each copy will occupy about 7 kB of
/// memory on a 64-bit system.
///
/// # Panics
///
/// Don't assume that all REAPER functions exposed here are always available. It's possible that the
/// user runs your plug-in in an older version of REAPER where a function is missing. See the
/// documentation of [low-level `Reaper`] for ways how to deal with this.
///
/// # Work in progress
///
/// Many functions which are available in the low-level API have not been lifted to the medium-level
/// API yet. Unlike the low-level API, the medium-level one is hand-written and probably a perpetual
/// work in progress. If you can't find the function that you need, you can always resort to the
/// low-level API by navigating to [`low()`]. Of course you are welcome to contribute to bring the
/// medium-level API on par with the low-level one.
///
/// # Design
///
/// ## What's this `<dyn MainThread>` thing about?
///
/// In REAPER and probably many other DAWs there are at least two important threads:
///
/// 1. The main thread (responsible for things like UI, driven by the UI main loop).
/// 2. The audio thread (responsible for processing audio and MIDI buffers, driven by the audio
/// hardware)
///
/// Most functions offered by REAPER are only safe to be executed in the main thread. If you execute
/// them in the audio thread, REAPER will crash. Or worse: It will seemingly work on your machine
/// and crash on someone else's. There are also a few functions which are only safe to be executed
/// in the audio thread. And there are also very few functions which are safe to be executed from
/// *any* thread (thread-safe).
///
/// There's currently no way to make sure at compile time that a function is called in the correct
/// thread. Of course that would be the best. In an attempt to still let the compiler help you a
/// bit, the traits [`MainThread`] and [`AudioThread`] have been introduced. They are marker threads
/// which are used as type bound on each method which is not thread-safe. So depending on the
/// context we can expose an instance of [`ReaperFunctions`] which has only functions unlocked
/// which are safe to be executed from e.g. the audio thread. The compiler will complain if you
/// attempt to call an audio-thread-only method on `ReaperFunctions<dyn MainThread>` and vice versa.
///
/// Of course that technique can't prevent anyone from acquiring a main-thread only instance and
/// use it in the audio hook. But still, it adds some extra safety.
///
/// The alternative to tagging functions via marker traits would have been to implement e.g.
/// audio-thread-only functions in a trait `CallableFromAudioThread` as default functions and create
/// a struct that inherits those default functions. Disadvantage: Consumer always would have to
/// bring the trait into scope to see the functions. That's confusing. It also would provide less
/// amount of safety.
///
/// ## Why no fail-fast at runtime when getting threading wrong?
///
/// Another thing which could help would be to panic when a main-thread-only function is called in
/// the audio thread or vice versa. This would prevent "it works on my machine" scenarios. However,
/// this is currently not being done because of possible performance implications.
///
/// [`Reaper`]: struct.Reaper.html
/// [`Reaper::functions()`]: struct.Reaper.html#method.functions
/// [`Reaper::create_real_time_functions()`]: struct.Reaper.html#method.create_real_time_functions
/// [`low()`]: #method.low
/// [low-level `Reaper`]: /reaper_rs_low/struct.Reaper.html
/// [`MainThread`]: trait.MainThread.html
/// [`AudioThread`]: trait.AudioThread.html
/// [`ReaperFunctions`]: struct.ReaperFunctions.html
#[derive(Clone, Debug)]
pub struct ReaperFunctions<S: ?Sized + ThreadScope = dyn MainThread> {
    low: reaper_rs_low::Reaper,
    p: PhantomData<S>,
}

impl<S: ?Sized + ThreadScope> ReaperFunctions<S> {
    pub(crate) fn new(low: reaper_rs_low::Reaper) -> ReaperFunctions<S> {
        ReaperFunctions {
            low,
            p: PhantomData,
        }
    }

    /// Gives access to the low-level Reaper instance.
    pub fn low(&self) -> &reaper_rs_low::Reaper {
        &self.low
    }

    /// Returns the requested project and optionally its file name.
    ///
    /// With `projfn_out_optional_sz` you can tell REAPER how many characters of the file name you
    /// want. If you are not interested in the file name at all, pass 0.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let reaper = reaper_rs_medium::Reaper::default();
    /// use reaper_rs_medium::ProjectRef::Tab;
    /// use reaper_rs_medium::ReaperMainThreadFunctions;
    ///
    /// let result = reaper.enum_projects(Tab(4), 256).ok_or("No such tab")?;
    /// let project_dir = result.file_path.ok_or("Project not saved yet")?.parent();
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    // TODO-low Like many functions, this is not marked as unsafe - yet it is still unsafe in one
    //  way: It must be called in the main thread, otherwise there will be undefined behavior. For
    //  now, the strategy is to just document it and have the type system help a bit
    //  (`ReaperFunctions<MainThread>`). However, there *is* a way to make it safe in the sense of
    //  failing fast without running into undefined behavior: Assert at each function call that we
    //  are in the main thread. The main thread ID could be easily obtained at construction time
    //  of medium-level Reaper. So all it needs is acquiring the current thread and compare its ID
    //  with the main thread ID (both presumably cheap). I think that would be fine. Maybe we should
    //  provide a feature to turn it on/off or make it a debug_assert only or provide an additional
    //  unchecked version. In audio-thread functions it might be too much overhead though calling
    //  is_in_real_time_audio() each time, so maybe we should mark them as unsafe.
    pub fn enum_projects(
        &self,
        proj_ref: ProjectRef,
        projfn_out_optional_sz: u32,
    ) -> Option<EnumProjectsResult>
    where
        S: MainThread,
    {
        let idx = proj_ref.to_raw();
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

    /// Returns the track at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let reaper = reaper_rs_medium::Reaper::default();
    /// use reaper_rs_medium::ProjectContext::CurrentProject;
    ///
    /// let track = reaper.get_track(CurrentProject, 3).ok_or("No such track")?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_track(&self, proj: ProjectContext, trackidx: u32) -> Option<MediaTrack>
    where
        S: MainThread,
    {
        self.require_valid_project(proj);
        unsafe { self.get_track_unchecked(proj, trackidx) }
    }

    /// Like [`get_track()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_track()`]: #method.get_track
    pub unsafe fn get_track_unchecked(
        &self,
        proj: ProjectContext,
        trackidx: u32,
    ) -> Option<MediaTrack>
    where
        S: MainThread,
    {
        let ptr = self.low.GetTrack(proj.to_raw(), trackidx as i32);
        NonNull::new(ptr)
    }

    /// Checks if the given pointer is still valid.
    ///
    /// Returns `true` if the pointer is a valid object of the correct type in the given project.
    /// The project is ignored if the pointer itself is a project.
    pub fn validate_ptr_2<'a>(
        &self,
        proj: ProjectContext,
        pointer: impl Into<ReaperPointer<'a>>,
    ) -> bool {
        let pointer = pointer.into();
        unsafe {
            self.low.ValidatePtr2(
                proj.to_raw(),
                pointer.ptr_as_void(),
                pointer.key_into_raw().as_ptr(),
            )
        }
    }

    /// Checks if the given pointer is still valid.
    ///
    /// Returns `true` if the pointer is a valid object of the correct type in the current project.
    pub fn validate_ptr<'a>(&self, pointer: impl Into<ReaperPointer<'a>>) -> bool
    where
        S: MainThread,
    {
        let pointer = pointer.into();
        unsafe {
            self.low
                .ValidatePtr(pointer.ptr_as_void(), pointer.key_into_raw().as_ptr())
        }
    }

    /// Redraws the arrange view and ruler.
    pub fn update_timeline(&self)
    where
        S: MainThread,
    {
        self.low.UpdateTimeline();
    }

    /// Shows a message to the user in the ReaScript console.
    ///
    /// This is also useful for debugging. Send "\n" for newline and "" to clear the console.
    pub fn show_console_msg<'a>(&self, msg: impl Into<ReaperStringArg<'a>>) {
        unsafe { self.low.ShowConsoleMsg(msg.into().as_ptr()) }
    }

    /// Gets or sets a track attribute.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track or invalid new value.
    pub unsafe fn get_set_media_track_info(
        &self,
        tr: MediaTrack,
        parmname: TrackInfoKey,
        set_new_value: *mut c_void,
    ) -> *mut c_void
    where
        S: MainThread,
    {
        self.low
            .GetSetMediaTrackInfo(tr.as_ptr(), parmname.into_raw().as_ptr(), set_new_value)
    }

    /// Convenience function which returns the given track's parent track (`P_PARTRACK`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_par_track(
        &self,
        tr: MediaTrack,
    ) -> Option<MediaTrack>
    where
        S: MainThread,
    {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::ParTrack, null_mut())
            as *mut raw::MediaTrack;
        NonNull::new(ptr)
    }

    /// Convenience function which returns the given track's parent project (`P_PROJECT`).
    ///
    /// In REAPER < 5.95 this returns `None`.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_project(&self, tr: MediaTrack) -> Option<ReaProject>
    where
        S: MainThread,
    {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::Project, null_mut())
            as *mut raw::ReaProject;
        NonNull::new(ptr)
    }

    /// Convenience function which grants temporary access to the given track's name (`P_NAME`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_name<R>(
        &self,
        tr: MediaTrack,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R>
    where
        S: MainThread,
    {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::Name, null_mut());
        unsafe { create_passing_c_str(ptr as *const c_char) }.map(f)
    }

    /// Convenience function which returns the given track's input monitoring mode (I_RECMON).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_rec_mon(&self, tr: MediaTrack) -> InputMonitoringMode
    where
        S: MainThread,
    {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::RecMon, null_mut());
        let irecmon = unsafe { unref_as::<i32>(ptr) }.unwrap();
        InputMonitoringMode::try_from(irecmon).expect("Unknown input monitoring mode")
    }

    /// Convenience function which returns the given track's recording input (I_RECINPUT).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_rec_input(
        &self,
        tr: MediaTrack,
    ) -> Option<RecordingInput>
    where
        S: MainThread,
    {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::RecInput, null_mut());
        let rec_input_index = unsafe { unref_as::<i32>(ptr) }.unwrap();
        if rec_input_index < 0 {
            None
        } else {
            Some(RecordingInput::try_from_raw(rec_input_index).unwrap())
        }
    }

    /// Convenience function which returns the type and location of the given track
    /// (IP_TRACKNUMBER).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_track_number(
        &self,
        tr: MediaTrack,
    ) -> Option<TrackRef>
    where
        S: MainThread,
    {
        use TrackRef::*;
        match self.get_set_media_track_info(tr, TrackInfoKey::TrackNumber, null_mut()) as i32 {
            -1 => Some(MasterTrack),
            0 => None,
            n if n > 0 => Some(NormalTrack(n as u32 - 1)),
            _ => unreachable!(),
        }
    }

    /// Convenience function which returns the given track's GUID (GUID).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_guid(&self, tr: MediaTrack) -> GUID
    where
        S: MainThread,
    {
        let ptr = self.get_set_media_track_info(tr, TrackInfoKey::Guid, null_mut());
        unsafe { unref_as::<GUID>(ptr) }.unwrap()
    }

    /// Returns whether we are in the real-time audio thread.
    ///
    /// *Real-time* means somewhere between [`OnAudioBuffer`] calls, not in some worker or
    /// anticipative FX thread.
    ///
    /// TODO-medium There are different kinds of audio threads, one being the real-time audio
    ///  thread.
    ///
    /// [`OnAudioBuffer`]: trait.MediumOnAudioBuffer.html#method.call
    pub fn is_in_real_time_audio(&self) -> bool {
        self.low.IsInRealTimeAudio() != 0
    }

    /// Performs an action belonging to the main section.
    ///
    /// To perform non-native actions (ReaScripts, custom or extension plugin actions) safely, see
    /// [`named_command_lookup()`].
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    ///
    /// [`named_command_lookup()`]: #method.named_command_lookup
    pub fn main_on_command_ex(&self, command: CommandId, flag: i32, proj: ProjectContext) {
        self.require_valid_project(proj);
        unsafe { self.main_on_command_ex_unchecked(command, flag, proj) }
    }

    /// Like [`main_on_command_ex()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`main_on_command_ex()`]: #method.main_on_command_ex
    pub unsafe fn main_on_command_ex_unchecked(
        &self,
        command: CommandId,
        flag: i32,
        proj: ProjectContext,
    ) {
        self.low
            .Main_OnCommandEx(command.to_raw(), flag, proj.to_raw());
    }

    /// Mutes or unmutes the given track.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let reaper = reaper_rs_medium::Reaper::default();
    /// use reaper_rs_medium::{NotificationBehavior::NotifyAll, ProjectContext::CurrentProject};
    /// use reaper_rs_medium::get_cpp_control_surface;
    ///
    /// let track = reaper.get_track(CurrentProject, 0).ok_or("Track doesn't exist")?;
    /// unsafe {
    ///     reaper.csurf_set_surface_mute(track, true, NotifyAll);
    /// }
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_set_surface_mute(
        &self,
        trackid: MediaTrack,
        mute: bool,
        ignoresurf: NotificationBehavior,
    ) {
        self.low
            .CSurf_SetSurfaceMute(trackid.as_ptr(), mute, ignoresurf.to_raw());
    }

    /// Soloes or unsoloes the given track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_set_surface_solo(
        &self,
        trackid: MediaTrack,
        solo: bool,
        ignoresurf: NotificationBehavior,
    ) {
        self.low
            .CSurf_SetSurfaceSolo(trackid.as_ptr(), solo, ignoresurf.to_raw());
    }

    /// Generates a random GUID.
    pub fn gen_guid(&self) -> GUID
    where
        S: MainThread,
    {
        let mut guid = MaybeUninit::uninit();
        unsafe {
            self.low.genGuid(guid.as_mut_ptr());
        }
        unsafe { guid.assume_init() }
    }

    /// Grants temporary access to the section with the given ID.
    // In order to not need unsafe, we take the closure. For normal medium-level API usage, this is
    // the safe way to go.
    pub fn section_from_unique_id<R>(
        &self,
        unique_id: SectionId,
        f: impl FnOnce(&KbdSectionInfo) -> R,
    ) -> Option<R>
    where
        S: MainThread,
    {
        let ptr = self.low.SectionFromUniqueID(unique_id.to_raw());
        if ptr.is_null() {
            return None;
        }
        NonNull::new(ptr).map(|nnp| f(&KbdSectionInfo(nnp)))
    }

    /// Like [`section_from_unique_id()`] but returns the section.
    ///
    /// # Safety
    ///
    /// The lifetime of the returned section is unbounded.
    ///
    /// [`section_from_unique_id()`]: #method.section_from_unique_id
    // The closure-taking function might be too restrictive in some cases, e.g. it wouldn't let us
    // return an iterator (which is of course lazily evaluated). Also, in some cases we might know
    // that a section is always valid, e.g. if it's the main section. A higher-level API could
    // use this for such edge cases. If not the main section, a higher-level API
    // should check if the section still exists (via section index) before each usage.
    pub unsafe fn section_from_unique_id_unchecked(
        &self,
        unique_id: SectionId,
    ) -> Option<KbdSectionInfo>
    where
        S: MainThread,
    {
        let ptr = self.low.SectionFromUniqueID(unique_id.to_raw());
        NonNull::new(ptr).map(KbdSectionInfo)
    }

    /// Performs an action belonging to the main section.
    ///
    /// Unlike [`main_on_command_ex()`], this function also allows to control actions learned with
    /// MIDI/OSC.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project or window.
    ///
    /// [`main_on_command_ex()`]: #method.main_on_command_ex
    // Kept return value type i32 because I have no idea what the return value is about.
    pub unsafe fn kbd_on_main_action_ex(
        &self,
        cmd: CommandId,
        value: ActionValueChange,
        hwnd: WindowContext,
        proj: ProjectContext,
    ) -> i32
    where
        S: MainThread,
    {
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
        self.low.KBD_OnMainActionEx(
            cmd.to_raw(),
            val,
            valhw,
            relmode,
            hwnd.to_raw(),
            proj.to_raw(),
        )
    }

    /// Returns the REAPER main window handle.
    pub fn get_main_hwnd(&self) -> Hwnd
    where
        S: MainThread,
    {
        require_non_null_panic(self.low.GetMainHwnd())
    }

    /// Looks up the command ID for a named command.
    ///
    /// Named commands can be registered by extensions (e.g. `_SWS_ABOUT`), ReaScripts
    /// (e.g. `_113088d11ae641c193a2b7ede3041ad5`) or custom actions.
    pub fn named_command_lookup<'a>(
        &self,
        command_name: impl Into<ReaperStringArg<'a>>,
    ) -> Option<CommandId>
    where
        S: MainThread,
    {
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

    /// Returns the number of tracks in the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn count_tracks(&self, proj: ProjectContext) -> u32
    where
        S: MainThread,
    {
        self.require_valid_project(proj);
        unsafe { self.count_tracks_unchecked(proj) }
    }

    /// Like [`count_tracks()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`count_tracks()`]: #method.count_tracks
    pub unsafe fn count_tracks_unchecked(&self, proj: ProjectContext) -> u32
    where
        S: MainThread,
    {
        self.low.CountTracks(proj.to_raw()) as u32
    }

    /// Creates a new track at the given index.
    pub fn insert_track_at_index(&self, idx: u32, want_defaults: TrackDefaultsBehavior) {
        self.low.InsertTrackAtIndex(
            idx as i32,
            want_defaults == TrackDefaultsBehavior::AddDefaultEnvAndFx,
        );
    }

    /// Returns the maximum number of MIDI input devices (usually 63).
    pub fn get_max_midi_inputs(&self) -> u32 {
        self.low.GetMaxMidiInputs() as u32
    }

    /// Returns the maximum number of MIDI output devices (usually 64).
    pub fn get_max_midi_outputs(&self) -> u32 {
        self.low.GetMaxMidiOutputs() as u32
    }

    /// Returns information about the given MIDI input device.
    ///
    /// With `nameout_sz` you can tell REAPER how many characters of the device name you want. If
    /// you are not interested in the device name at all, pass 0.
    pub fn get_midi_input_name(
        &self,
        dev: MidiInputDeviceId,
        nameout_sz: u32,
    ) -> GetMidiDevNameResult
    where
        S: MainThread,
    {
        if nameout_sz == 0 {
            let is_present = unsafe { self.low.GetMIDIInputName(dev.to_raw(), null_mut(), 0) };
            GetMidiDevNameResult {
                is_present,
                name: None,
            }
        } else {
            let (name, is_present) = with_string_buffer(nameout_sz, |buffer, max_size| unsafe {
                self.low.GetMIDIInputName(dev.to_raw(), buffer, max_size)
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

    /// Returns information about the given MIDI output device.
    ///
    /// With `nameout_sz` you can tell REAPER how many characters of the device name you want. If
    /// you are not interested in the device name at all, pass 0.
    pub fn get_midi_output_name(
        &self,
        dev: MidiOutputDeviceId,
        nameout_sz: u32,
    ) -> GetMidiDevNameResult
    where
        S: MainThread,
    {
        if nameout_sz == 0 {
            let is_present = unsafe { self.low.GetMIDIOutputName(dev.to_raw(), null_mut(), 0) };
            GetMidiDevNameResult {
                is_present,
                name: None,
            }
        } else {
            let (name, is_present) = with_string_buffer(nameout_sz, |buffer, max_size| unsafe {
                self.low.GetMIDIOutputName(dev.to_raw(), buffer, max_size)
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
    // should be Option, if it's -1 or > 0, it should be Result. So we just keep the i32. That's
    // also one reason why we just publish the convenience functions.
    unsafe fn track_fx_add_by_name<'a>(
        &self,
        track: MediaTrack,
        fxname: impl Into<ReaperStringArg<'a>>,
        rec_fx: TrackFxChainType,
        instantiate: FxAddByNameBehavior,
    ) -> i32
    where
        S: MainThread,
    {
        self.low.TrackFX_AddByName(
            track.as_ptr(),
            fxname.into().as_ptr(),
            rec_fx == TrackFxChainType::InputFxChain,
            instantiate.into(),
        )
    }

    /// Returns the index of the first FX instance in a track or monitoring FX chain.
    ///
    /// The FX name can have a prefix to further specify its type: `VST3:` | `VST2:` | `VST:` |
    /// `AU:` | `JS:` | `DX:`
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_add_by_name_query<'a>(
        &self,
        track: MediaTrack,
        fxname: impl Into<ReaperStringArg<'a>>,
        rec_fx: TrackFxChainType,
    ) -> Option<u32>
    where
        S: MainThread,
    {
        match self.track_fx_add_by_name(track, fxname, rec_fx, FxAddByNameBehavior::Query) {
            -1 => None,
            idx if idx >= 0 => Some(idx as u32),
            _ => unreachable!(),
        }
    }

    /// Adds an instance of an FX to a track or monitoring FX chain.
    ///
    /// See [`track_fx_add_by_name_query()`] for possible FX name prefixes.
    ///
    /// # Errors
    ///
    /// Returns an error if the FX couldn't be added (e.g. if no such FX is installed).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    ///
    /// [`track_fx_add_by_name_query()`]: #method.track_fx_add_by_name_query
    pub unsafe fn track_fx_add_by_name_add<'a>(
        &self,
        track: MediaTrack,
        fxname: impl Into<ReaperStringArg<'a>>,
        rec_fx: TrackFxChainType,
        force_add: AddFxBehavior,
    ) -> Result<u32, AddFxFailed>
    where
        S: MainThread,
    {
        match self.track_fx_add_by_name(track, fxname, rec_fx, force_add.into()) {
            -1 => Err(AddFxFailed),
            idx if idx >= 0 => Ok(idx as u32),
            _ => unreachable!(),
        }
    }

    /// Returns whether the given track FX is enabled.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_enabled(&self, track: MediaTrack, fx: TrackFxLocation) -> bool
    where
        S: MainThread,
    {
        self.low.TrackFX_GetEnabled(track.as_ptr(), fx.to_raw())
    }

    /// Returns the name of the given FX.
    ///
    /// With `buf_sz` you can tell REAPER how many characters of the FX name you want.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
    ///
    /// # Errors
    ///
    /// Returns an error if the FX doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_fx_name(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        buf_sz: u32,
    ) -> Result<CString, FxNotFound>
    where
        S: MainThread,
    {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            self.low
                .TrackFX_GetFXName(track.as_ptr(), fx.to_raw(), buffer, max_size)
        });
        if !successful {
            return Err(FxNotFound);
        }
        Ok(name)
    }

    /// Returns the index of the first track FX that is a virtual instrument.
    ///
    /// Doesn't look in the input FX chain.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_instrument(&self, track: MediaTrack) -> Option<u32>
    where
        S: MainThread,
    {
        let index = self.low.TrackFX_GetInstrument(track.as_ptr());
        if index == -1 {
            return None;
        }
        Some(index as u32)
    }

    /// Enables or disables a track FX.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_set_enabled(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        enabled: bool,
    ) {
        self.low
            .TrackFX_SetEnabled(track.as_ptr(), fx.to_raw(), enabled);
    }

    /// Returns the number of parameters of given track FX.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_num_params(&self, track: MediaTrack, fx: TrackFxLocation) -> u32
    where
        S: MainThread,
    {
        self.low.TrackFX_GetNumParams(track.as_ptr(), fx.to_raw()) as u32
    }

    /// Returns the current project if it's just being loaded or saved.
    ///
    /// This is usually only used from `project_config_extension_t`.
    // TODO-low `project_config_extension_t` is not yet ported
    pub fn get_current_project_in_load_save(&self) -> Option<ReaProject>
    where
        S: MainThread,
    {
        let ptr = self.low.GetCurrentProjectInLoadSave();
        NonNull::new(ptr)
    }

    /// Returns the name of the given track FX parameter.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
    ///
    /// # Errors
    ///
    /// Returns an error if the FX or parameter doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_param_name(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        param: u32,
        buf_sz: u32,
    ) -> Result<CString, FxOrParameterNotFound>
    where
        S: MainThread,
    {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            self.low.TrackFX_GetParamName(
                track.as_ptr(),
                fx.to_raw(),
                param as i32,
                buffer,
                max_size,
            )
        });
        if !successful {
            return Err(FxOrParameterNotFound);
        }
        Ok(name)
    }

    /// Returns the current value of the given track FX parameter formatted as string.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
    ///
    /// # Errors
    ///
    /// Returns an error if the FX or parameter doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_formatted_param_value(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        param: u32,
        buf_sz: u32,
    ) -> Result<CString, FxOrParameterNotFound>
    where
        S: MainThread,
    {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            self.low.TrackFX_GetFormattedParamValue(
                track.as_ptr(),
                fx.to_raw(),
                param as i32,
                buffer,
                max_size,
            )
        });
        if !successful {
            return Err(FxOrParameterNotFound);
        }
        Ok(name)
    }

    /// Returns the given value formatted as string according to the given track FX parameter.
    ///
    /// This only works with FX that supports Cockos VST extensions.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
    ///
    /// # Errors
    ///
    /// Returns an error if the FX or parameter doesn't exist. Also errors if the FX doesn't support
    /// formatting arbitrary parameter values *and* the given value is not equal to the current
    /// one. If the given value is equal to the current one, it's just like calling
    /// [`track_fx_get_formatted_param_value`].
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    ///
    /// [`track_fx_get_formatted_param_value`]: #method.track_fx_get_formatted_param_value
    pub unsafe fn track_fx_format_param_value_normalized(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        param: u32,
        value: ReaperNormalizedFxParamValue,
        buf_sz: u32,
    ) -> Result<CString, FxOrParameterNotFoundOrCockosExtNotSupported>
    where
        S: MainThread,
    {
        assert!(buf_sz > 0);
        let (name, successful) = with_string_buffer(buf_sz, |buffer, max_size| {
            self.low.TrackFX_FormatParamValueNormalized(
                track.as_ptr(),
                fx.to_raw(),
                param as i32,
                value.get(),
                buffer,
                max_size,
            )
        });
        if !successful {
            return Err(FxOrParameterNotFoundOrCockosExtNotSupported);
        }
        Ok(name)
    }

    /// Sets the value of the given track FX parameter.
    ///
    /// # Errors
    ///
    /// Returns an error if the FX or parameter doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_set_param_normalized(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        param: u32,
        value: ReaperNormalizedFxParamValue,
    ) -> Result<(), FxOrParameterNotFound>
    where
        S: MainThread,
    {
        let successful = self.low.TrackFX_SetParamNormalized(
            track.as_ptr(),
            fx.to_raw(),
            param as i32,
            value.get(),
        );
        if !successful {
            return Err(FxOrParameterNotFound);
        }
        Ok(())
    }

    /// Returns information about the (last) focused FX window.
    ///
    /// Returns `Some` if an FX window has focus or was the last focused one and is still open.
    /// Returns `None` if no FX window has focus.
    pub fn get_focused_fx(&self) -> Option<GetFocusedFxResult>
    where
        S: MainThread,
    {
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
        let fxnumber = unsafe { fxnumber.assume_init() };
        use GetFocusedFxResult::*;
        match result {
            0 => None,
            1 => Some(TrackFx {
                track_ref: convert_tracknumber_to_track_ref(tracknumber),
                fx_location: TrackFxLocation::try_from_raw(fxnumber).unwrap(),
            }),
            2 => {
                // TODO-low Add test
                let fxnumber = fxnumber as u32;
                Some(TakeFx {
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

    /// Returns information about the last touched FX parameter.
    ///
    /// Returns `Some` if an FX parameter has been touched already and that FX is still existing.
    /// Returns `None` otherwise.
    pub fn get_last_touched_fx(&self) -> Option<GetLastTouchedFxResult>
    where
        S: MainThread,
    {
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
        let fxnumber = unsafe { fxnumber.assume_init() };
        let paramnumber = unsafe { paramnumber.assume_init() as u32 };
        use GetLastTouchedFxResult::*;
        if tracknumber_high_word == 0 {
            Some(TrackFx {
                track_ref: convert_tracknumber_to_track_ref(tracknumber),
                fx_location: TrackFxLocation::try_from_raw(fxnumber).unwrap(),
                // Although the parameter is called paramnumber, it's zero-based (checked)
                param_index: paramnumber,
            })
        } else {
            // TODO-low Add test
            let fxnumber = fxnumber as u32;
            Some(TakeFx {
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

    /// Copies, moves or reorders FX.
    ///
    /// Reorders if source and destination track are the same.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_copy_to_track(
        &self,
        src: (MediaTrack, TrackFxLocation),
        dest: (MediaTrack, TrackFxLocation),
        is_move: TransferBehavior,
    ) {
        self.low.TrackFX_CopyToTrack(
            src.0.as_ptr(),
            src.1.to_raw(),
            dest.0.as_ptr(),
            dest.1.to_raw(),
            is_move == TransferBehavior::Move,
        );
    }

    /// Removes the given FX from the track FX chain.
    ///
    /// # Errors
    ///
    /// Returns an error if the FX doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_delete(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
    ) -> Result<(), FxNotFound>
    where
        S: MainThread,
    {
        let succesful = self.low.TrackFX_Delete(track.as_ptr(), fx.to_raw());
        if !succesful {
            return Err(FxNotFound);
        }
        Ok(())
    }

    /// Returns information about the given FX parameter's step sizes.
    ///
    /// Returns `None` if the FX parameter doesn't report step sizes or if the FX or parameter
    /// doesn't exist (there's no way to distinguish with just this function).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    //
    // Option makes more sense than Result here because this function is at the same time the
    // correct function to be used to determine *if* a parameter reports step sizes. So
    // "parameter doesn't report step sizes" is a valid result.
    pub unsafe fn track_fx_get_parameter_step_sizes(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        param: u32,
    ) -> Option<GetParameterStepSizesResult>
    where
        S: MainThread,
    {
        let mut step = MaybeUninit::uninit();
        let mut small_step = MaybeUninit::uninit();
        let mut large_step = MaybeUninit::uninit();
        let mut is_toggle = MaybeUninit::uninit();
        let successful = self.low.TrackFX_GetParameterStepSizes(
            track.as_ptr(),
            fx.to_raw(),
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

    /// Returns the current value and min/mid/max values of the given track FX.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_param_ex(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        param: u32,
    ) -> GetParamExResult
    where
        S: MainThread,
    {
        let mut min_val = MaybeUninit::uninit();
        let mut max_val = MaybeUninit::uninit();
        let mut mid_val = MaybeUninit::uninit();
        let value = self.low.TrackFX_GetParamEx(
            track.as_ptr(),
            fx.to_raw(),
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

    /// Starts a new undo block.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let reaper = reaper_rs_medium::Reaper::default();
    /// use reaper_rs_medium::{ProjectContext::CurrentProject, UndoScope::Scoped, ProjectPart::*};
    ///
    /// reaper.undo_begin_block_2(CurrentProject);
    /// // ... modify something ...
    /// reaper.undo_end_block_2(CurrentProject, "Modify something", Scoped(Items | Fx));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn undo_begin_block_2(&self, proj: ProjectContext) {
        self.require_valid_project(proj);
        unsafe { self.undo_begin_block_2_unchecked(proj) };
    }

    /// Like [`undo_begin_block_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`undo_begin_block_2()`]: #method.undo_begin_block_2
    pub unsafe fn undo_begin_block_2_unchecked(&self, proj: ProjectContext) {
        self.low.Undo_BeginBlock2(proj.to_raw());
    }

    /// Ends the current undo block.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn undo_end_block_2<'a>(
        &self,
        proj: ProjectContext,
        descchange: impl Into<ReaperStringArg<'a>>,
        extraflags: UndoScope,
    ) {
        self.require_valid_project(proj);
        unsafe {
            self.undo_end_block_2_unchecked(proj, descchange, extraflags);
        }
    }

    /// Like [`undo_end_block_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`undo_end_block_2()`]: #method.undo_end_block_2
    pub unsafe fn undo_end_block_2_unchecked<'a>(
        &self,
        proj: ProjectContext,
        descchange: impl Into<ReaperStringArg<'a>>,
        extraflags: UndoScope,
    ) {
        self.low.Undo_EndBlock2(
            proj.to_raw(),
            descchange.into().as_ptr(),
            extraflags.to_raw(),
        );
    }

    /// Grants temporary access to the the description of the last undoable operation, if any.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn undo_can_undo_2<R>(&self, proj: ProjectContext, f: impl FnOnce(&CStr) -> R) -> Option<R>
    where
        S: MainThread,
    {
        self.require_valid_project(proj);
        unsafe { self.undo_can_undo_2_unchecked(proj, f) }
    }

    /// Like [`undo_can_undo_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`undo_can_undo_2()`]: #method.undo_can_undo_2
    pub unsafe fn undo_can_undo_2_unchecked<R>(
        &self,
        proj: ProjectContext,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R>
    where
        S: MainThread,
    {
        let ptr = self.low.Undo_CanUndo2(proj.to_raw());
        create_passing_c_str(ptr).map(f)
    }

    /// Grants temporary access to the description of the next redoable operation, if any.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn undo_can_redo_2<R>(&self, proj: ProjectContext, f: impl FnOnce(&CStr) -> R) -> Option<R>
    where
        S: MainThread,
    {
        self.require_valid_project(proj);
        unsafe { self.undo_can_redo_2_unchecked(proj, f) }
    }

    /// Like [`undo_can_redo_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`undo_can_redo_2()`]: #method.undo_can_redo_2
    pub unsafe fn undo_can_redo_2_unchecked<R>(
        &self,
        proj: ProjectContext,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R>
    where
        S: MainThread,
    {
        let ptr = self.low.Undo_CanRedo2(proj.to_raw());
        create_passing_c_str(ptr).map(f)
    }

    /// Makes the last undoable operation undone.
    ///
    /// Returns `false` if there was nothing to be undone.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn undo_do_undo_2(&self, proj: ProjectContext) -> bool
    where
        S: MainThread,
    {
        self.require_valid_project(proj);
        unsafe { self.undo_do_undo_2_unchecked(proj) }
    }

    /// Like [`undo_do_undo_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`undo_do_undo_2()`]: #method.undo_do_undo_2
    pub unsafe fn undo_do_undo_2_unchecked(&self, proj: ProjectContext) -> bool
    where
        S: MainThread,
    {
        self.low.Undo_DoUndo2(proj.to_raw()) != 0
    }

    /// Executes the next redoable action.
    ///
    /// Returns `false` if there was nothing to be redone.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn undo_do_redo_2(&self, proj: ProjectContext) -> bool
    where
        S: MainThread,
    {
        self.require_valid_project(proj);
        unsafe { self.undo_do_redo_2_unchecked(proj) }
    }

    /// Like [`undo_do_redo_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`undo_do_redo_2()`]: #method.undo_do_redo_2
    pub unsafe fn undo_do_redo_2_unchecked(&self, proj: ProjectContext) -> bool
    where
        S: MainThread,
    {
        self.low.Undo_DoRedo2(proj.to_raw()) != 0
    }

    /// Marks the given project as dirty.
    ///
    /// *Dirty* means the project needs to be saved. Only makes a difference if "Maximum undo
    /// memory" is not 0 in REAPER's preferences (0 disables undo/prompt to save).
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn mark_project_dirty(&self, proj: ProjectContext) {
        self.require_valid_project(proj);
        unsafe {
            self.mark_project_dirty_unchecked(proj);
        }
    }

    /// Like [`mark_project_dirty()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`mark_project_dirty()`]: #method.mark_project_dirty
    pub unsafe fn mark_project_dirty_unchecked(&self, proj: ProjectContext) {
        self.low.MarkProjectDirty(proj.to_raw());
    }

    /// Returns whether the given project is dirty.
    ///
    /// Always returns `false` if "Maximum undo memory" is 0 in REAPER's preferences.
    ///
    /// Also see [`mark_project_dirty()`]
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    ///
    /// [`mark_project_dirty()`]: #method.mark_project_dirty
    pub fn is_project_dirty(&self, proj: ProjectContext) -> bool
    where
        S: MainThread,
    {
        self.require_valid_project(proj);
        unsafe { self.is_project_dirty_unchecked(proj) }
    }

    /// Like [`is_project_dirty()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`is_project_dirty()`]: #method.is_project_dirty
    pub unsafe fn is_project_dirty_unchecked(&self, proj: ProjectContext) -> bool
    where
        S: MainThread,
    {
        self.low.IsProjectDirty(proj.to_raw()) != 0
    }

    /// Notifies all control surfaces that something in the track list has changed.
    ///
    /// Behavior not confirmed.
    pub fn track_list_update_all_external_surfaces(&self) {
        self.low.TrackList_UpdateAllExternalSurfaces();
    }

    /// Returns the version of the REAPER application in which this plug-in is currently running.
    pub fn get_app_version(&self) -> ReaperVersion<'static>
    where
        S: MainThread,
    {
        let ptr = self.low.GetAppVersion();
        let version_str = unsafe { CStr::from_ptr(ptr) };
        ReaperVersion::new(version_str)
    }

    /// Returns the track automation mode, regardless of the global override.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_automation_mode(&self, tr: MediaTrack) -> AutomationMode
    where
        S: MainThread,
    {
        let result = self.low.GetTrackAutomationMode(tr.as_ptr());
        AutomationMode::try_from(result).expect("Unknown automation mode")
    }

    /// Returns the global track automation override, if any.
    pub fn get_global_automation_override(&self) -> Option<GlobalAutomationModeOverride>
    where
        S: MainThread,
    {
        use GlobalAutomationModeOverride::*;
        match self.low.GetGlobalAutomationOverride() {
            -1 => None,
            6 => Some(Bypass),
            x => Some(Mode(x.try_into().expect("Unknown automation mode"))),
        }
    }

    /// Returns the track envelope for the given track and configuration chunk name.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    // TODO-low Test
    pub unsafe fn get_track_envelope_by_chunk_name(
        &self,
        track: MediaTrack,
        cfgchunkname: EnvChunkName,
    ) -> Option<TrackEnvelope>
    where
        S: MainThread,
    {
        let ptr = self
            .low
            .GetTrackEnvelopeByChunkName(track.as_ptr(), cfgchunkname.into_raw().as_ptr());
        NonNull::new(ptr)
    }

    /// Returns the track envelope for the given track and envelope display name.
    ///
    /// For getting common envelopes (like volume or pan) using
    /// [`get_track_envelope_by_chunk_name()`] is better because it provides more type safety.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    ///
    /// [`get_track_envelope_by_chunk_name()`]: #method.get_track_envelope_by_chunk_name
    pub unsafe fn get_track_envelope_by_name<'a>(
        &self,
        track: MediaTrack,
        envname: impl Into<ReaperStringArg<'a>>,
    ) -> Option<TrackEnvelope>
    where
        S: MainThread,
    {
        let ptr = self
            .low
            .GetTrackEnvelopeByName(track.as_ptr(), envname.into().as_ptr());
        NonNull::new(ptr)
    }

    /// Gets a track attribute as numerical value.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_media_track_info_value(&self, tr: MediaTrack, parmname: TrackInfoKey) -> f64
    where
        S: MainThread,
    {
        self.low
            .GetMediaTrackInfo_Value(tr.as_ptr(), parmname.into_raw().as_ptr())
    }

    /// Gets the number of FX instances on the given track's normal FX chain.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_count(&self, track: MediaTrack) -> u32
    where
        S: MainThread,
    {
        self.low.TrackFX_GetCount(track.as_ptr()) as u32
    }

    /// Gets the number of FX instances on the given track's input FX chain.
    ///
    /// On the master track, this refers to the monitoring FX chain.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_rec_count(&self, track: MediaTrack) -> u32
    where
        S: MainThread,
    {
        self.low.TrackFX_GetRecCount(track.as_ptr()) as u32
    }

    /// Returns the GUID of the given track FX.
    ///
    /// # Errors
    ///
    /// Returns an error if the FX doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_fx_guid(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
    ) -> Result<GUID, FxNotFound>
    where
        S: MainThread,
    {
        let ptr = self.low.TrackFX_GetFXGUID(track.as_ptr(), fx.to_raw());
        unref(ptr).ok_or(FxNotFound)
    }

    /// Returns the current value of the given track FX in REAPER-normalized form.
    ///
    /// # Errors
    ///
    /// Returns an error if the FX or parameter doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_param_normalized(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        param: u32,
    ) -> Result<ReaperNormalizedFxParamValue, FxOrParameterNotFound>
    where
        S: MainThread,
    {
        let raw_value =
            self.low
                .TrackFX_GetParamNormalized(track.as_ptr(), fx.to_raw(), param as i32);
        if raw_value < 0.0 {
            return Err(FxOrParameterNotFound);
        }
        Ok(ReaperNormalizedFxParamValue::new(raw_value))
    }

    /// Returns the master track of the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_master_track(&self, proj: ProjectContext) -> MediaTrack
    where
        S: MainThread,
    {
        self.require_valid_project(proj);
        unsafe { self.get_master_track_unchecked(proj) }
    }

    /// Like [`get_master_track()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_master_track()`]: #method.get_master_track
    pub unsafe fn get_master_track_unchecked(&self, proj: ProjectContext) -> MediaTrack
    where
        S: MainThread,
    {
        let ptr = self.low.GetMasterTrack(proj.to_raw());
        require_non_null_panic(ptr)
    }

    /// Converts the given GUID to a string (including braces).
    // TODO-medium Introduce GUID newtype
    pub fn guid_to_string(&self, g: &GUID) -> CString
    where
        S: MainThread,
    {
        let (guid_string, _) = with_string_buffer(64, |buffer, _| unsafe {
            self.low.guidToString(g as *const GUID, buffer)
        });
        guid_string
    }

    /// Returns the master tempo of the current project.
    pub fn master_get_tempo(&self) -> Bpm
    where
        S: MainThread,
    {
        Bpm(self.low.Master_GetTempo())
    }

    /// Sets the current tempo of the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn set_current_bpm(&self, proj: ProjectContext, bpm: Bpm, want_undo: UndoBehavior) {
        self.require_valid_project(proj);
        unsafe {
            self.set_current_bpm_unchecked(proj, bpm, want_undo);
        }
    }

    /// Like [`set_current_bpm()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`set_current_bpm()`]: #method.set_current_bpm
    pub unsafe fn set_current_bpm_unchecked(
        &self,
        proj: ProjectContext,
        bpm: Bpm,
        want_undo: UndoBehavior,
    ) {
        self.low.SetCurrentBPM(
            proj.to_raw(),
            bpm.get(),
            want_undo == UndoBehavior::AddUndoPoint,
        );
    }

    /// Returns the master play rate of the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn master_get_play_rate(&self, project: ProjectContext) -> PlaybackSpeedFactor
    where
        S: MainThread,
    {
        self.require_valid_project(project);
        unsafe { self.master_get_play_rate_unchecked(project) }
    }

    /// Like [`master_get_play_rate()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`master_get_play_rate()`]: #method.master_get_play_rate
    pub unsafe fn master_get_play_rate_unchecked(
        &self,
        project: ProjectContext,
    ) -> PlaybackSpeedFactor
    where
        S: MainThread,
    {
        let raw = unsafe { self.low.Master_GetPlayRate(project.to_raw()) };
        PlaybackSpeedFactor(raw)
    }

    /// Sets the master play rate of the current project.
    pub fn csurf_on_play_rate_change(&self, playrate: PlaybackSpeedFactor) {
        self.low.CSurf_OnPlayRateChange(playrate.get());
    }

    /// Shows a message box to the user.
    ///
    /// Blocks the main thread.
    pub fn show_message_box<'a>(
        &self,
        msg: impl Into<ReaperStringArg<'a>>,
        title: impl Into<ReaperStringArg<'a>>,
        r#type: MessageBoxType,
    ) -> MessageBoxResult
    where
        S: MainThread,
    {
        let result = unsafe {
            self.low
                .ShowMessageBox(msg.into().as_ptr(), title.into().as_ptr(), r#type.into())
        };
        result.try_into().expect("Unknown message box result")
    }

    /// Parses the given string as GUID.
    ///
    /// # Errors
    ///
    /// Returns an error if the given string is not a valid GUID expression.
    pub fn string_to_guid<'a>(
        &self,
        str: impl Into<ReaperStringArg<'a>>,
    ) -> Result<GUID, GuidExpressionInvalid>
    where
        S: MainThread,
    {
        let mut guid = MaybeUninit::uninit();
        unsafe {
            self.low
                .stringToGuid(str.into().as_ptr(), guid.as_mut_ptr());
        }
        let guid = unsafe { guid.assume_init() };
        if guid == ZERO_GUID {
            return Err(GuidExpressionInvalid);
        }
        Ok(guid)
    }

    /// Sets the input monitoring mode of the given track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_input_monitoring_change_ex(
        &self,
        trackid: MediaTrack,
        monitor: InputMonitoringMode,
        allowgang: GangBehavior,
    ) -> i32
    where
        S: MainThread,
    {
        self.low.CSurf_OnInputMonitorChangeEx(
            trackid.as_ptr(),
            monitor.into(),
            allowgang == GangBehavior::AllowGang,
        )
    }

    /// Sets a track attribute as numerical value.
    ///
    /// # Errors
    ///
    /// Returns an error if an invalid (e.g. non-numerical) track attribute key is passed.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    // TODO-medium Maybe rename TrackInfo to TrackAttribute.
    pub unsafe fn set_media_track_info_value(
        &self,
        tr: MediaTrack,
        parmname: TrackInfoKey,
        newvalue: f64,
    ) -> Result<(), InvalidTrackInfoKey>
    where
        S: MainThread,
    {
        let successful =
            self.low
                .SetMediaTrackInfo_Value(tr.as_ptr(), parmname.into_raw().as_ptr(), newvalue);
        if !successful {
            return Err(InvalidTrackInfoKey);
        }
        Ok(())
    }

    /// Stuffs a 3-byte MIDI message into a queue or send it to an external MIDI hardware.
    pub fn stuff_midimessage(&self, mode: StuffMidiMessageTarget, msg: impl ShortMessage) {
        let bytes = msg.to_bytes();
        self.low.StuffMIDIMessage(
            mode.to_raw(),
            bytes.0.into(),
            bytes.1.into(),
            bytes.2.into(),
        );
    }

    /// Converts a decibel value into a volume slider value.
    pub fn db2slider(&self, x: Db) -> VolumeSliderValue
    where
        S: MainThread,
    {
        VolumeSliderValue(self.low.DB2SLIDER(x.get()))
    }

    /// Converts a volume slider value into a decibel value.
    pub fn slider2db(&self, y: VolumeSliderValue) -> Db
    where
        S: MainThread,
    {
        Db(self.low.SLIDER2DB(y.get()))
    }

    /// Returns the given track's volume and pan.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful (unclear when this happens).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_ui_vol_pan(
        &self,
        track: MediaTrack,
    ) -> Result<VolumeAndPan, ReaperFunctionFailed>
    where
        S: MainThread,
    {
        let mut volume = MaybeUninit::uninit();
        let mut pan = MaybeUninit::uninit();
        let successful =
            self.low
                .GetTrackUIVolPan(track.as_ptr(), volume.as_mut_ptr(), pan.as_mut_ptr());
        if !successful {
            return Err(ReaperFunctionFailed);
        }
        Ok(VolumeAndPan {
            volume: ReaperVolumeValue::new(volume.assume_init()),
            pan: ReaperPanValue::new(pan.assume_init()),
        })
    }

    /// Sets the given track's volume.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_set_surface_volume(
        &self,
        trackid: MediaTrack,
        volume: ReaperVolumeValue,
        ignoresurf: NotificationBehavior,
    ) {
        self.low
            .CSurf_SetSurfaceVolume(trackid.as_ptr(), volume.get(), ignoresurf.to_raw());
    }

    /// Sets the given track's volume, also supports relative changes and gang.
    ///
    /// Returns the value that has actually been set. I think this only deviates if 0.0 is sent.
    /// Then it returns a slightly higher value - the one which actually corresponds to -150 dB.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_volume_change_ex(
        &self,
        trackid: MediaTrack,
        volume: ValueChange<ReaperVolumeValue>,
        allow_gang: GangBehavior,
    ) -> ReaperVolumeValue
    where
        S: MainThread,
    {
        let raw = self.low.CSurf_OnVolumeChangeEx(
            trackid.as_ptr(),
            volume.value(),
            volume.is_relative(),
            allow_gang == GangBehavior::AllowGang,
        );
        ReaperVolumeValue::new(raw)
    }

    /// Sets the given track's pan.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_set_surface_pan(
        &self,
        trackid: MediaTrack,
        pan: ReaperPanValue,
        ignoresurf: NotificationBehavior,
    ) {
        self.low
            .CSurf_SetSurfacePan(trackid.as_ptr(), pan.get(), ignoresurf.to_raw());
    }

    /// Sets the given track's pan. Also supports relative changes and gang.
    ///
    /// Returns the value that has actually been set.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_pan_change_ex(
        &self,
        trackid: MediaTrack,
        pan: ValueChange<ReaperPanValue>,
        allow_gang: GangBehavior,
    ) -> ReaperPanValue
    where
        S: MainThread,
    {
        let raw = self.low.CSurf_OnPanChangeEx(
            trackid.as_ptr(),
            pan.value(),
            pan.is_relative(),
            allow_gang == GangBehavior::AllowGang,
        );
        ReaperPanValue::new(raw)
    }

    /// Counts the number of selected tracks in the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn count_selected_tracks_2(
        &self,
        proj: ProjectContext,
        wantmaster: MasterTrackBehavior,
    ) -> u32
    where
        S: MainThread,
    {
        self.require_valid_project(proj);
        unsafe { self.count_selected_tracks_2_unchecked(proj, wantmaster) }
    }

    /// Like [`count_selected_tracks_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`count_selected_tracks_2()`]: #method.count_selected_tracks_2
    pub unsafe fn count_selected_tracks_2_unchecked(
        &self,
        proj: ProjectContext,
        wantmaster: MasterTrackBehavior,
    ) -> u32
    where
        S: MainThread,
    {
        self.low.CountSelectedTracks2(
            proj.to_raw(),
            wantmaster == MasterTrackBehavior::IncludeMasterTrack,
        ) as u32
    }

    /// Selects or deselects the given track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_selected(&self, track: MediaTrack, selected: bool) {
        self.low.SetTrackSelected(track.as_ptr(), selected);
    }

    /// Returns a selected track from the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_selected_track_2(
        &self,
        proj: ProjectContext,
        seltrackidx: u32,
        wantmaster: MasterTrackBehavior,
    ) -> Option<MediaTrack>
    where
        S: MainThread,
    {
        self.require_valid_project(proj);
        unsafe { self.get_selected_track_2_unchecked(proj, seltrackidx, wantmaster) }
    }

    /// Like [`get_selected_track_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_selected_track_2()`]: #method.get_selected_track_2
    pub unsafe fn get_selected_track_2_unchecked(
        &self,
        proj: ProjectContext,
        seltrackidx: u32,
        wantmaster: MasterTrackBehavior,
    ) -> Option<MediaTrack>
    where
        S: MainThread,
    {
        let ptr = self.low.GetSelectedTrack2(
            proj.to_raw(),
            seltrackidx as i32,
            wantmaster == MasterTrackBehavior::IncludeMasterTrack,
        );
        NonNull::new(ptr)
    }

    /// Selects exactly one track and deselects all others.
    ///
    /// If `None` is passed, deselects all tracks.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_only_track_selected(&self, track: Option<MediaTrack>) {
        let ptr = match track {
            None => null_mut(),
            Some(t) => t.as_ptr(),
        };
        self.low.SetOnlyTrackSelected(ptr);
    }

    /// Deletes the given track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn delete_track(&self, tr: MediaTrack) {
        self.low.DeleteTrack(tr.as_ptr());
    }

    /// Returns the number of sends, receives or hardware outputs of the given track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_num_sends(&self, tr: MediaTrack, category: TrackSendCategory) -> u32
    where
        S: MainThread,
    {
        self.low.GetTrackNumSends(tr.as_ptr(), category.into()) as u32
    }

    // Gets or sets an attributes of a send, receive or hardware output of the given track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track or invalid new value.
    pub unsafe fn get_set_track_send_info(
        &self,
        tr: MediaTrack,
        category: TrackSendCategory,
        sendidx: u32,
        parmname: TrackSendInfoKey,
        set_new_value: *mut c_void,
    ) -> *mut c_void
    where
        S: MainThread,
    {
        self.low.GetSetTrackSendInfo(
            tr.as_ptr(),
            category.into(),
            sendidx as i32,
            parmname.into_raw().as_ptr(),
            set_new_value,
        )
    }

    /// Convenience function which returns the destination track (`P_DESTTRACK`) of the given send
    /// or receive.
    ///
    /// # Errors
    ///
    /// Returns an error if the send or receive doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_send_info_desttrack(
        &self,
        tr: MediaTrack,
        category: TrackSendDirection,
        sendidx: u32,
    ) -> Result<MediaTrack, ReaperFunctionFailed>
    where
        S: MainThread,
    {
        let ptr = self.get_set_track_send_info(
            tr,
            category.into(),
            sendidx,
            TrackSendInfoKey::DestTrack,
            null_mut(),
        ) as *mut raw::MediaTrack;
        require_non_null(ptr).map_err(|_| ReaperFunctionFailed)
    }

    /// Returns the RPPXML state of the given track.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful (unclear when this happens).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_state_chunk(
        &self,
        track: MediaTrack,
        str_need_big_sz: u32,
        isundo_optional: ChunkCacheHint,
    ) -> Result<CString, ReaperFunctionFailed>
    where
        S: MainThread,
    {
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
            return Err(ReaperFunctionFailed);
        }
        Ok(chunk_content)
    }

    /// Creates a send, receive or hardware output for the given track.
    ///
    /// Returns the index of the created send or receive.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let reaper = reaper_rs_medium::Reaper::default();
    /// use reaper_rs_medium::{ProjectContext::CurrentProject, SendTarget::HardwareOutput};
    ///
    /// let src_track = reaper.get_track(CurrentProject, 0).ok_or("Source track doesn't exist")?;
    /// let send_index = unsafe {
    ///     reaper.create_track_send(src_track, HardwareOutput)?;
    /// };
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if not successful (unclear when this happens).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn create_track_send(
        &self,
        tr: MediaTrack,
        desttr_in_optional: SendTarget,
    ) -> Result<u32, CreateTrackSendFailed>
    where
        S: MainThread,
    {
        let result = self
            .low
            .CreateTrackSend(tr.as_ptr(), desttr_in_optional.to_raw());
        if result < 0 {
            return Err(CreateTrackSendFailed);
        }
        Ok(result as u32)
    }

    /// Arms or unarms the given track for recording.
    ///
    /// Seems to return `true` if it was armed and `false` if not.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_rec_arm_change_ex(
        &self,
        trackid: MediaTrack,
        recarm: RecordArmState,
        allowgang: GangBehavior,
    ) -> bool
    where
        S: MainThread,
    {
        self.low.CSurf_OnRecArmChangeEx(
            trackid.as_ptr(),
            recarm.into(),
            allowgang == GangBehavior::AllowGang,
        )
    }

    /// Sets the RPPXML state of the given track.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful (for example if the given chunk is not accepted).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_state_chunk<'a>(
        &self,
        track: MediaTrack,
        str: impl Into<ReaperStringArg<'a>>,
        isundo_optional: ChunkCacheHint,
    ) -> Result<(), ReaperFunctionFailed>
    where
        S: MainThread,
    {
        let successful = self.low.SetTrackStateChunk(
            track.as_ptr(),
            str.into().as_ptr(),
            isundo_optional == ChunkCacheHint::UndoMode,
        );
        if !successful {
            return Err(ReaperFunctionFailed);
        }
        Ok(())
    }

    /// Shows or hides an FX user interface.
    pub unsafe fn track_fx_show(&self, track: MediaTrack, show_flag: FxShowInstruction) {
        self.low.TrackFX_Show(
            track.as_ptr(),
            show_flag.location_to_raw(),
            show_flag.instruction_to_raw(),
        );
    }

    /// Returns the floating window handle of the given FX, if there is any.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_floating_window(
        &self,
        track: MediaTrack,
        index: TrackFxLocation,
    ) -> Option<Hwnd>
    where
        S: MainThread,
    {
        let ptr = self
            .low
            .TrackFX_GetFloatingWindow(track.as_ptr(), index.to_raw());
        NonNull::new(ptr)
    }

    /// Returns whether the user interface of the given FX is open.
    ///
    /// *Open* means either visible in the FX chain window or visible in a floating window.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_open(&self, track: MediaTrack, fx: TrackFxLocation) -> bool
    where
        S: MainThread,
    {
        self.low.TrackFX_GetOpen(track.as_ptr(), fx.to_raw())
    }

    /// Sets the given track send's volume.
    ///
    /// Returns the value that has actually been set. If the send doesn't exist, returns 0.0 (which
    /// can also be a valid value that has been set, so that's not very useful).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_send_volume_change(
        &self,
        trackid: MediaTrack,
        send_index: u32,
        volume: ValueChange<ReaperVolumeValue>,
    ) -> ReaperVolumeValue
    where
        S: MainThread,
    {
        let raw = self.low.CSurf_OnSendVolumeChange(
            trackid.as_ptr(),
            send_index as i32,
            volume.value(),
            volume.is_relative(),
        );
        ReaperVolumeValue::new(raw)
    }

    /// Sets the given track send's pan.
    ///
    /// Returns the value that has actually been set.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_send_pan_change(
        &self,
        trackid: MediaTrack,
        send_index: u32,
        pan: ValueChange<ReaperPanValue>,
    ) -> ReaperPanValue
    where
        S: MainThread,
    {
        let raw = self.low.CSurf_OnSendPanChange(
            trackid.as_ptr(),
            send_index as i32,
            pan.value(),
            pan.is_relative(),
        );
        ReaperPanValue::new(raw)
    }

    /// Grants temporary access to the name of the action registered under the given command ID
    /// within the specified section.
    ///
    /// Returns `None` if the action doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid section.
    pub unsafe fn kbd_get_text_from_cmd<R>(
        &self,
        cmd: CommandId,
        section: SectionContext,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R>
    where
        S: MainThread,
    {
        let ptr = self.low.kbd_getTextFromCmd(cmd.get(), section.to_raw());
        create_passing_c_str(ptr)
            // Removed action returns empty string for some reason. We want None in this case!
            .filter(|s| s.to_bytes().len() > 0)
            .map(f)
    }

    /// Returns the current on/off state of a toggleable action.
    ///
    /// Returns `None` if the action doesn't support on/off states (or if the action doesn't exist).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid section.
    //
    // Option makes more sense than Result here because this function is at the same time the
    // correct function to be used to determine *if* an action reports on/off states. So
    // "action doesn't report on/off states" is a valid result.
    pub unsafe fn get_toggle_command_state_2(
        &self,
        section: SectionContext,
        command_id: CommandId,
    ) -> Option<bool>
    where
        S: MainThread,
    {
        let result = self
            .low
            .GetToggleCommandState2(section.to_raw(), command_id.to_raw());
        if result == -1 {
            return None;
        }
        return Some(result != 0);
    }

    /// Grants temporary access to the name of the command registered under the given command ID.
    ///
    /// The string will *not* start with `_` (e.g. it will return `SWS_ABOUT`).
    ///
    /// Returns `None` if the given command ID is a built-in action or if there's no such ID.
    pub fn reverse_named_command_lookup<R>(
        &self,
        command_id: CommandId,
        f: impl FnOnce(&CStr) -> R,
    ) -> Option<R>
    where
        S: MainThread,
    {
        let ptr = self.low.ReverseNamedCommandLookup(command_id.to_raw());
        unsafe { create_passing_c_str(ptr) }.map(f)
    }

    /// Returns a track send's volume and pan.
    ///
    /// # Errors
    ///
    /// Returns an error if the send doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    // // send_idx>=0 for hw ouputs, >=nb_of_hw_ouputs for sends. See GetTrackReceiveUIVolPan.
    // Returns Err if send not existing
    pub unsafe fn get_track_send_ui_vol_pan(
        &self,
        track: MediaTrack,
        send_index: u32,
    ) -> Result<VolumeAndPan, ReaperFunctionFailed>
    where
        S: MainThread,
    {
        let mut volume = MaybeUninit::uninit();
        let mut pan = MaybeUninit::uninit();
        let successful = self.low.GetTrackSendUIVolPan(
            track.as_ptr(),
            send_index as i32,
            volume.as_mut_ptr(),
            pan.as_mut_ptr(),
        );
        if !successful {
            return Err(ReaperFunctionFailed);
        }
        Ok(VolumeAndPan {
            volume: ReaperVolumeValue::new(volume.assume_init()),
            pan: ReaperPanValue::new(pan.assume_init()),
        })
    }

    /// Returns the index of the currently selected FX preset as well as the total preset count.
    ///
    /// # Errors
    ///
    /// Returns an error e.g. if the FX doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_preset_index(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
    ) -> Result<TrackFxGetPresetIndexResult, ReaperFunctionFailed>
    where
        S: MainThread,
    {
        let mut num_presets = MaybeUninit::uninit();
        let index =
            self.low
                .TrackFX_GetPresetIndex(track.as_ptr(), fx.to_raw(), num_presets.as_mut_ptr());
        if index == -1 {
            return Err(ReaperFunctionFailed);
        }
        Ok(TrackFxGetPresetIndexResult {
            index: index as u32,
            count: num_presets.assume_init() as u32,
        })
    }

    /// Selects a preset of the given track FX.
    ///
    /// # Errors
    ///
    /// Returns an error e.g. if the FX doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_set_preset_by_index(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        idx: FxPresetRef,
    ) -> Result<(), ReaperFunctionFailed>
    where
        S: MainThread,
    {
        let successful =
            self.low
                .TrackFX_SetPresetByIndex(track.as_ptr(), fx.to_raw(), idx.to_raw());
        if !successful {
            return Err(ReaperFunctionFailed);
        }
        Ok(())
    }

    /// Navigates within the presets of the given track FX.
    ///
    /// TODO-example
    ///  presetmove==1 activates the next preset, presetmove==-1 activates the previous preset, etc.
    ///
    /// # Errors
    ///
    /// Returns an error e.g. if the FX doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_navigate_presets(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        presetmove: i32,
    ) -> Result<(), ReaperFunctionFailed>
    where
        S: MainThread,
    {
        let successful = self
            .low
            .TrackFX_NavigatePresets(track.as_ptr(), fx.to_raw(), presetmove);
        if !successful {
            return Err(ReaperFunctionFailed);
        }
        Ok(())
    }

    /// Returns information about the currently selected preset of the given FX.
    ///
    /// *Currently selected* means the preset which is currently showing in the REAPER dropdown.
    /// TODO-medium Try this: or the full path to a factory preset file for VST3 plug-ins
    ///  (.vstpreset). We might return paths in that case!?
    ///
    /// With `presetname_sz` you can tell REAPER how many characters of the preset name you want. If
    /// you are not interested in the preset name at all, pass 0.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_preset(
        &self,
        track: MediaTrack,
        fx: TrackFxLocation,
        presetname_sz: u32,
    ) -> TrackFxGetPresetResult
    where
        S: MainThread,
    {
        if presetname_sz == 0 {
            let state_matches_preset =
                self.low
                    .TrackFX_GetPreset(track.as_ptr(), fx.to_raw(), null_mut(), 0);
            TrackFxGetPresetResult {
                state_matches_preset,
                name: None,
            }
        } else {
            let (name, state_matches_preset) =
                with_string_buffer(presetname_sz, |buffer, max_size| {
                    self.low
                        .TrackFX_GetPreset(track.as_ptr(), fx.to_raw(), buffer, max_size)
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

    /// Grants temporary access to an already open MIDI input device.
    ///
    /// Returns `None` if the device doesn't exist, is not connected or is not already opened. The
    /// device must be enabled in REAPER's MIDI preferences.
    ///
    /// This function is typically called in the [audio hook]. But it's also okay to call it in a
    /// VST plug-in as long as [`is_in_real_time_audio()`] returns `true`.
    ///
    /// # Design
    ///
    /// The device is not just returned because then we would have to mark this function as unsafe.
    /// Returning the device would tempt the consumer to cache the pointer somewhere, which is bad
    /// because the MIDI device can appear/disappear anytime and REAPER doesn't notify us about it.
    /// If we would call [`get_read_buf()`] on a cached pointer and the MIDI device is gone, REAPER
    /// would crash.
    ///
    /// Calling this function in every audio hook invocation is fast enough and the official way
    /// to tap MIDI messages directly. Because of that we
    /// [take a closure and pass a reference](https://stackoverflow.com/questions/61106587).
    ///
    /// [audio hook]: struct.Reaper.html#method.audio_reg_hardware_hook_add
    /// [`is_in_real_time_audio()`]: #method.is_in_real_time_audio
    /// [`get_read_buf()`]: struct.MidiInput.html#method.get_read_buf
    pub fn get_midi_input<R>(
        &self,
        idx: MidiInputDeviceId,
        f: impl FnOnce(&MidiInput) -> R,
    ) -> Option<R>
    where
        S: AudioThread,
    {
        let ptr = self.low.GetMidiInput(idx.to_raw());
        if ptr.is_null() {
            return None;
        }
        NonNull::new(ptr).map(|nnp| f(&MidiInput(nnp)))
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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GetParameterStepSizesResult {
    /// Normal (non-toggleable) parameter.
    ///
    /// Each of the decimal numbers are > 0.
    Normal {
        step: f64,
        small_step: Option<f64>,
        large_step: Option<f64>,
    },
    /// Toggleable parameter.
    Toggle,
}

/// Each of these values can be negative! They are not normalized.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GetParamExResult {
    /// Current value.
    pub value: f64,
    /// Minimum possible value.
    pub min_val: f64,
    /// Center value.
    pub mid_val: f64,
    /// Maximum possible value.
    pub max_val: f64,
}

#[derive(Clone, PartialEq, Hash, Debug)]
pub struct EnumProjectsResult {
    /// Project pointer.
    pub project: ReaProject,
    /// Path to project file (only if project saved and path requested).
    pub file_path: Option<PathBuf>,
}

#[derive(Clone, PartialEq, Hash, Debug)]
pub struct GetMidiDevNameResult {
    /// Whether the device is currently connected.
    pub is_present: bool,
    /// Name of the device (only if name requested).
    pub name: Option<CString>,
}

#[derive(Clone, PartialEq, Hash, Debug)]
pub struct TrackFxGetPresetResult {
    /// Whether the current state of the FX matches the preset.
    ///
    /// `false` if the current FX parameters do not exactly match the preset (in other words, if
    /// the user loaded the preset but moved the knobs afterwards).
    pub state_matches_preset: bool,
    /// Name of the preset.
    pub name: Option<CString>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TrackFxGetPresetIndexResult {
    /// Preset index.
    pub index: u32,
    /// Total number of presets available.
    pub count: u32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct VolumeAndPan {
    /// Volume.
    pub volume: ReaperVolumeValue,
    /// Pan.
    pub pan: ReaperPanValue,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GetLastTouchedFxResult {
    /// The last touched FX is a track FX.
    TrackFx {
        /// Track on which the FX is located.
        track_ref: TrackRef,
        /// Location of the FX on that track.
        fx_location: TrackFxLocation,
        /// Index of the last touched parameter.
        param_index: u32,
    },
    /// The last touched FX is a take FX.
    TakeFx {
        /// Index of the track on which the item is located.
        track_index: u32,
        /// Index of the item on that track.
        ///
        /// **Attention:** It's an index, so it's zero-based (the one-based result from the
        /// low-level function has been transformed for more consistency).
        item_index: u32,
        /// Index of the take within the item.
        take_index: u32,
        /// Index of the FX within the take FX chain.
        fx_index: u32,
        /// Index of the last touched parameter.
        param_index: u32,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GetFocusedFxResult {
    /// The (last) focused FX is a track FX.
    TrackFx {
        /// Track on which the FX is located.
        track_ref: TrackRef,
        /// Location of the FX on that track.
        fx_location: TrackFxLocation,
    },
    /// The (last) focused FX is a take FX.
    TakeFx {
        /// Index of the track on which the item is located.
        track_index: u32,
        /// Index of the item on that track.
        item_index: u32,
        /// Index of the take within the item.
        take_index: u32,
        /// Index of the FX within the take FX chain.
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

const ZERO_GUID: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};
