use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::{null, null_mut, NonNull};

use reaper_low::raw;

use crate::ProjectContext::CurrentProject;
use crate::{
    require_media_track_panic, Accel, ActionValueChange, AddFxBehavior,
    AdvancePlaybackPositionEvent, AudioDeviceAttributeKey, AutoSeekBehavior, AutomationMode,
    BeatAttachMode, BookmarkId, BookmarkRef, Bpm, ChunkCacheHint, CommandId, CommandItem, Db,
    DurationInSeconds, EditMode, EnvChunkName, FadeCurvature, FadeShape, FullPitchShiftMode,
    FxAddByNameBehavior, FxChainVisibility, FxPresetRef, FxShowInstruction, GangBehavior,
    GetThemeColorFlags, GlobalAutomationModeOverride, HelpMode, Hidden, Hwnd, InitialAction,
    InputMonitoringMode, InsertMediaFlag, InsertMediaMode, ItemAttributeKey, ItemGroupId,
    KbdSectionInfo, MarkerOrRegionPosition, MasterTrackBehavior, MeasureMode, MediaItem,
    MediaItemTake, MediaTrack, MenuOrToolbarItem, MessageBoxResult, MessageBoxType,
    MidiImportBehavior, MidiInput, MidiInputDeviceId, MidiOutput, MidiOutputDeviceId, NativeColor,
    NormalizedPlayRate, NotificationBehavior, OpenMediaExplorerMode, OpenProjectBehavior,
    OwnedPcmSource, OwnedReaperPitchShift, OwnedReaperResample, PanMode, ParamId, PcmSource,
    PeakFileMode, PitchShiftMode, PitchShiftSubMode, PlaybackSpeedFactor, PluginContext,
    PositionDescriptor, PositionInBeats, PositionInPulsesPerQuarterNote, PositionInQuarterNotes,
    PositionInSeconds, Progress, ProjectContext, ProjectInfoAttributeKey, ProjectRef,
    PromptForActionResult, ReaProject, ReaperFunctionError, ReaperFunctionResult,
    ReaperNormalizedFxParamValue, ReaperPanLikeValue, ReaperPanValue, ReaperPointer, ReaperStr,
    ReaperString, ReaperStringArg, ReaperVersion, ReaperVolumeValue, ReaperWidthValue,
    RecordArmMode, RecordingInput, RecordingMode, ReorderTracksBehavior, RequiredViewMode,
    ResampleMode, SectionContext, SectionId, SendTarget, SetTrackUiFlags, SoloMode,
    StuffMidiMessageTarget, SubMenuStart, TakeAttributeKey, TimeModeOverride, TimeRangeType,
    TrackArea, TrackAttributeKey, TrackDefaultsBehavior, TrackEnvelope, TrackFxChainType,
    TrackFxLocation, TrackLocation, TrackMuteOperation, TrackMuteState, TrackPolarity,
    TrackPolarityOperation, TrackRecArmOperation, TrackSendAttributeKey, TrackSendCategory,
    TrackSendDirection, TrackSendRef, TrackSoloOperation, TransferBehavior, UiRefreshBehavior,
    UndoBehavior, UndoScope, ValueChange, VolumeSliderValue, WindowContext,
};
pub use reaper_common_types::RgbColor;
use reaper_common_types::{Hz, Semitones};

use helgoboss_midi::ShortMessage;
use reaper_low::raw::GUID;

use crate::ptr_wrappers::require_hwnd_panic;
use crate::util::{
    create_passing_c_str, with_buffer, with_string_buffer, with_string_buffer_cstring,
    with_string_buffer_prefilled,
};
use camino::{Utf8Path, Utf8PathBuf};
use enumflags2::BitFlags;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::num::NonZeroU32;
use std::sync::OnceLock;

/// Represents a privilege to execute functions which are safe to execute from any thread.
pub trait AnyThread: private::Sealed {}

/// Represents a privilege to execute functions which are only safe to execute from the main thread.
pub trait MainThreadOnly: AnyThread + private::Sealed {}

/// Represents a privilege to execute functions which are only safe to execute from the real-time
/// audio thread.
pub trait AudioThreadOnly: AnyThread + private::Sealed {}

/// A usage scope which unlocks all functions that are safe to execute from the main thread.
#[derive(Copy, Clone, Debug, Default)]
pub struct MainThreadScope(pub(crate) ());

impl MainThreadOnly for MainThreadScope {}

impl AnyThread for MainThreadScope {}

/// A usage scope which unlocks all functions that are safe to execute from the real-time audio
/// thread.
#[derive(Copy, Clone, Debug, Default)]
pub struct RealTimeAudioThreadScope(pub(crate) ());

impl AudioThreadOnly for RealTimeAudioThreadScope {}

impl AnyThread for RealTimeAudioThreadScope {}

/// This is the main access point for most REAPER functions.
///
/// # Basics
///
/// You can obtain an instance of this struct by calling [`ReaperSession::reaper()`]. This
/// unlocks all functions which are safe to execute in the main thread. If you want access to the
/// functions which are safe to execute in the real-time audio thread, call
/// [`ReaperSession::create_real_time_reaper()`] instead. REAPER functions which are related to
/// registering/unregistering things are located in [`ReaperSession`].
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
/// ## What's the `<MainThreadScope>` in `Reaper<MainThreadScope>` about?
///
/// In REAPER and probably many other DAWs there are at least two important threads:
///
/// 1. The main thread (responsible for things like UI, driven by the UI main loop).
/// 2. The real-time audio thread (responsible for processing audio and MIDI buffers, driven by the
///    audio hardware)
///
/// Most functions offered by REAPER are only safe to be executed in the main thread. If you execute
/// them in another thread, REAPER will crash. Or worse: It will seemingly work on your machine
/// and crash on someone else's. There are also a few functions which are only safe to be executed
/// in the audio thread. And there are also very few functions which are safe to be executed from
/// *any* thread (thread-safe).
///
/// There's currently no way to make sure at compile time that a function is called in the correct
/// thread. Of course that would be the best. In an attempt to still let the compiler help you a
/// bit, the traits [`MainThreadOnly`] and [`RealTimeAudioThreadOnly`] have been introduced. They
/// are marker traits which are used as type bound on each method which is not thread-safe. So
/// depending on the context we can expose an instance of [`Reaper`] which has only
/// functions unlocked which are safe to be executed from e.g. the real-time audio thread. The
/// compiler will complain if you attempt to call a real-time-audio-thread-only method on
/// `Reaper<MainThreadScope>` and vice versa.
///
/// Of course that technique can't prevent anyone from acquiring a main-thread only instance and
/// use it in the audio hook. But still, it adds some extra safety.
///
/// The alternative to tagging functions via marker traits would have been to implement e.g.
/// audio-thread-only functions in a trait `CallableFromRealTimeAudioThread` as default functions
/// and create a struct that inherits those default functions. Disadvantage: Consumer always would
/// have to bring the trait into scope to see the functions. That's confusing. It also would provide
/// less amount of safety.
///
/// ## Why no fail-fast at runtime when calling audio-thread-only functions from wrong thread?
///
/// At the moment, there's a fail fast (panic) when attempting to execute main-thread-only functions
/// from the wrong thread. This prevents "it works on my machine" scenarios. However, this is
/// currently not being done the other way around (when executing real-time-audio-thread-only
/// functions from the wrong thread) because of possible performance implications. Latter scenario
/// should also be much more unlikely. Maybe we can introduce it in future in order to really avoid
/// undefined behavior even for those methods (which the lack of `unsafe` suggests). Checking the
/// thread ID is a very cheap operation (a few nano seconds), maybe even in the real-time audio
/// thread.
///
/// [`ReaperSession`]: struct.ReaperSession.html
/// [`ReaperSession::reaper()`]: struct.ReaperSession.html#method.reaper
/// [`ReaperSession::create_real_time_reaper()`]:
/// struct.ReaperSession.html#method.create_real_time_reaper
/// [`low()`]: #method.low
/// [low-level `Reaper`]: https://docs.rs/reaper-low
/// [`MainThreadOnly`]: trait.MainThreadOnly.html
/// [`RealTimeAudioThreadOnly`]: trait.RealTimeAudioThreadOnly.html
/// [`Reaper`]: struct.Reaper.html
#[derive(Clone, Debug, Default)]
pub struct Reaper<UsageScope = MainThreadScope> {
    low: reaper_low::Reaper,
    features: ReaperFeatures,
    p: PhantomData<UsageScope>,
}

static INSTANCE: OnceLock<Reaper<MainThreadScope>> = OnceLock::new();

impl Reaper<MainThreadScope> {
    /// Makes the given instance available globally.
    ///
    /// After this has been called, the instance can be queried globally using `get()`.
    ///
    /// This can be called once only. Subsequent calls won't have any effect!
    #[allow(clippy::result_large_err)]
    pub fn make_available_globally(
        reaper: Reaper<MainThreadScope>,
    ) -> Result<(), Reaper<MainThreadScope>> {
        INSTANCE.set(reaper)
    }

    /// Gives access to the instance which you made available globally before.
    ///
    /// # Panics
    ///
    /// This panics if [`make_available_globally()`] has not been called before.
    ///
    /// [`make_available_globally()`]: fn.make_available_globally.html
    pub fn get() -> &'static Reaper<MainThreadScope> {
        INSTANCE
            .get()
            .expect("call `make_available_globally()` before using `get()`")
    }
}

/// Features of a particular REAPER version.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct ReaperFeatures {
    /// Whether it is safe to call [`Reaper::show_console_msg`] from any thread (vs. just the main thread).
    pub show_console_msg_from_any_thread: bool,
}

impl ReaperFeatures {
    fn from_reaper_version(version: &ReaperVersion) -> Self {
        Self {
            show_console_msg_from_any_thread: version.revision() >= "7",
        }
    }
}

impl<UsageScope> Reaper<UsageScope>
where
    UsageScope: AnyThread,
{
    pub(crate) fn new(low: reaper_low::Reaper) -> Reaper<UsageScope> {
        let mut reaper = Reaper {
            low,
            p: PhantomData,
            features: Default::default(),
        };
        let version = reaper.get_app_version();
        reaper.features = ReaperFeatures::from_reaper_version(&version);
        reaper
    }

    /// Gives access to the low-level Reaper instance.
    pub fn low(&self) -> &reaper_low::Reaper {
        &self.low
    }

    /// Returns the plug-in context.
    pub fn plugin_context(&self) -> PluginContext<UsageScope> {
        PluginContext::new(self.low.plugin_context())
    }

    /// Returns the features supported by this REAPER version.
    pub fn features(&self) -> &ReaperFeatures {
        &self.features
    }

    /// Returns the requested project and optionally its file name.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the file name you want. If you
    /// are not interested in the file name at all, pass 0.
    ///
    /// # Threading
    ///
    /// If `buffer_size > 0`, this must be called from the main thread (panics if not).
    ///
    /// If `buffer_size == 0`, this may also be called from a real-time or worker thread, not from
    /// your own thread (this won't be checked!)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::ProjectRef::Tab;
    ///
    /// let result = session.reaper().enum_projects(Tab(4), 256).ok_or("No such tab")?;
    /// let project_dir = result.file_path.ok_or("Project not saved yet")?.parent();
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn enum_projects(
        &self,
        project_ref: ProjectRef,
        buffer_size: u32,
    ) -> Option<EnumProjectsResult>
    where
        UsageScope: AnyThread,
    {
        if buffer_size > 0 {
            assert!(
                self.low.plugin_context().is_in_main_thread(),
                "enum_projects must only be called from main thread if buffer_size > 0"
            );
        }
        let idx = project_ref.to_raw();
        if buffer_size == 0 {
            let ptr = unsafe { self.low.EnumProjects(idx, null_mut(), 0) };
            let project = ReaProject::new(ptr)?;
            Some(EnumProjectsResult {
                project,
                file_path: None,
            })
        } else {
            let (reaper_string, ptr) = with_string_buffer(buffer_size, |buffer, max_size| unsafe {
                self.low.EnumProjects(idx, buffer, max_size)
            });
            let project = ReaProject::new(ptr)?;
            if reaper_string.is_empty() {
                return Some(EnumProjectsResult {
                    project,
                    file_path: None,
                });
            }
            let owned_string = reaper_string.into_string();
            let res = EnumProjectsResult {
                project,
                file_path: Some(Utf8PathBuf::from(owned_string)),
            };
            Some(res)
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
    /// # let session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::ProjectContext::CurrentProject;
    ///
    /// let track = session.reaper().get_track(CurrentProject, 3).ok_or("No such track")?;
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_track(&self, project: ProjectContext, track_index: u32) -> Option<MediaTrack>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.get_track_unchecked(project, track_index) }
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
        project: ProjectContext,
        track_index: u32,
    ) -> Option<MediaTrack>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.GetTrack(project.to_raw(), track_index as i32);
        MediaTrack::new(ptr)
    }

    /// Returns the item at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_media_item(&self, project: ProjectContext, item_index: u32) -> Option<MediaItem>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.get_media_item_unchecked(project, item_index) }
    }

    /// Like [`get_media_item()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_media_item()`]: #method.get_media_item
    pub unsafe fn get_media_item_unchecked(
        &self,
        project: ProjectContext,
        item_index: u32,
    ) -> Option<MediaItem>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.GetMediaItem(project.to_raw(), item_index as i32);
        MediaItem::new(ptr)
    }

    /// Checks if the given pointer is still valid.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::ProjectContext::CurrentProject;
    ///
    /// let track = session.reaper().get_track(CurrentProject, 0).ok_or("No track")?;
    /// let track_is_valid = session.reaper().validate_ptr_2(CurrentProject, track);
    /// assert!(track_is_valid);
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// Returns `true` if the pointer is a valid object of the correct type in the given project.
    /// The project is ignored if the pointer itself is a project.
    pub fn validate_ptr_2<'a>(
        &self,
        project: ProjectContext,
        pointer: impl Into<ReaperPointer<'a>>,
    ) -> bool
    where
        UsageScope: AnyThread,
    {
        let pointer = pointer.into();
        unsafe {
            self.low.ValidatePtr2(
                project.to_raw(),
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
        UsageScope: AnyThread,
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
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.UpdateTimeline();
    }

    /// Redraws the arrange view.
    pub fn update_arrange(&self)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.UpdateArrange();
    }

    pub fn get_theme_color<'a>(
        &self,
        ini_key: impl Into<ReaperStringArg<'a>>,
        flags: BitFlags<GetThemeColorFlags>,
    ) -> ReaperFunctionResult<NativeColor>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let color = unsafe {
            self.low
                .GetThemeColor(ini_key.into().as_ptr(), flags.bits() as _)
        };
        if color == -1 {
            return Err(ReaperFunctionError::new("failed to get theme color"));
        }
        Ok(NativeColor::new(color))
    }

    /// Updates the track list after a minor change.
    pub fn track_list_adjust_windows_minor(&self)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.TrackList_AdjustWindows(true);
    }

    /// Updates the track list after a major change.
    pub fn track_list_adjust_windows_major(&self)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.TrackList_AdjustWindows(false);
    }

    /// Opens the given file.
    ///
    /// If it's a project, opens the project. If it's a track template, adds the template to the
    /// current project.
    ///
    /// This is also useful for debugging. Send "\n" for newline and "" to clear the console.
    pub fn main_open_project(&self, file: &Utf8Path, behavior: OpenProjectBehavior)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut expression = String::new();
        if behavior.open_as_template {
            expression += "template:";
        }
        if !behavior.prompt {
            expression += "noprompt:";
        }
        expression += file.as_str();
        let expression_reaper_string = ReaperString::from_string(expression);
        unsafe {
            self.low.Main_openProject(expression_reaper_string.as_ptr());
        }
    }

    /// Shows a message to the user in the ReaScript console.
    ///
    /// This is also useful for debugging. Send "\n" for newline and "" to clear the console.
    ///
    /// The message supports the following prefixes:
    ///
    /// - `!SHOW:`: Text will be added to console without opening the window
    /// - `!SHOWERR:` Displays error indicator in main menu bar if ReaScript Console not already visible
    ///
    /// # Panics
    ///
    /// In REAPER versions < 7, this panics when not called from the main thread.
    pub fn show_console_msg<'a>(&self, message: impl Into<ReaperStringArg<'a>>)
    where
        UsageScope: AnyThread,
    {
        if !self.features.show_console_msg_from_any_thread {
            self.require_main_thread();
        }
        unsafe { self.low.ShowConsoleMsg(message.into().as_ptr()) }
    }

    /// Returns the hypothetical path of the peak file for the given audio file.
    ///
    /// It can either be `<file_name>.reapeaks` or a hashed file name in another path.
    ///
    /// `peaks_file_extension` should usually be `.reapeaks` but it can also be another extension if
    /// you want to store/read other metadata. E.g. the MP3 side uses ".reapindex" for the MP3
    /// seeking index file.
    ///
    /// This returns a [`CString`] instead of a [`Utf8PathBuf`] because REAPER versions < v7.15+dev0509 had
    /// a bug that could cause the resulting string to not be proper UTF-8 if the
    /// system temp path included non-ASCII characters. So one must be careful when interpreting the result.
    pub fn get_peak_file_name_ex_2<'a>(
        &self,
        file_name: &Utf8Path,
        buffer_size: u32,
        mode: PeakFileMode,
        peaks_file_extension: impl Into<ReaperStringArg<'a>>,
    ) -> CString
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let file_name_reaper_string = convert_path_to_reaper_string(file_name);
        let (reaper_string, _) = with_string_buffer(buffer_size, |buffer, max_size| unsafe {
            self.low.GetPeakFileNameEx2(
                file_name_reaper_string.as_ptr(),
                buffer,
                max_size,
                mode.to_raw(),
                peaks_file_extension.into().as_ptr(),
            );
        });
        reaper_string.into_inner()
    }

    /// Gets or sets a track attribute.
    ///
    /// Returns the current value if `new_value` is `null_mut()`.
    ///
    /// It's recommended to use one of the convenience functions instead. They all start with
    /// `get_set_media_track_info_` and are more type-safe.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track or invalid new value.
    pub unsafe fn get_set_media_track_info(
        &self,
        track: MediaTrack,
        attribute_key: TrackAttributeKey,
        new_value: *mut c_void,
    ) -> *mut c_void
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .GetSetMediaTrackInfo(track.as_ptr(), attribute_key.into_raw().as_ptr(), new_value)
    }

    /// Gets or sets a take attribute.
    ///
    /// Returns the current value if `new_value` is `null_mut()`.
    ///
    /// It's recommended to use one of the convenience functions instead. They all start with
    /// `get_set_media_item_take_info_` and are more type-safe.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take or invalid new value.
    pub unsafe fn get_set_media_item_take_info(
        &self,
        take: MediaItemTake,
        attribute_key: TakeAttributeKey,
        new_value: *mut c_void,
    ) -> *mut c_void
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.GetSetMediaItemTakeInfo(
            take.as_ptr(),
            attribute_key.into_raw().as_ptr(),
            new_value,
        )
    }

    /// Adds an "Extensions" main menu (if not already added), which the extension can populate/modify with
    /// [`crate::ReaperSession::plugin_register_add_hook_custom_menu`].
    pub fn add_extensions_main_menu(&self)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.AddExtensionsMainMenu();
    }

    /// Gets or sets an item attribute.
    ///
    /// Returns the current value if `new_value` is `null_mut()`.
    ///
    /// It's recommended to use one of the convenience functions instead. They all start with
    /// `get_set_media_item_info_` and are more type-safe.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item or invalid new value.
    pub unsafe fn get_set_media_item_info(
        &self,
        item: MediaItem,
        attribute_key: ItemAttributeKey,
        new_value: *mut c_void,
    ) -> *mut c_void
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .GetSetMediaItemInfo(item.as_ptr(), attribute_key.into_raw().as_ptr(), new_value)
    }

    /// Gets a media item attribute as numerical value.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_media_item_info_value(
        &self,
        item: MediaItem,
        attribute_key: ItemAttributeKey,
    ) -> f64
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .GetMediaItemInfo_Value(item.as_ptr(), attribute_key.into_raw().as_ptr())
    }

    /// Returns the MIDI tick (PPQ) position corresponding to a specific project time in
    /// quarter notes.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take.
    pub unsafe fn midi_get_ppq_pos_from_proj_qn(
        &self,
        take: MediaItemTake,
        qn: PositionInQuarterNotes,
    ) -> PositionInPulsesPerQuarterNote
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let pos = self.low.MIDI_GetPPQPosFromProjQN(take.as_ptr(), qn.get());
        PositionInPulsesPerQuarterNote::new_panic(pos)
    }

    /// Gets a media item take attribute as numerical value.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_media_item_take_info_value(
        &self,
        take: MediaItemTake,
        attribute_key: TakeAttributeKey,
    ) -> f64
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .GetMediaItemTakeInfo_Value(take.as_ptr(), attribute_key.into_raw().as_ptr())
    }

    /// Sets a take attribute as numerical value.
    ///
    /// # Errors
    ///
    /// Returns an error if an invalid (e.g. non-numerical) take attribute key is passed.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take.
    pub unsafe fn set_media_item_take_info_value(
        &self,
        take: MediaItemTake,
        attribute_key: TakeAttributeKey,
        new_value: f64,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.SetMediaItemTakeInfo_Value(
            take.as_ptr(),
            attribute_key.into_raw().as_ptr(),
            new_value,
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't set take attribute (maybe attribute key is invalid)",
            ));
        }
        Ok(())
    }

    /// Sets an item attribute as numerical value.
    ///
    /// # Errors
    ///
    /// Returns an error if an invalid (e.g. non-numerical) item attribute key is passed.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn set_media_item_info_value(
        &self,
        item: MediaItem,
        attribute_key: ItemAttributeKey,
        new_value: f64,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.SetMediaItemInfo_Value(
            item.as_ptr(),
            attribute_key.into_raw().as_ptr(),
            new_value,
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't set item attribute (maybe attribute key is invalid)",
            ));
        }
        Ok(())
    }

    /// Convenience function which sets the take's source (`P_SOURCE`).
    ///
    /// Returns the previous source in case the take had a source assigned.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take.
    pub unsafe fn get_set_media_item_take_info_set_source(
        &self,
        take: MediaItemTake,
        source: OwnedPcmSource,
    ) -> Option<OwnedPcmSource>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        // According to the docs we must obtain the old source before setting a new one.
        // The docs also say that it's now our responsibility to free the old source, which is why
        // we return it as `OwnedPcmSource`. If the caller does nothing with the old source,
        // it will be freed automatically (RAII), nice!
        let previous_source_ptr =
            self.get_set_media_item_take_info(take, TakeAttributeKey::Source, null_mut())
                as *mut raw::PCM_source;
        // We pass ownership of the new source to REAPER, that's what the leak does! If the take
        // gets deleted, REAPER will free the source accordingly. The only way of getting back
        // ownership of the old source is to replace it with yet another one. Rust ownership
        // paradigms at its best!
        let new_source_ptr = source.leak().as_ptr();
        self.get_set_media_item_take_info(take, TakeAttributeKey::Source, new_source_ptr as _);
        NonNull::new(previous_source_ptr).map(|raw| OwnedPcmSource::from_raw(raw))
    }

    /// Convenience function which returns the take's custom color (`I_CUSTOMCOLOR`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take.
    pub unsafe fn get_set_media_item_take_info_get_custom_color(
        &self,
        take: MediaItemTake,
    ) -> NativeColorValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr =
            self.get_set_media_item_take_info(take, TakeAttributeKey::CustomColor, null_mut());
        let raw = deref_as::<i32>(ptr).expect("I_CUSTOMCOLOR pointer is null");
        NativeColorValue::from_raw(raw)
    }

    /// Convenience function which sets the take's custom color (`I_CUSTOMCOLOR`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take.
    pub unsafe fn get_set_media_item_take_info_set_custom_color(
        &self,
        take: MediaItemTake,
        value: NativeColorValue,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_item_take_info(
            take,
            TakeAttributeKey::CustomColor,
            &value.to_raw() as *const _ as _,
        );
    }

    /// Convenience function which returns the take's pitch shift mode (`I_PITCHMODE`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take.
    pub unsafe fn get_set_media_item_take_info_get_pitch_mode(
        &self,
        take: MediaItemTake,
    ) -> Option<FullPitchShiftMode>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_take_info(take, TakeAttributeKey::PitchMode, null_mut());
        let raw = deref_as::<i32>(ptr).expect("I_PITCHMODE pointer is null");
        FullPitchShiftMode::from_raw(raw)
    }

    /// Convenience function which sets the take's pitch shift mode (`I_PITCHMODE`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take.
    pub unsafe fn get_set_media_item_take_info_set_pitch_mode(
        &self,
        take: MediaItemTake,
        value: Option<FullPitchShiftMode>,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = value.map(FullPitchShiftMode::to_raw).unwrap_or(-1);
        self.get_set_media_item_take_info(take, TakeAttributeKey::PitchMode, &raw as *const _ as _);
    }

    /// Convenience function which returns the take's pitch adjustment (`D_PITCH`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take.
    pub unsafe fn get_set_media_item_take_info_get_pitch(&self, take: MediaItemTake) -> Semitones
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_take_info(take, TakeAttributeKey::Pitch, null_mut());
        let raw = deref_as::<f64>(ptr).expect("I_PITCH pointer is null");
        Semitones::new_panic(raw)
    }

    /// Convenience function which sets the take's pitch adjustment (`D_PITCH`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take.
    pub unsafe fn get_set_media_item_take_info_set_pitch(
        &self,
        take: MediaItemTake,
        value: Semitones,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = value.get();
        self.get_set_media_item_take_info(take, TakeAttributeKey::Pitch, &raw as *const _ as _);
    }

    /// Convenience function which returns the given track's parent track (`P_PARTRACK`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_par_track(
        &self,
        track: MediaTrack,
    ) -> Option<MediaTrack>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::ParTrack, null_mut())
            as *mut raw::MediaTrack;
        MediaTrack::new(ptr)
    }

    /// Convenience function which returns the given track's parent project (`P_PROJECT`).
    ///
    /// In REAPER < 5.95 this returns `None`.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_project(
        &self,
        track: MediaTrack,
    ) -> Option<ReaProject>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::Project, null_mut())
            as *mut raw::ReaProject;
        ReaProject::new(ptr)
    }

    /// Convenience function which grants temporary access to the given track's name (`P_NAME`).
    ///
    /// Returns `None` if the given track is the master track.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use reaper_medium::ProjectContext::CurrentProject;
    /// let session = reaper_medium::ReaperSession::default();
    ///
    /// let track = session.reaper().get_track(CurrentProject, 0).ok_or("no track")?;
    /// let track_name = unsafe {
    ///     session.reaper().get_set_media_track_info_get_name(
    ///         track,
    ///         |name| name.to_owned()
    ///     )
    /// };
    /// let track_name = match &track_name {
    ///     None => "Master track",
    ///     Some(name) => name.to_str()
    /// };
    /// session.reaper().show_console_msg(format!("Track name is {}", track_name));
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_name<R>(
        &self,
        track: MediaTrack,
        use_name: impl FnOnce(&ReaperStr) -> R,
    ) -> Option<R>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::Name, null_mut());
        create_passing_c_str(ptr as *const c_char).map(use_name)
    }

    /// Convenience function which sets the track's name (`P_NAME`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use reaper_medium::ProjectContext::CurrentProject;
    /// let session = reaper_medium::ReaperSession::default();
    ///
    /// let track = session.reaper().get_track(CurrentProject, 0).ok_or("no track")?;
    /// unsafe {
    ///     session.reaper().get_set_media_track_info_set_name(track, "Guitar");
    /// }
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_set_name<'a>(
        &self,
        track: MediaTrack,
        name: impl Into<ReaperStringArg<'a>>,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_track_info(track, TrackAttributeKey::Name, name.into().as_ptr() as _);
    }

    /// Convenience function which returns the item's beat attach mode (`C_BEATATTACHMODE`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_beat_attach_mode(
        &self,
        item: MediaItem,
    ) -> Option<BeatAttachMode>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::BeatAttachMode, null_mut());
        let raw = deref_as::<i8>(ptr).expect("C_BEATATTACHMODE pointer is null");
        match raw {
            -1 => None,
            x => Some(BeatAttachMode::from_raw(x)),
        }
    }

    /// Convenience function which returns the track's beat attach mode (`C_BEATATTACHMODE`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_beat_attach_mode(
        &self,
        track: MediaTrack,
    ) -> Option<BeatAttachMode>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr =
            self.get_set_media_track_info(track, TrackAttributeKey::BeatAttachMode, null_mut());
        let raw = deref_as::<i8>(ptr).expect("C_BEATATTACHMODE pointer is null");
        match raw {
            -1 => None,
            x => Some(BeatAttachMode::from_raw(x)),
        }
    }

    /// Convenience function which sets the item's beat attach mode (`C_BEATATTACHMODE`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_beat_attach_mode(
        &self,
        item: MediaItem,
        mode: Option<BeatAttachMode>,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = mode.map(BeatAttachMode::to_raw).unwrap_or(-1i8);
        self.get_set_media_item_info(
            item,
            ItemAttributeKey::BeatAttachMode,
            &raw as *const _ as _,
        );
    }

    /// Convenience function which returns the item's volume (`D_VOL`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_vol(&self, item: MediaItem) -> ReaperVolumeValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::Vol, null_mut());
        let raw = deref_as::<f64>(ptr).expect("D_VOL pointer is null");
        ReaperVolumeValue::new_panic(raw)
    }

    /// Convenience function which sets the item's volume (`D_VOL`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_vol(&self, item: MediaItem, volume: ReaperVolumeValue)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_item_info(item, ItemAttributeKey::Vol, &volume.get() as *const _ as _);
    }

    /// Convenience function which returns the item's snap offset (`D_SNAPOFFSET`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_snap_offset(
        &self,
        item: MediaItem,
    ) -> DurationInSeconds
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::SnapOffset, null_mut());
        let raw = deref_as::<f64>(ptr).expect("D_VOL pointer is null");
        DurationInSeconds::new_panic(raw)
    }

    /// Convenience function which sets the item's snap offset (`D_SNAPOFFSET`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_snap_offset(
        &self,
        item: MediaItem,
        value: DurationInSeconds,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_item_info(
            item,
            ItemAttributeKey::SnapOffset,
            &value.get() as *const _ as _,
        );
    }

    /// Convenience function which returns the item's fade-in length (`D_FADEINLEN`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_fade_in_len(
        &self,
        item: MediaItem,
    ) -> DurationInSeconds
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::FadeInLen, null_mut());
        let raw = deref_as::<f64>(ptr).expect("D_FADEINLEN pointer is null");
        DurationInSeconds::new_panic(raw)
    }

    /// Convenience function which sets the item's fade-in length (`D_FADEINLEN`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_fade_in_len(
        &self,
        item: MediaItem,
        value: DurationInSeconds,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_item_info(
            item,
            ItemAttributeKey::FadeInLen,
            &value.get() as *const _ as _,
        );
    }

    /// Convenience function which returns the item's fade-out length (`D_FADEOUTLEN`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_fade_out_len(
        &self,
        item: MediaItem,
    ) -> DurationInSeconds
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::FadeOutLen, null_mut());
        let raw = deref_as::<f64>(ptr).expect("D_FADEOUTLEN pointer is null");
        DurationInSeconds::new_panic(raw)
    }

    /// Convenience function which sets the item's fade-out length (`D_FADEOUTLEN`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_fade_out_len(
        &self,
        item: MediaItem,
        value: DurationInSeconds,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_item_info(
            item,
            ItemAttributeKey::FadeOutLen,
            &value.get() as *const _ as _,
        );
    }

    /// Convenience function which returns the item's auto fade-in length (`D_FADEINLEN_AUTO`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_fade_in_len_auto(
        &self,
        item: MediaItem,
    ) -> Option<DurationInSeconds>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::FadeInLenAuto, null_mut());
        let raw = deref_as::<f64>(ptr).expect("D_FADEINLEN_AUTO pointer is null");
        raw.try_into().ok()
    }

    /// Convenience function which sets the item's auto fade-in length (`D_FADEINLEN_AUTO`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_fade_in_len_auto(
        &self,
        item: MediaItem,
        value: Option<DurationInSeconds>,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = value.map(|v| v.get()).unwrap_or(-1.0);
        self.get_set_media_item_info(item, ItemAttributeKey::FadeInLenAuto, &raw as *const _ as _);
    }

    /// Convenience function which returns the item's auto fade-out length (`D_FADEOUTLEN_AUTO`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_fade_out_len_auto(
        &self,
        item: MediaItem,
    ) -> Option<DurationInSeconds>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::FadeOutLenAuto, null_mut());
        let raw = deref_as::<f64>(ptr).expect("D_FADEOUTLEN_AUTO pointer is null");
        raw.try_into().ok()
    }

    /// Convenience function which sets the item's auto fade-out length (`D_FADEOUTLEN_AUTO`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_fade_out_len_auto(
        &self,
        item: MediaItem,
        value: Option<DurationInSeconds>,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = value.map(|v| v.get()).unwrap_or(-1.0);
        self.get_set_media_item_info(
            item,
            ItemAttributeKey::FadeOutLenAuto,
            &raw as *const _ as _,
        );
    }

    /// Convenience function which returns the item's fade-in shape (`C_FADEINSHAPE`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_fade_in_shape(&self, item: MediaItem) -> FadeShape
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::FadeInShape, null_mut());
        let raw = deref_as::<i32>(ptr).expect("C_FADEINSHAPE pointer is null");
        FadeShape::from_raw(raw)
    }

    /// Convenience function which sets the item's fade-in shape (`C_FADEINSHAPE`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_fade_in_shape(
        &self,
        item: MediaItem,
        value: FadeShape,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_item_info(
            item,
            ItemAttributeKey::FadeInShape,
            &value.to_raw() as *const _ as _,
        );
    }

    /// Convenience function which returns the item's fade-out shape (`C_FADEOUTSHAPE`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_fade_out_shape(&self, item: MediaItem) -> FadeShape
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::FadeOutShape, null_mut());
        let raw = deref_as::<i32>(ptr).expect("C_FADEOUTSHAPE pointer is null");
        FadeShape::from_raw(raw)
    }

    /// Convenience function which sets the item's fade-out shape (`C_FADEOUTSHAPE`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_fade_out_shape(
        &self,
        item: MediaItem,
        value: FadeShape,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_item_info(
            item,
            ItemAttributeKey::FadeOutShape,
            &value.to_raw() as *const _ as _,
        );
    }

    /// Convenience function which returns the item's auto fade-in curvature (`D_FADEINDIR`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_fade_in_dir(&self, item: MediaItem) -> FadeCurvature
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::FadeInDir, null_mut());
        let raw = deref_as::<f64>(ptr).expect("D_FADEINDIR pointer is null");
        FadeCurvature::new(raw)
    }

    /// Convenience function which sets the item's fade-in curvature (`D_FADEINDIR`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_fade_in_dir(
        &self,
        item: MediaItem,
        value: FadeCurvature,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_item_info(
            item,
            ItemAttributeKey::FadeInDir,
            &value.get() as *const _ as _,
        );
    }

    /// Convenience function which returns the item's auto fade-out curvature (`D_FADEOUTDIR`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_fade_out_dir(&self, item: MediaItem) -> FadeCurvature
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::FadeOutDir, null_mut());
        let raw = deref_as::<f64>(ptr).expect("D_FADEOUTDIR pointer is null");
        FadeCurvature::new(raw)
    }

    /// Convenience function which sets the item's fade-out curvature (`D_FADEOUTDIR`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_fade_out_dir(
        &self,
        item: MediaItem,
        value: FadeCurvature,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_item_info(
            item,
            ItemAttributeKey::FadeOutDir,
            &value.get() as *const _ as _,
        );
    }

    /// Convenience function which returns the item's group ID (`I_GROUPID`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_group_id(
        &self,
        item: MediaItem,
    ) -> Option<ItemGroupId>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::GroupId, null_mut());
        let raw = deref_as::<i32>(ptr).expect("I_GROUPID pointer is null");
        ItemGroupId::new(raw)
    }

    /// Convenience function which sets the item's group ID (`D_GROUPID`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_group_id(
        &self,
        item: MediaItem,
        value: Option<ItemGroupId>,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = if let Some(v) = value { v.get() } else { 0 };
        self.get_set_media_item_info(item, ItemAttributeKey::GroupId, &raw as *const _ as _);
    }

    /// Convenience function which returns the track's custom color (`I_CUSTOMCOLOR`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_track_info_get_custom_color(
        &self,
        track: MediaTrack,
    ) -> NativeColorValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::CustomColor, null_mut());
        let raw = deref_as::<i32>(ptr).expect("I_CUSTOMCOLOR pointer is null");
        NativeColorValue::from_raw(raw)
    }

    /// Convenience function which sets the track's custom color (`I_CUSTOMCOLOR`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_track_info_set_custom_color(
        &self,
        track: MediaTrack,
        value: NativeColorValue,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_track_info(
            track,
            TrackAttributeKey::CustomColor,
            &value.to_raw() as *const _ as _,
        );
    }

    /// Convenience function which returns the item's custom color (`I_CUSTOMCOLOR`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_get_custom_color(
        &self,
        item: MediaItem,
    ) -> NativeColorValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_item_info(item, ItemAttributeKey::CustomColor, null_mut());
        let raw = deref_as::<i32>(ptr).expect("I_CUSTOMCOLOR pointer is null");
        NativeColorValue::from_raw(raw)
    }

    /// Convenience function which sets the item's custom color (`I_CUSTOMCOLOR`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_set_media_item_info_set_custom_color(
        &self,
        item: MediaItem,
        value: NativeColorValue,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_item_info(
            item,
            ItemAttributeKey::CustomColor,
            &value.to_raw() as *const _ as _,
        );
    }

    /// Convenience function which sets the take's name (`P_NAME`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take.
    pub unsafe fn get_set_media_item_take_info_set_name<'a>(
        &self,
        take: MediaItemTake,
        name: impl Into<ReaperStringArg<'a>>,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_item_take_info(take, TakeAttributeKey::Name, name.into().as_ptr() as _);
    }

    /// Convenience function which grants temporary access to extension-specific data associated
    /// with the given track (`P_EXT:xyz`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_ext<'a, R>(
        &self,
        track: MediaTrack,
        data_id: impl Into<ReaperStringArg<'a>>,
        use_data: impl FnOnce(&ReaperStr) -> R,
    ) -> Option<R>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(
            track,
            TrackAttributeKey::Ext(data_id.into().into_inner()),
            null_mut(),
        );
        create_passing_c_str(ptr as *const c_char).map(use_data)
    }

    /// Convenience function which sets extension-specific data associated with a track
    /// (`P_EXT:xyz`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_set_ext<'a>(
        &self,
        track: MediaTrack,
        data_id: impl Into<ReaperStringArg<'a>>,
        data: impl Into<ReaperStringArg<'a>>,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_track_info(
            track,
            TrackAttributeKey::Ext(data_id.into().into_inner()),
            data.into().as_ptr() as _,
        );
    }

    /// Sets a project info string attribute.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_set_project_info_string_set<'a>(
        &self,
        project: ProjectContext,
        attribute_key: ProjectInfoAttributeKey,
        value: impl Into<ReaperStringArg<'a>>,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.get_set_project_info_string_set_unchecked(project, attribute_key, value) }
    }

    /// Like [`get_set_project_info_string_set()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_set_project_info_string_set()`]: #method.get_set_project_info_string_set
    pub unsafe fn get_set_project_info_string_set_unchecked<'a>(
        &self,
        project: ProjectContext,
        attribute_key: ProjectInfoAttributeKey,
        value: impl Into<ReaperStringArg<'a>>,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.GetSetProjectInfo_String(
            project.to_raw(),
            attribute_key.into_raw().as_ptr(),
            value.into().as_ptr() as _,
            true,
        );
        if !successful {
            return Err(ReaperFunctionError::new("couldn't set project info string"));
        }
        Ok(())
    }

    /// Convenience function which returns the given track's input monitoring mode (`I_RECMON`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_rec_mon(
        &self,
        track: MediaTrack,
    ) -> InputMonitoringMode
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::RecMon, null_mut());
        let irecmon = deref_as::<i32>(ptr).expect("I_RECMON pointer is null");
        InputMonitoringMode::from_raw(irecmon)
    }

    /// Convenience function which returns the given track's solo mode (`I_SOLO`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_solo(&self, track: MediaTrack) -> SoloMode
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::Solo, null_mut());
        let isolo = deref_as::<i32>(ptr).expect("I_SOLO pointer is null");
        SoloMode::from_raw(isolo)
    }

    /// Convenience function which sets the track's solo state (`I_SOLO`).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_set_solo(&self, track: MediaTrack, mode: SoloMode)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let value = mode.to_raw();
        self.get_set_media_track_info(track, TrackAttributeKey::Solo, &value as *const _ as _);
    }

    /// Convenience function which sets whether the track is shown in the mixer (`B_SHOWINMIXER`).
    ///
    /// Do not use on master track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_set_show_in_mixer(&self, track: MediaTrack, show: bool)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_track_info(
            track,
            TrackAttributeKey::ShowInMixer,
            &show as *const _ as _,
        );
    }

    /// Convenience function which sets whether the track is shown in the arrange view (`B_SHOWINTCP`).
    ///
    /// Do not use on master track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_set_show_in_tcp(&self, track: MediaTrack, show: bool)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_track_info(track, TrackAttributeKey::ShowInTcp, &show as *const _ as _);
    }

    /// Convenience function which returns the given track's pan mode (I_PANMODE).
    ///
    /// Returns `None` if the track uses the project default.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_pan_mode(&self, track: MediaTrack) -> Option<PanMode>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::PanMode, null_mut());
        let ipanmode = deref_as::<i32>(ptr).expect("I_PANMODE pointer is null");
        if ipanmode == -1 {
            return None;
        }
        Some(PanMode::from_raw(ipanmode))
    }

    /// Convenience function which returns the given track's pan (D_PAN).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_pan(&self, track: MediaTrack) -> ReaperPanValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::Pan, null_mut());
        let pan = deref_as::<f64>(ptr).expect("I_PAN pointer is null");
        ReaperPanValue::new_panic(pan)
    }

    /// Convenience function which returns the given track's dual-pan position 1 (D_DUALPANL).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_dual_pan_l(
        &self,
        track: MediaTrack,
    ) -> ReaperPanValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::DualPanL, null_mut());
        let pan = deref_as::<f64>(ptr).expect("D_DUALPANL pointer is null");
        ReaperPanValue::new_panic(pan)
    }

    /// Convenience function which returns the given track's dual-pan position 2 (D_DUALPANR).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_dual_pan_r(
        &self,
        track: MediaTrack,
    ) -> ReaperPanValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::DualPanR, null_mut());
        let pan = deref_as::<f64>(ptr).expect("D_DUALPANR pointer is null");
        ReaperPanValue::new_panic(pan)
    }

    /// Convenience function which returns the given track's width (D_WIDTH).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_width(&self, track: MediaTrack) -> ReaperWidthValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::Width, null_mut());
        let width = deref_as::<f64>(ptr).expect("I_WIDTH pointer is null");
        ReaperWidthValue::new(width)
    }

    /// Convenience function which returns the given track's recording input (I_RECINPUT).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_rec_input(
        &self,
        track: MediaTrack,
    ) -> Option<RecordingInput>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::RecInput, null_mut());
        let rec_input_index = deref_as::<i32>(ptr).expect("rec_input_index pointer is null");
        RecordingInput::from_raw(rec_input_index)
    }

    /// Convenience function which returns the given track's recording mode (I_RECMODE).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_rec_mode(&self, track: MediaTrack) -> RecordingMode
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::RecMode, null_mut());
        let rec_mode_index = deref_as::<i32>(ptr).expect("rec_mode_index pointer is null");
        RecordingMode::from_raw(rec_mode_index)
    }

    /// Convenience function which returns the type and location of the given track
    /// (IP_TRACKNUMBER).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_track_number(
        &self,
        track: MediaTrack,
    ) -> Option<TrackLocation>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        use TrackLocation::*;
        match self.get_set_media_track_info(track, TrackAttributeKey::TrackNumber, null_mut())
            as i32
        {
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
    pub unsafe fn get_set_media_track_info_set_guid(&self, track: MediaTrack, guid: &GUID)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.get_set_media_track_info(track, TrackAttributeKey::Guid, guid as *const _ as *mut _);
    }

    /// Convenience function which sets the given track's GUID (GUID).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_set_media_track_info_get_guid(&self, track: MediaTrack) -> GUID
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_media_track_info(track, TrackAttributeKey::Guid, null_mut());
        deref_as::<GUID>(ptr).expect("GUID pointer is null")
    }

    /// Returns whether we are in the real-time audio thread.
    ///
    /// *Real-time* means somewhere between [`OnAudioBuffer`] calls, not in some worker or
    /// anticipative FX thread.
    ///
    /// [`OnAudioBuffer`]: trait.OnAudioBuffer.html#method.call
    pub fn is_in_real_time_audio(&self) -> bool
    where
        UsageScope: AnyThread,
    {
        self.low.IsInRealTimeAudio() != 0
    }

    /// Returns whether audio is running at all.
    pub fn audio_is_running(&self) -> bool
    where
        UsageScope: AnyThread,
    {
        self.low.Audio_IsRunning() != 0
    }

    /// Starts playing.
    pub fn csurf_on_play(&self)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CSurf_OnPlay();
    }

    /// Stops playing.
    pub fn csurf_on_stop(&self)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CSurf_OnStop();
    }

    /// Pauses playing.
    pub fn csurf_on_pause(&self)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CSurf_OnPause();
    }

    /// Starts recording.
    pub fn csurf_on_record(&self)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CSurf_OnRecord();
    }

    /// Informs control surfaces that the repeat mode has changed.
    ///
    /// Doesn't actually change the repeat mode.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid control surface.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::{NotificationBehavior::NotifyAll, ProjectContext::CurrentProject};
    ///
    /// let track = session.reaper().get_track(CurrentProject, 0).ok_or("no tracks")?;
    /// unsafe {
    ///     session.reaper().csurf_set_repeat_state(true, NotifyAll);
    /// }
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub unsafe fn csurf_set_repeat_state(
        &self,
        repeat_state: bool,
        notification_behavior: NotificationBehavior,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .CSurf_SetRepeatState(repeat_state, notification_behavior.to_raw());
    }

    /// Returns `true` if any track in the given project is soloed.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn any_track_solo(&self, project: ProjectContext) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.any_track_solo_unchecked(project) }
    }

    /// Like [`any_track_solo()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`any_track_solo()`]: #method.any_track_solo
    pub unsafe fn any_track_solo_unchecked(&self, project: ProjectContext) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.AnyTrackSolo(project.to_raw())
    }

    /// Directly simulates a play button hit.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn on_play_button_ex(&self, project: ProjectContext)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.on_play_button_ex_unchecked(project) }
    }

    /// Like [`on_play_button_ex()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`on_play_button_ex()`]: #method.on_play_button_ex
    pub unsafe fn on_play_button_ex_unchecked(&self, project: ProjectContext)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.OnPlayButtonEx(project.to_raw());
    }

    /// Directly simulates a stop button hit.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn on_stop_button_ex(&self, project: ProjectContext)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.on_stop_button_ex_unchecked(project) }
    }

    /// Like [`on_stop_button_ex()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`on_stop_button_ex()`]: #method.on_stop_button_ex
    pub unsafe fn on_stop_button_ex_unchecked(&self, project: ProjectContext)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.OnStopButtonEx(project.to_raw());
    }

    /// Directly simulates a pause button hit.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn on_pause_button_ex(&self, project: ProjectContext)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.on_pause_button_ex_unchecked(project) }
    }

    /// Like [`on_pause_button_ex()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`on_pause_button_ex()`]: #method.on_pause_button_ex
    pub unsafe fn on_pause_button_ex_unchecked(&self, project: ProjectContext)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.OnPauseButtonEx(project.to_raw());
    }

    /// Queries the current play state.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_play_state_ex(&self, project: ProjectContext) -> PlayState
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.get_play_state_ex_unchecked(project) }
    }

    /// Like [`get_play_state_ex()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_play_state_ex()`]: #method.get_play_state_ex
    pub unsafe fn get_play_state_ex_unchecked(&self, project: ProjectContext) -> PlayState
    where
        UsageScope: AnyThread,
    {
        let result = self.low.GetPlayStateEx(project.to_raw()) as u32;
        PlayState {
            is_playing: result & 1 > 0,
            is_paused: result & 2 > 0,
            is_recording: result & 4 > 0,
        }
    }

    /// Queries the current repeat state.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_set_repeat_ex_get(&self, project: ProjectContext) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.get_set_repeat_ex_get_unchecked(project) }
    }

    /// Like [`get_set_repeat_ex_get()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_set_repeat_ex_get()`]: #method.get_set_repeat_ex_get
    pub unsafe fn get_set_repeat_ex_get_unchecked(&self, project: ProjectContext) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.GetSetRepeatEx(project.to_raw(), -1) > 0
    }

    /// Sets the repeat state.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_set_repeat_ex_set(&self, project: ProjectContext, repeat: bool)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe {
            self.get_set_repeat_ex_set_unchecked(project, repeat);
        }
    }

    /// Like [`get_set_repeat_ex_set()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_set_repeat_ex_set()`]: #method.get_set_repeat_ex_set
    pub unsafe fn get_set_repeat_ex_set_unchecked(&self, project: ProjectContext, repeat: bool)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.GetSetRepeatEx(project.to_raw(), i32::from(repeat));
    }

    /// Grants temporary access to the data of the given marker/region.
    ///
    /// The given index starts as 0 and counts both markers and regions.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn enum_project_markers_3<R>(
        &self,
        project: ProjectContext,
        index: u32,
        // TODO-high Other functions should take an option, too! Otherwise we can't give back
        // ownership  in case this didn't return anything! Same for all other continuation
        // passing functions!
        use_result: impl FnOnce(Option<EnumProjectMarkers3Result>) -> R,
    ) -> R
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.enum_project_markers_3_unchecked(project, index, use_result) }
    }

    /// Like [`enum_project_markers_3()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`enum_project_markers_3()`]: #method.enum_project_markers_3
    pub unsafe fn enum_project_markers_3_unchecked<R>(
        &self,
        project: ProjectContext,
        index: u32,
        use_result: impl FnOnce(Option<EnumProjectMarkers3Result>) -> R,
    ) -> R
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut is_region = MaybeUninit::zeroed();
        let mut pos = MaybeUninit::zeroed();
        let mut region_end = MaybeUninit::zeroed();
        let mut name = MaybeUninit::zeroed();
        let mut id = MaybeUninit::zeroed();
        let mut color = MaybeUninit::zeroed();
        let successful = self.low.EnumProjectMarkers3(
            project.to_raw(),
            index as _,
            is_region.as_mut_ptr(),
            pos.as_mut_ptr(),
            region_end.as_mut_ptr(),
            name.as_mut_ptr(),
            id.as_mut_ptr(),
            color.as_mut_ptr(),
        );
        if successful == 0 {
            return use_result(None);
        }
        let result = EnumProjectMarkers3Result {
            position: PositionInSeconds::new_panic(pos.assume_init()),
            region_end_position: if is_region.assume_init() {
                Some(PositionInSeconds::new_panic(region_end.assume_init()))
            } else {
                None
            },
            name: create_passing_c_str(name.assume_init()).unwrap(),
            id: BookmarkId(id.assume_init() as _),
            color: NativeColor(color.assume_init() as _),
        };
        use_result(Some(result))
    }

    /// Creates a PCM source from the given file name.
    ///
    /// # Errors
    ///
    /// Returns an error if the PCM source could not be created.
    ///
    /// # Panics
    ///
    /// Panics if the given file name is not valid UTF-8.
    pub fn pcm_source_create_from_file_ex(
        &self,
        file_name: &Utf8Path,
        midi_import_behavior: MidiImportBehavior,
    ) -> ReaperFunctionResult<OwnedPcmSource>
    where
        UsageScope: AnyThread,
    {
        let file_name_reaper_string = convert_path_to_reaper_string(file_name);
        let ptr = unsafe {
            self.low.PCM_Source_CreateFromFileEx(
                file_name_reaper_string.as_ptr(),
                match midi_import_behavior {
                    MidiImportBehavior::UsePreference => false,
                    MidiImportBehavior::ForceNoMidiImport => true,
                },
            )
        };
        NonNull::new(ptr)
            .ok_or_else(|| ReaperFunctionError::new("couldn't create PCM source from file"))
            .map(OwnedPcmSource)
    }

    /// Creates a PCM source from a type identifier.
    ///
    /// TODO-high Documentation/API ... unstable
    ///
    /// Use this if you're going to load its state via LoadState/ProjectStateContext.
    /// Valid types include "WAVE", "MIDI", or whatever plug-ins define as well.
    ///
    /// # Errors
    ///
    /// Returns an error if the PCM source could not be created.
    pub fn pcm_source_create_from_type<'a>(
        &self,
        source_type: impl Into<ReaperStringArg<'a>>,
    ) -> ReaperFunctionResult<OwnedPcmSource>
    where
        UsageScope: MainThreadOnly,
    {
        // TODO-medium Can maybe be relaxed.
        self.require_main_thread();
        let ptr = unsafe {
            self.low
                .PCM_Source_CreateFromType(source_type.into().as_ptr())
        };
        NonNull::new(ptr)
            .ok_or_else(|| ReaperFunctionError::new("couldn't create PCM source from type"))
            .map(OwnedPcmSource)
    }

    /// Goes to the given marker.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn go_to_marker(&self, project: ProjectContext, marker: BookmarkRef)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe {
            self.go_to_marker_unchecked(project, marker);
        }
    }

    /// Like [`go_to_marker()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// [`go_to_marker()`]: #method.go_to_marker
    pub unsafe fn go_to_marker_unchecked(&self, project: ProjectContext, marker: BookmarkRef)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.GoToMarker(
            project.to_raw(),
            marker.to_raw(),
            marker.uses_timeline_order(),
        );
    }

    /// Seeks to the given region after the current one finishes playing (smooth seek).
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn go_to_region(&self, project: ProjectContext, region: BookmarkRef)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe {
            self.go_to_region_unchecked(project, region);
        }
    }

    /// Like [`go_to_region()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`go_to_region()`]: #method.go_to_region
    pub unsafe fn go_to_region_unchecked(&self, project: ProjectContext, region: BookmarkRef)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.GoToRegion(
            project.to_raw(),
            region.to_raw(),
            region.uses_timeline_order(),
        );
    }

    /// Converts the given time into beats.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn time_map_2_time_to_beats(
        &self,
        project: ProjectContext,
        tpos: PositionInSeconds,
    ) -> TimeMap2TimeToBeatsResult
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.time_map_2_time_to_beats_unchecked(project, tpos) }
    }

    /// Like [`time_map_2_time_to_beats()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`time_map_2_time_to_beats()`]: #method.time_map_2_time_to_beats
    pub unsafe fn time_map_2_time_to_beats_unchecked(
        &self,
        project: ProjectContext,
        tpos: PositionInSeconds,
    ) -> TimeMap2TimeToBeatsResult
    where
        UsageScope: AnyThread,
    {
        let mut measures = MaybeUninit::zeroed();
        let mut measure_length = MaybeUninit::zeroed();
        let mut full_beats = MaybeUninit::zeroed();
        let mut common_denom = MaybeUninit::zeroed();
        let beats_within_measure = self.low.TimeMap2_timeToBeats(
            project.to_raw(),
            tpos.get(),
            measures.as_mut_ptr(),
            measure_length.as_mut_ptr(),
            full_beats.as_mut_ptr(),
            common_denom.as_mut_ptr(),
        );
        TimeMap2TimeToBeatsResult {
            full_beats: PositionInBeats::new_panic(full_beats.assume_init()),
            measure_index: measures.assume_init(),
            beats_since_measure: PositionInBeats::new_panic(beats_within_measure),
            time_signature: TimeSignature {
                numerator: NonZeroU32::new(measure_length.assume_init() as _).unwrap(),
                denominator: NonZeroU32::new(common_denom.assume_init() as _).unwrap(),
            },
        }
    }

    /// Returns information about the given measure.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn time_map_get_measure_info(
        &self,
        project: ProjectContext,
        measure_index: i32,
    ) -> TimeMapGetMeasureInfoResult
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.time_map_get_measure_info_unchecked(project, measure_index) }
    }

    /// Like [`time_map_get_measure_info()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`time_map_get_measure_info()`]: #method.time_map_get_measure_info
    pub unsafe fn time_map_get_measure_info_unchecked(
        &self,
        project: ProjectContext,
        measure_index: i32,
    ) -> TimeMapGetMeasureInfoResult
    where
        UsageScope: AnyThread,
    {
        let mut start_qn = MaybeUninit::zeroed();
        let mut end_qn = MaybeUninit::zeroed();
        let mut num = MaybeUninit::zeroed();
        let mut denom = MaybeUninit::zeroed();
        let mut bpm = MaybeUninit::zeroed();
        let start_time = self.low.TimeMap_GetMeasureInfo(
            project.to_raw(),
            measure_index,
            start_qn.as_mut_ptr(),
            end_qn.as_mut_ptr(),
            num.as_mut_ptr(),
            denom.as_mut_ptr(),
            bpm.as_mut_ptr(),
        );
        TimeMapGetMeasureInfoResult {
            start_time: PositionInSeconds::new_panic(start_time),
            start_qn: PositionInQuarterNotes::new_panic(start_qn.assume_init()),
            end_qn: PositionInQuarterNotes::new_panic(end_qn.assume_init()),
            time_signature: TimeSignature {
                numerator: NonZeroU32::new(num.assume_init() as _).unwrap(),
                denominator: NonZeroU32::new(denom.assume_init() as _).unwrap(),
            },
            tempo: Bpm::new_panic(bpm.assume_init()),
        }
    }

    /// Converts the given beat position to time, optionally starting from a specific measure.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn time_map_2_beats_to_time(
        &self,
        project: ProjectContext,
        measure_mode: MeasureMode,
        bpos: PositionInBeats,
    ) -> PositionInSeconds
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.time_map_2_beats_to_time_unchecked(project, measure_mode, bpos) }
    }

    /// Like [`time_map_2_beats_to_time()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`time_map_2_beats_to_time()`]: #method.time_map_2_beats_to_time
    pub unsafe fn time_map_2_beats_to_time_unchecked(
        &self,
        project: ProjectContext,
        measure_mode: MeasureMode,
        bpos: PositionInBeats,
    ) -> PositionInSeconds
    where
        UsageScope: AnyThread,
    {
        use MeasureMode::*;
        let tpos = self.low.TimeMap2_beatsToTime(
            project.to_raw(),
            bpos.get(),
            match measure_mode {
                IgnoreMeasure => null(),
                FromMeasureAtIndex(i) => &i as *const _,
            },
        );
        PositionInSeconds::new_panic(tpos)
    }

    /// Converts the given quarter-note position to a measure index and returns the measure bounds
    /// in quarter notes.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn time_map_qn_to_measures(
        &self,
        project: ProjectContext,
        qn: PositionInQuarterNotes,
    ) -> TimeMapQnToMeasuresResult
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.time_map_qn_to_measures_unchecked(project, qn) }
    }

    /// Like [`time_map_qn_to_measures()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`time_map_qn_to_measures()`]: #method.time_map_qn_to_measures
    pub unsafe fn time_map_qn_to_measures_unchecked(
        &self,
        project: ProjectContext,
        qn: PositionInQuarterNotes,
    ) -> TimeMapQnToMeasuresResult
    where
        UsageScope: AnyThread,
    {
        let mut start_qn = MaybeUninit::zeroed();
        let mut end_qn = MaybeUninit::zeroed();
        let measure = self.low.TimeMap_QNToMeasures(
            project.to_raw(),
            qn.get(),
            start_qn.as_mut_ptr(),
            end_qn.as_mut_ptr(),
        );
        TimeMapQnToMeasuresResult {
            measure_index: measure,
            start: PositionInQuarterNotes::new_panic(start_qn.assume_init()),
            end: PositionInQuarterNotes::new_panic(end_qn.assume_init()),
        }
    }

    /// Converts the given quarter-note position to time.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn time_map_2_qn_to_time(
        &self,
        project: ProjectContext,
        qn: PositionInQuarterNotes,
    ) -> PositionInSeconds
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.time_map_2_qn_to_time_unchecked(project, qn) }
    }

    /// Like [`time_map_2_qn_to_time()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`time_map_2_qn_to_time()`]: #method.time_map_2_qn_to_time
    pub unsafe fn time_map_2_qn_to_time_unchecked(
        &self,
        project: ProjectContext,
        qn: PositionInQuarterNotes,
    ) -> PositionInSeconds
    where
        UsageScope: AnyThread,
    {
        let tpos = self.low.TimeMap2_QNToTime(project.to_raw(), qn.get());
        PositionInSeconds::new_panic(tpos)
    }

    /// Converts the given time to a quarter-note position.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn time_map_2_time_to_qn(
        &self,
        project: ProjectContext,
        tpos: PositionInSeconds,
    ) -> PositionInQuarterNotes
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.time_map_2_time_to_qn_unchecked(project, tpos) }
    }

    /// Like [`time_map_2_time_to_qn()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`time_map_2_time_to_qn()`]: #method.time_map_2_time_to_qn
    pub unsafe fn time_map_2_time_to_qn_unchecked(
        &self,
        project: ProjectContext,
        tpos: PositionInSeconds,
    ) -> PositionInQuarterNotes
    where
        UsageScope: AnyThread,
    {
        let qn = self.low.TimeMap2_timeToQN(project.to_raw(), tpos.get());
        PositionInQuarterNotes::new_panic(qn)
    }

    /// Converts the given quarter-note position to time.
    ///
    /// Quarter notes are counted from the start of the project, regardless of any partial measures.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn time_map_2_qn_to_time_abs(
        &self,
        project: ProjectContext,
        qn: PositionInQuarterNotes,
    ) -> PositionInSeconds
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.time_map_2_qn_to_time_unchecked(project, qn) }
    }

    /// Like [`time_map_2_qn_to_time_abs()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`time_map_2_qn_to_time_abs()`]: #method.time_map_2_qn_to_time_abs
    pub unsafe fn time_map_2_qn_to_time_abs_unchecked(
        &self,
        project: ProjectContext,
        qn: PositionInQuarterNotes,
    ) -> PositionInSeconds
    where
        UsageScope: AnyThread,
    {
        let tpos = self.low.TimeMap2_QNToTime(project.to_raw(), qn.get());
        PositionInSeconds::new_panic(tpos)
    }

    /// Converts the given time to a quarter-note position.
    ///
    /// Quarter notes are counted from the start of the project, regardless of any partial measures.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn time_map_2_time_to_qn_abs(
        &self,
        project: ProjectContext,
        tpos: PositionInSeconds,
    ) -> PositionInQuarterNotes
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.time_map_2_time_to_qn_abs_unchecked(project, tpos) }
    }

    /// Like [`time_map_2_time_to_qn_abs()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`time_map_2_time_to_qn_abs()`]: #method.time_map_2_time_to_qn_abs
    pub unsafe fn time_map_2_time_to_qn_abs_unchecked(
        &self,
        project: ProjectContext,
        tpos: PositionInSeconds,
    ) -> PositionInQuarterNotes
    where
        UsageScope: AnyThread,
    {
        let qn = self.low.TimeMap2_timeToQN(project.to_raw(), tpos.get());
        PositionInQuarterNotes::new_panic(qn)
    }

    /// Gets the arrange view start/end time for the given screen coordinates.
    ///
    /// Set both `screen_x_start` and `screen_x_end` to 0 to get the full arrange view's start/end
    /// time.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_set_arrange_view_2_get(
        &self,
        project: ProjectContext,
        screen_x_start: u32,
        screen_x_end: u32,
    ) -> GetSetArrangeView2Result
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.get_set_arrange_view_2_get_unchecked(project, screen_x_start, screen_x_end) }
    }

    /// Like [`get_set_arrange_view_2_get()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_set_arrange_view_2_get()`]: #method.get_set_arrange_view_2_get
    pub unsafe fn get_set_arrange_view_2_get_unchecked(
        &self,
        project: ProjectContext,
        screen_x_start: u32,
        screen_x_end: u32,
    ) -> GetSetArrangeView2Result
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut start_time = MaybeUninit::zeroed();
        let mut end_time = MaybeUninit::zeroed();
        self.low.GetSet_ArrangeView2(
            project.to_raw(),
            false,
            screen_x_start as _,
            screen_x_end as _,
            start_time.as_mut_ptr(),
            end_time.as_mut_ptr(),
        );
        GetSetArrangeView2Result {
            start_time: PositionInSeconds::new_panic(start_time.assume_init()),
            end_time: PositionInSeconds::new_panic(end_time.assume_init()),
        }
    }

    /// Returns the effective tempo in BPM at the given position (i.e. 2x in /8 signatures).
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn time_map_2_get_divided_bpm_at_time(
        &self,
        project: ProjectContext,
        tpos: PositionInSeconds,
    ) -> Bpm
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.time_map_2_get_divided_bpm_at_time_unchecked(project, tpos) }
    }

    /// Like [`time_map_2_get_divided_bpm_at_time()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`time_map_2_get_divided_bpm_at_time()`]: #method.time_map_2_get_divided_bpm_at_time
    pub unsafe fn time_map_2_get_divided_bpm_at_time_unchecked(
        &self,
        project: ProjectContext,
        tpos: PositionInSeconds,
    ) -> Bpm
    where
        UsageScope: AnyThread,
    {
        let bpm = self
            .low
            .TimeMap2_GetDividedBpmAtTime(project.to_raw(), tpos.get());
        Bpm::new_panic(bpm)
    }

    /// Returns the current position of the edit cursor.
    ///
    /// # Errors
    ///
    /// Errors if REAPER returns an invalid position (INF, -INF or NaN). This doesn't usually happen, but I observed
    /// it once.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_cursor_position_ex(
        &self,
        project: ProjectContext,
    ) -> ReaperFunctionResult<PositionInSeconds>
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.get_cursor_position_ex_unchecked(project) }
    }

    /// Like [`get_cursor_position_ex()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_cursor_position_ex()`]: #method.get_cursor_position_ex
    pub unsafe fn get_cursor_position_ex_unchecked(
        &self,
        project: ProjectContext,
    ) -> ReaperFunctionResult<PositionInSeconds>
    where
        UsageScope: AnyThread,
    {
        let res = self.low.GetCursorPositionEx(project.to_raw());
        PositionInSeconds::new(res).map_err(|_| "returned cursor position was invalid".into())
    }

    /// Returns the latency-compensated actual-what-you-hear position.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_play_position_ex(&self, project: ProjectContext) -> PositionInSeconds
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.get_play_position_ex_unchecked(project) }
    }

    /// Like [`get_play_position_ex()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_play_position_ex()`]: #method.get_play_position_ex
    pub unsafe fn get_play_position_ex_unchecked(
        &self,
        project: ProjectContext,
    ) -> PositionInSeconds
    where
        UsageScope: AnyThread,
    {
        let res = self.low.GetPlayPositionEx(project.to_raw());
        PositionInSeconds::new_panic(res)
    }

    /// Returns the position of the next audio block being processed.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_play_position_2_ex(&self, project: ProjectContext) -> PositionInSeconds
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.get_play_position_2_ex_unchecked(project) }
    }

    /// Like [`get_play_position_2_ex()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_play_position_2_ex()`]: #method.get_play_position_2_ex
    pub unsafe fn get_play_position_2_ex_unchecked(
        &self,
        project: ProjectContext,
    ) -> PositionInSeconds
    where
        UsageScope: AnyThread,
    {
        let res = self.low.GetPlayPosition2Ex(project.to_raw());
        PositionInSeconds::new_panic(res)
    }

    /// Returns the number of markers and regions in the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn count_project_markers(&self, project: ProjectContext) -> CountProjectMarkersResult
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.count_project_markers_unchecked(project) }
    }

    /// Like [`count_project_markers()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`count_project_markers()`]: #method.count_project_markers
    pub unsafe fn count_project_markers_unchecked(
        &self,
        project: ProjectContext,
    ) -> CountProjectMarkersResult
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut num_markers = MaybeUninit::zeroed();
        let mut num_regions = MaybeUninit::zeroed();
        let total_count = self.low.CountProjectMarkers(
            project.to_raw(),
            num_markers.as_mut_ptr(),
            num_regions.as_mut_ptr(),
        );
        CountProjectMarkersResult {
            total_count: total_count as _,
            marker_count: num_markers.assume_init() as _,
            region_count: num_regions.assume_init() as _,
        }
    }

    /// Gets the last project marker before the given time and/or the project region that includes
    /// the given time.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_last_marker_and_cur_region(
        &self,
        project: ProjectContext,
        time: PositionInSeconds,
    ) -> GetLastMarkerAndCurRegionResult
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.get_last_marker_and_cur_region_unchecked(project, time) }
    }

    /// Like [`get_last_marker_and_cur_region()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_last_marker_and_cur_region()`]: #method.get_last_marker_and_cur_region
    pub unsafe fn get_last_marker_and_cur_region_unchecked(
        &self,
        project: ProjectContext,
        time: PositionInSeconds,
    ) -> GetLastMarkerAndCurRegionResult
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut marker_idx = MaybeUninit::zeroed();
        let mut region_idx = MaybeUninit::zeroed();
        self.low.GetLastMarkerAndCurRegion(
            project.to_raw(),
            time.get(),
            marker_idx.as_mut_ptr(),
            region_idx.as_mut_ptr(),
        );
        GetLastMarkerAndCurRegionResult {
            marker_index: make_some_if_not_negative(marker_idx.assume_init()),
            region_index: make_some_if_not_negative(region_idx.assume_init()),
        }
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
    pub fn main_on_command_ex(&self, command: CommandId, flag: i32, project: ProjectContext)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.main_on_command_ex_unchecked(command, flag, project) }
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
        command_id: CommandId,
        flag: i32,
        project: ProjectContext,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .Main_OnCommandEx(command_id.to_raw(), flag, project.to_raw());
    }

    /// Sends an action command to the last focused MIDI editor.
    ///
    /// # Errors
    ///
    /// Returns an error if no MIDI editor is open or if the view mode does not match the input.
    pub fn midi_editor_last_focused_on_command(
        &self,
        command_id: CommandId,
        required_view_mode: RequiredViewMode,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.MIDIEditor_LastFocused_OnCommand(
            command_id.to_raw(),
            required_view_mode == RequiredViewMode::ListView,
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "no MIDI editor with this view mode open",
            ));
        }
        Ok(())
    }

    /// Sends an action command to the given MIDI editor.
    ///
    /// # Errors
    ///
    /// Returns an error if the supplied MIDI editor pointer is not valid (not an open MIDI editor).
    pub fn midi_editor_on_command(
        &self,
        midi_editor: Hwnd,
        command_id: CommandId,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = unsafe {
            self.low
                .MIDIEditor_OnCommand(midi_editor.as_ptr(), command_id.to_raw())
        };
        if !successful {
            return Err(ReaperFunctionError::new(
                "given window is not an open MIDI editor",
            ));
        }
        Ok(())
    }

    /// Informs control surfaces that the given track's mute state has changed.
    ///
    /// Doesn't actually change the mute state.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track or an invalid control surface.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::{NotificationBehavior::NotifyAll, ProjectContext::CurrentProject};
    ///
    /// let track = session.reaper().get_track(CurrentProject, 0).ok_or("no tracks")?;
    /// unsafe {
    ///     session.reaper().csurf_set_surface_mute(track, true, NotifyAll);
    /// }
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub unsafe fn csurf_set_surface_mute(
        &self,
        track: MediaTrack,
        mute: bool,
        notification_behavior: NotificationBehavior,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .CSurf_SetSurfaceMute(track.as_ptr(), mute, notification_behavior.to_raw());
    }

    /// Informs control surfaces that the given track's solo state has changed.
    ///
    /// Doesn't actually change the solo state.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track or an invalid control surface.
    pub unsafe fn csurf_set_surface_solo(
        &self,
        track: MediaTrack,
        solo: bool,
        notification_behavior: NotificationBehavior,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .CSurf_SetSurfaceSolo(track.as_ptr(), solo, notification_behavior.to_raw());
    }

    /// Generates a random GUID.
    pub fn gen_guid(&self) -> GUID
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        // We zero this just for being safe
        let mut guid = MaybeUninit::zeroed();
        unsafe {
            self.low.genGuid(guid.as_mut_ptr());
        }
        unsafe { guid.assume_init() }
    }

    /// Grants temporary access to the section with the given ID.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::SectionId;
    ///
    /// let action_count =
    ///     session.reaper().section_from_unique_id(SectionId::new(1), |s| s.action_list_cnt());
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    //
    // In order to not need unsafe, we take the closure. For normal medium-level API usage, this is
    // the safe way to go.
    pub fn section_from_unique_id<R>(
        &self,
        section_id: SectionId,
        use_section: impl FnOnce(&KbdSectionInfo) -> R,
    ) -> Option<R>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.SectionFromUniqueID(section_id.to_raw());
        if ptr.is_null() {
            return None;
        }
        NonNull::new(ptr).map(|nnp| use_section(&KbdSectionInfo(nnp)))
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
        section_id: SectionId,
    ) -> Option<KbdSectionInfo>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.SectionFromUniqueID(section_id.to_raw());
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
        command_id: CommandId,
        value_change: ActionValueChange,
        window: WindowContext,
        project: ProjectContext,
    ) -> i32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let (val, valhw, relmode) = value_change.to_raw();
        self.low.KBD_OnMainActionEx(
            command_id.to_raw(),
            val,
            valhw,
            relmode,
            window.to_raw(),
            project.to_raw(),
        )
    }

    /// Opens an action picker window for prompting the user to select an action.
    pub fn prompt_for_action_create(&self, initial: InitialAction, section_id: SectionId)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .PromptForAction(1, initial.to_raw(), section_id.to_raw());
    }

    /// Polls an action picker session which has been previously created via
    /// [`prompt_for_action_create()`].
    ///
    /// [`prompt_for_action_create()`]: #method.prompt_for_action_create
    pub fn prompt_for_action_poll(&self, section_id: SectionId) -> PromptForActionResult
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let result = self.low.PromptForAction(0, 0, section_id.to_raw());
        PromptForActionResult::from_raw(result)
    }

    /// Finishes an action picker session which has been previously created via
    /// [`prompt_for_action_create()`].
    ///
    /// [`prompt_for_action_create()`]: #method.prompt_for_action_create
    pub fn prompt_for_action_finish(&self, section_id: SectionId)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.PromptForAction(-1, 0, section_id.to_raw());
    }

    /// Returns the REAPER main window handle.
    pub fn get_main_hwnd(&self) -> Hwnd
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        require_hwnd_panic(self.low.GetMainHwnd())
    }

    /// Returns the focused MIDI editor window.
    pub fn midi_editor_get_active(&self) -> Option<Hwnd>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        Hwnd::new(self.low.MIDIEditor_GetActive())
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
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw_id = unsafe { self.low.NamedCommandLookup(command_name.into().as_ptr()) as u32 };
        if raw_id == 0 {
            return None;
        }
        Some(CommandId(raw_id))
    }

    /// Returns a project configuration variable descriptor to be used with
    /// [`project_config_var_addr`]
    ///
    /// [`project_config_var_addr`]: #method.project_config_var_addr
    pub fn project_config_var_get_offs<'a>(
        &self,
        name: impl Into<ReaperStringArg<'a>>,
    ) -> Option<ProjectConfigVarGetOffsResult>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut size = MaybeUninit::zeroed();
        let offset = unsafe {
            self.low
                .projectconfig_var_getoffs(name.into().as_ptr(), size.as_mut_ptr())
        };
        if offset < 0 {
            return None;
        }
        let result = ProjectConfigVarGetOffsResult {
            offset: offset as _,
            size: unsafe { size.assume_init() } as _,
        };
        Some(result)
    }

    /// Returns the project configuration object at the given address.
    pub fn project_config_var_addr(
        &self,
        project: ProjectContext,
        index: u32,
    ) -> Option<NonNull<c_void>>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.project_config_var_addr_unchecked(project, index) }
    }

    /// Like [`project_config_var_addr()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`project_config_var_addr()`]: #method.project_config_var_addr
    pub unsafe fn project_config_var_addr_unchecked(
        &self,
        project: ProjectContext,
        index: u32,
    ) -> Option<NonNull<c_void>>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self
            .low
            .projectconfig_var_addr(project.to_raw(), index as _);
        NonNull::new(ptr)
    }

    /// Opens a file picker.
    ///
    /// Returns `None` if the user canceled the dialog.
    pub fn get_user_file_name_for_read<'a>(
        &self,
        path: &Utf8Path,
        title: impl Into<ReaperStringArg<'a>>,
        defext: impl Into<ReaperStringArg<'a>>,
    ) -> Option<Utf8PathBuf>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let (file, successful) =
            with_string_buffer_prefilled(path.to_string(), 4096, |buffer, _| unsafe {
                self.low.GetUserFileNameForRead(
                    buffer,
                    title.into().as_ptr(),
                    defext.into().as_ptr(),
                )
            });
        if !successful {
            return None;
        }
        Some(Utf8PathBuf::from(file.into_string()))
    }

    /// Grants temporary access to the "reaper.ini" full filename.
    pub fn get_ini_file<R>(&self, use_ini_file: impl FnOnce(&Utf8Path) -> R) -> R
    where
        UsageScope: AnyThread,
    {
        // TODO-high I think we should either insist on the path being UTF-8 and return an Utf8Path (separate crate)
        //  or not interpret the path as UTF-8 and return Path. At the moment, it's something inbetween.
        let ptr = self.low.get_ini_file();
        let reaper_str =
            unsafe { create_passing_c_str(ptr).expect("should always return ini path") };
        let path = Utf8Path::new(reaper_str.to_str());
        use_ini_file(path)
    }

    /// Returns the REAPER preference with the given name.
    pub fn get_config_var<'a>(
        &self,
        name: impl Into<ReaperStringArg<'a>>,
    ) -> Option<GetConfigVarResult>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut size = MaybeUninit::zeroed();
        let ptr = unsafe {
            self.low
                .get_config_var(name.into().as_ptr(), size.as_mut_ptr())
        };
        let res = GetConfigVarResult {
            size: unsafe { size.assume_init() as u32 },
            value: NonNull::new(ptr)?,
        };
        Some(res)
    }

    /// Clears the ReaScript console.
    pub fn clear_console(&self)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.ClearConsole();
    }

    /// Returns the number of tracks in the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn count_tracks(&self, project: ProjectContext) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.count_tracks_unchecked(project) }
    }

    /// Like [`count_tracks()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`count_tracks()`]: #method.count_tracks
    pub unsafe fn count_tracks_unchecked(&self, project: ProjectContext) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CountTracks(project.to_raw()) as u32
    }

    /// Returns an integer that changes when the project state changes.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_project_state_change_count(&self, project: ProjectContext) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.get_project_state_change_count_unchecked(project) }
    }

    /// Like [`get_project_state_change_count()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_project_state_change_count()`]: #method.get_project_state_change_count
    pub unsafe fn get_project_state_change_count_unchecked(&self, project: ProjectContext) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.GetProjectStateChangeCount(project.to_raw()) as u32
    }

    /// Returns the number of items in the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn count_media_items(&self, project: ProjectContext) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.count_media_items_unchecked(project) }
    }

    /// Like [`count_media_items()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`count_media_items()`]: #method.count_media_items
    pub unsafe fn count_media_items_unchecked(&self, project: ProjectContext) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CountMediaItems(project.to_raw()) as u32
    }

    /// Returns the length of the given project.
    ///
    /// The length is the maximum of end of media item, markers, end of regions and tempo map.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_project_length(&self, project: ProjectContext) -> DurationInSeconds
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.get_project_length_unchecked(project) }
    }

    /// Like [`get_project_length()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_project_length()`]: #method.get_project_length
    pub unsafe fn get_project_length_unchecked(&self, project: ProjectContext) -> DurationInSeconds
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let res = self.low.GetProjectLength(project.to_raw());
        DurationInSeconds::new_panic(res)
    }

    /// Sets the position of the edit cursor and optionally moves the view and/or seeks.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn set_edit_curs_pos_2(
        &self,
        project: ProjectContext,
        time: PositionInSeconds,
        options: SetEditCurPosOptions,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe {
            self.set_edit_curs_pos_2_unchecked(project, time, options);
        }
    }

    /// Like [`set_edit_curs_pos_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`set_edit_curs_pos_2()`]: #method.set_edit_curs_pos_2
    pub unsafe fn set_edit_curs_pos_2_unchecked(
        &self,
        project: ProjectContext,
        time: PositionInSeconds,
        options: SetEditCurPosOptions,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.SetEditCurPos2(
            project.to_raw(),
            time.get(),
            options.move_view,
            options.seek_play,
        );
    }

    /// Returns the loop point or time selection time range that's currently set in the given
    /// project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_set_loop_time_range_2_get(
        &self,
        project: ProjectContext,
        time_range_type: TimeRangeType,
    ) -> Option<GetLoopTimeRange2Result>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.get_set_loop_time_range_2_get_unchecked(project, time_range_type) }
    }

    /// Like [`get_set_loop_time_range_2_get()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_set_loop_time_range_2_get()`]: #method.get_set_loop_time_range_2_get
    pub unsafe fn get_set_loop_time_range_2_get_unchecked(
        &self,
        project: ProjectContext,
        time_range_type: TimeRangeType,
    ) -> Option<GetLoopTimeRange2Result>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut start = MaybeUninit::zeroed();
        let mut end = MaybeUninit::zeroed();
        use TimeRangeType::*;
        self.low.GetSet_LoopTimeRange2(
            project.to_raw(),
            false,
            match time_range_type {
                LoopPoints => true,
                TimeSelection => false,
            },
            start.as_mut_ptr(),
            end.as_mut_ptr(),
            false,
        );
        let (start, end) = (start.assume_init(), end.assume_init());
        if start == 0.0 && end == 0.0 {
            return None;
        }
        let res = GetLoopTimeRange2Result {
            start: PositionInSeconds::new_panic(start),
            end: PositionInSeconds::new_panic(end),
        };
        Some(res)
    }

    /// Sets the loop point or time selection time range for the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_set_loop_time_range_2_set(
        &self,
        project: ProjectContext,
        time_range_type: TimeRangeType,
        start: PositionInSeconds,
        end: PositionInSeconds,
        auto_seek_behavior: AutoSeekBehavior,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe {
            self.get_set_loop_time_range_2_set_unchecked(
                project,
                time_range_type,
                start,
                end,
                auto_seek_behavior,
            );
        }
    }

    /// Like [`get_set_loop_time_range_2_set()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_set_loop_time_range_2_set()`]: #method.get_set_loop_time_range_2_set
    pub unsafe fn get_set_loop_time_range_2_set_unchecked(
        &self,
        project: ProjectContext,
        time_range_type: TimeRangeType,
        start: PositionInSeconds,
        end: PositionInSeconds,
        auto_seek_behavior: AutoSeekBehavior,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        use AutoSeekBehavior::*;
        use TimeRangeType::*;
        self.low.GetSet_LoopTimeRange2(
            project.to_raw(),
            true,
            match time_range_type {
                LoopPoints => true,
                TimeSelection => false,
            },
            &mut start.get() as _,
            &mut end.get() as _,
            match auto_seek_behavior {
                DenyAutoSeek => false,
                AllowAutoSeek => true,
            },
        );
    }

    /// Creates a new track at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn insert_track_in_project(
        &self,
        project: ProjectContext,
        index: u32,
        defaults_behavior: TrackDefaultsBehavior,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe {
            self.insert_track_in_project_unchecked(project, index, defaults_behavior);
        }
    }

    /// Like [`insert_track_in_project_unchecked()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`insert_track_in_project_unchecked()`]: #method.insert_track_in_project_unchecked
    pub unsafe fn insert_track_in_project_unchecked(
        &self,
        project: ProjectContext,
        index: u32,
        defaults_behavior: TrackDefaultsBehavior,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.InsertTrackInProject(
            project.to_raw(),
            index as i32,
            (defaults_behavior == TrackDefaultsBehavior::AddDefaultEnvAndFx).into(),
        );
    }

    /// Creates a new track at the given index.
    pub fn insert_track_at_index(&self, index: u32, defaults_behavior: TrackDefaultsBehavior)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.InsertTrackAtIndex(
            index as i32,
            defaults_behavior == TrackDefaultsBehavior::AddDefaultEnvAndFx,
        );
    }

    /// Moves all selected tracks to the given index.
    ///
    /// # Errors
    ///
    /// Returns an error if no tracks were selected.
    pub fn reorder_selected_tracks(
        &self,
        index: u32,
        behavior: ReorderTracksBehavior,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self
            .low
            .ReorderSelectedTracks(index as i32, behavior.to_raw());
        if !successful {
            return Err(ReaperFunctionError::new("no track selected"));
        }
        Ok(())
    }

    /// Resets all MIDI devices.
    pub fn midi_reinit(&self)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.midi_reinit();
    }

    /// Returns the maximum number of MIDI input devices (usually 63).
    pub fn get_max_midi_inputs(&self) -> u32
    where
        UsageScope: AnyThread,
    {
        self.low.GetMaxMidiInputs() as u32
    }

    /// Returns the maximum number of MIDI output devices (usually 64).
    pub fn get_max_midi_outputs(&self) -> u32
    where
        UsageScope: AnyThread,
    {
        self.low.GetMaxMidiOutputs() as u32
    }

    /// Returns information about the given MIDI input device.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the device name you want.
    /// If you are not interested in the device name at all, pass 0.
    pub fn get_midi_input_name(
        &self,
        device_id: MidiInputDeviceId,
        buffer_size: u32,
    ) -> GetMidiDevNameResult
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        if buffer_size == 0 {
            let is_present =
                unsafe { self.low.GetMIDIInputName(device_id.to_raw(), null_mut(), 0) };
            GetMidiDevNameResult {
                is_present,
                name: None,
            }
        } else {
            let (name, is_present) =
                with_string_buffer_cstring(buffer_size, |buffer, max_size| unsafe {
                    self.low
                        .GetMIDIInputName(device_id.to_raw(), buffer, max_size)
                });
            if name.is_empty() {
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
    /// With `buffer_size` you can tell REAPER how many bytes of the device name you want.
    /// If you are not interested in the device name at all, pass 0.
    pub fn get_midi_output_name(
        &self,
        device_id: MidiOutputDeviceId,
        buffer_size: u32,
    ) -> GetMidiDevNameResult
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        if buffer_size == 0 {
            let is_present = unsafe {
                self.low
                    .GetMIDIOutputName(device_id.to_raw(), null_mut(), 0)
            };
            GetMidiDevNameResult {
                is_present,
                name: None,
            }
        } else {
            let (name, is_present) =
                with_string_buffer_cstring(buffer_size, |buffer, max_size| unsafe {
                    self.low
                        .GetMIDIOutputName(device_id.to_raw(), buffer, max_size)
                });
            if name.is_empty() {
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

    /// Returns a new pitch shift API instance.
    ///
    /// Version must be [raw::REAPER_PITCHSHIFT_API_VER].
    pub fn reaper_get_pitch_shift_api(&self, version: i32) -> Option<OwnedReaperPitchShift>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.ReaperGetPitchShiftAPI(version);
        NonNull::new(raw).map(|ptr| unsafe { OwnedReaperPitchShift::from_raw(ptr) })
    }

    /// Returns information about the given pitch shift mode.
    ///
    /// Start querying modes at 0. Returns `None` when no more modes possible.
    pub fn enum_pitch_shift_modes(
        &self,
        mode: PitchShiftMode,
    ) -> Option<EnumPitchShiftModesResult<'static>>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut name = MaybeUninit::zeroed();
        let exists = unsafe {
            self.low
                .EnumPitchShiftModes(mode.to_raw(), name.as_mut_ptr())
        };
        if !exists {
            return None;
        }
        let name = unsafe { name.assume_init() };
        let res = if name.is_null() {
            EnumPitchShiftModesResult::Unsupported
        } else {
            EnumPitchShiftModesResult::Supported {
                name: unsafe { create_passing_c_str(name).unwrap() },
            }
        };
        Some(res)
    }

    /// Grants temporary access to the name of the given pitch shift sub mode.
    ///
    /// Start querying modes at 0. Returns `None` when no more sub modes possible.
    pub fn enum_pitch_shift_sub_modes<R>(
        &self,
        mode: PitchShiftMode,
        sub_mode: PitchShiftSubMode,
        use_name: impl FnOnce(Option<&ReaperStr>) -> R,
    ) -> R
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let name = self
            .low
            .EnumPitchShiftSubModes(mode.to_raw(), sub_mode.to_raw());
        if name.is_null() {
            return use_name(None);
        }
        let name = unsafe { create_passing_c_str(name).unwrap() };
        use_name(Some(name))
    }

    /// Returns a new resample instance.
    pub fn resampler_create(&self) -> OwnedReaperResample
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.Resampler_Create();
        let ptr = NonNull::new(raw).expect("REAPER didn't return a resample instance");
        unsafe { OwnedReaperResample::from_raw(ptr) }
    }

    /// Returns the name of the given resample mode.
    ///
    /// Start querying modes at 0. Returns `None` when no more sub modes possible.
    pub fn resample_enum_modes(&self, mode: ResampleMode) -> Option<&'static ReaperStr>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let name = self.low.Resample_EnumModes(mode.to_raw());
        if name.is_null() {
            return None;
        }
        let name = unsafe { create_passing_c_str(name).unwrap() };
        Some(name)
    }

    // Return type Option or Result can't be easily chosen here because if instantiate is 0, it
    // should be Option, if it's -1 or > 0, it should be Result. So we just keep the i32. That's
    // also one reason why we just publish the convenience functions.
    unsafe fn track_fx_add_by_name<'a>(
        &self,
        track: MediaTrack,
        fx_name: impl Into<ReaperStringArg<'a>>,
        fx_chain_type: TrackFxChainType,
        behavior: FxAddByNameBehavior,
    ) -> i32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.TrackFX_AddByName(
            track.as_ptr(),
            fx_name.into().as_ptr(),
            fx_chain_type == TrackFxChainType::InputFxChain,
            behavior.to_raw(),
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
        fx_name: impl Into<ReaperStringArg<'a>>,
        fx_chain_type: TrackFxChainType,
    ) -> Option<u32>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        match self.track_fx_add_by_name(track, fx_name, fx_chain_type, FxAddByNameBehavior::Query) {
            -1 => None,
            idx if idx >= 0 => Some(idx as u32),
            _ => unreachable!(),
        }
    }

    /// Returns the parameter index corresponding to the given identifier.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_param_from_ident(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        ident: ParamId,
    ) -> Option<u32>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        match self.low.TrackFX_GetParamFromIdent(
            track.as_ptr(),
            fx_location.to_raw(),
            ident.into_raw().as_ptr(),
        ) {
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
        fx_name: impl Into<ReaperStringArg<'a>>,
        fx_chain_type: TrackFxChainType,
        behavior: AddFxBehavior,
    ) -> ReaperFunctionResult<u32>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        match self.track_fx_add_by_name(track, fx_name, fx_chain_type, behavior.into()) {
            -1 => Err(ReaperFunctionError::new("FX couldn't be added")),
            idx if idx >= 0 => Ok(idx as u32),
            _ => unreachable!(),
        }
    }

    /// Returns whether the given track FX is enabled.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_enabled(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
    ) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .TrackFX_GetEnabled(track.as_ptr(), fx_location.to_raw())
    }

    /// Returns whether the given track FX is offline.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_offline(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
    ) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .TrackFX_GetOffline(track.as_ptr(), fx_location.to_raw())
    }

    /// Returns the name of the given FX.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the FX name you want.
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
        fx_location: TrackFxLocation,
        buffer_size: u32,
    ) -> ReaperFunctionResult<ReaperString>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        assert!(buffer_size > 0);
        let (name, successful) = with_string_buffer(buffer_size, |buffer, max_size| {
            self.low
                .TrackFX_GetFXName(track.as_ptr(), fx_location.to_raw(), buffer, max_size)
        });
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't get FX name (probably FX doesn't exist)",
            ));
        }
        Ok(name)
    }

    /// Returns the name of the given track send or hardware output send.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the send name you want.
    ///
    /// When choosing the send index, keep in mind that the hardware output sends (if any) come
    /// first.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
    ///
    /// # Errors
    ///
    /// Returns an error if the track send doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_send_name(
        &self,
        track: MediaTrack,
        send_index: u32,
        buffer_size: u32,
    ) -> ReaperFunctionResult<ReaperString>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        assert!(buffer_size > 0);
        let (name, successful) = with_string_buffer(buffer_size, |buffer, max_size| {
            self.low
                .GetTrackSendName(track.as_ptr(), send_index as i32, buffer, max_size)
        });
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't get send name (probably send doesn't exist)",
            ));
        }
        Ok(name)
    }

    /// Returns the name of the given track receive.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the receive name you want.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
    ///
    /// # Errors
    ///
    /// Returns an error if the track send doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_receive_name(
        &self,
        track: MediaTrack,
        receive_index: u32,
        buffer_size: u32,
    ) -> ReaperFunctionResult<ReaperString>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        assert!(buffer_size > 0);
        let (name, successful) = with_string_buffer(buffer_size, |buffer, max_size| {
            self.low
                .GetTrackReceiveName(track.as_ptr(), receive_index as i32, buffer, max_size)
        });
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't get receive name (probably receive doesn't exist)",
            ));
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
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
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
        fx_location: TrackFxLocation,
        enabled: bool,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .TrackFX_SetEnabled(track.as_ptr(), fx_location.to_raw(), enabled);
    }

    /// Sets the given track FX offline or online.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_set_offline(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        offline: bool,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .TrackFX_SetOffline(track.as_ptr(), fx_location.to_raw(), offline);
    }

    /// Returns the number of parameters of given track FX.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_num_params(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
    ) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .TrackFX_GetNumParams(track.as_ptr(), fx_location.to_raw()) as u32
    }

    /// Returns the audio device input/output latency in samples.
    pub fn get_input_output_latency(&self) -> GetInputOutputLatencyResult
    where
        UsageScope: AnyThread,
    {
        let mut input_latency = MaybeUninit::uninit();
        let mut output_latency = MaybeUninit::uninit();
        unsafe {
            self.low
                .GetInputOutputLatency(input_latency.as_mut_ptr(), output_latency.as_mut_ptr())
        };
        GetInputOutputLatencyResult {
            input_latency: unsafe { input_latency.assume_init() } as u32,
            output_latency: unsafe { output_latency.assume_init() } as u32,
        }
    }

    /// Returns the current project if it's just being loaded or saved.
    ///
    /// This is usually only used from `project_config_extension_t`.
    // TODO-low `project_config_extension_t` is not yet ported
    pub fn get_current_project_in_load_save(&self) -> Option<ReaProject>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.GetCurrentProjectInLoadSave();
        ReaProject::new(ptr)
    }

    /// Returns the name of the given track FX parameter.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the parameter name you want.
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
        fx_location: TrackFxLocation,
        param_index: u32,
        buffer_size: u32,
    ) -> ReaperFunctionResult<ReaperString>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        assert!(buffer_size > 0);
        let (name, successful) = with_string_buffer(buffer_size, |buffer, max_size| {
            self.low.TrackFX_GetParamName(
                track.as_ptr(),
                fx_location.to_raw(),
                param_index as i32,
                buffer,
                max_size,
            )
        });
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't get FX parameter name (probably FX or parameter doesn't exist)",
            ));
        }
        Ok(name)
    }

    /// Returns the current value of the given track FX parameter formatted as string.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the parameter value string you
    /// want.
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
        fx_location: TrackFxLocation,
        param_index: u32,
        buffer_size: u32,
    ) -> ReaperFunctionResult<ReaperString>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        assert!(buffer_size > 0);
        let (name, successful) = with_string_buffer(buffer_size, |buffer, max_size| {
            self.low.TrackFX_GetFormattedParamValue(
                track.as_ptr(),
                fx_location.to_raw(),
                param_index as i32,
                buffer,
                max_size,
            )
        });
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't format current FX parameter value (probably FX or parameter doesn't exist)",
            ));
        }
        Ok(name)
    }

    /// Returns the given value formatted as string according to the given track FX parameter.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the parameter value string you
    /// want.
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
        fx_location: TrackFxLocation,
        param_index: u32,
        param_value: ReaperNormalizedFxParamValue,
        buffer_size: u32,
    ) -> ReaperFunctionResult<ReaperString>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        assert!(buffer_size > 0);
        let (name, successful) = with_string_buffer(buffer_size, |buffer, max_size| {
            self.low.TrackFX_FormatParamValueNormalized(
                track.as_ptr(),
                fx_location.to_raw(),
                param_index as i32,
                param_value.get(),
                buffer,
                max_size,
            )
        });
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't format FX parameter value (FX maybe doesn't support Cockos extensions or FX or parameter doesn't exist)",
            ));
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
    /// - REAPER can crash if you pass an invalid track.
    /// - Calling this from any other thread than the main thread causes undefined behavior!
    /// - However, there's one exception: Calling it in a real-time thread directly "from the track"
    ///   which is currently processing is okay, and only for REAPER >= v6.52+dev0323. Previous
    ///   REAPER versions will send control surface change notifications, in the wrong thread.
    ///   Newer versions don't send any notifications when this function is called in real-time.
    pub unsafe fn track_fx_set_param_normalized(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        param_index: u32,
        param_value: ReaperNormalizedFxParamValue,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: AnyThread,
    {
        let successful = self.low.TrackFX_SetParamNormalized(
            track.as_ptr(),
            fx_location.to_raw(),
            param_index as i32,
            param_value.get(),
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't set FX parameter value (probably FX or parameter doesn't exist)",
            ));
        }
        Ok(())
    }

    /// Notifies REAPER that we are done changing parameter values
    ///
    /// This is important for automation mode _Touch_.
    ///
    /// # Errors
    ///
    /// Returns an error if the FX or parameter doesn't exist.
    ///
    /// # Safety
    ///
    /// - REAPER can crash if you pass an invalid track.
    /// - Calling this from any other thread than the main thread causes undefined behavior!
    /// - However, there's one exception: Calling it in a real-time thread directly "from the track"
    ///   which is currently processing is okay, and only for REAPER >= v6.52+dev0323. Previous
    ///   REAPER versions will send control surface change notifications, in the wrong thread.
    ///   Newer versions don't send any notifications when this function is called in real-time.
    pub unsafe fn track_fx_end_param_edit(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        param_index: u32,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: AnyThread,
    {
        let successful =
            self.low
                .TrackFX_EndParamEdit(track.as_ptr(), fx_location.to_raw(), param_index as i32);
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't end FX parameter edit (probably FX or parameter doesn't exist)",
            ));
        }
        Ok(())
    }

    /// Returns information about the (last) focused FX window.
    ///
    /// Returns `Some` if an FX window has focus or was the last focused one and is still open.
    ///
    /// Returns `None` otherwise.
    #[deprecated = "use `get_focused_fx_2` instead"]
    pub fn get_focused_fx(&self) -> Option<GetFocusedFxResult>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
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
        self.get_focused_fx_internal(result, tracknumber, itemnumber, fxnumber)
    }

    /// Returns information about the focused FX window.
    ///
    /// Returns `Some` if an FX window has focus or was the last focused one and is still open.
    /// The wrapped value contains additional information about whether the window is still focused.
    ///
    /// Returns `None` otherwise.
    #[deprecated = "use `get_touched_or_focused_fx_currently_focused_fx` instead"]
    pub fn get_focused_fx_2(&self) -> Option<GetFocusedFx2Result>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut tracknumber = MaybeUninit::uninit();
        let mut itemnumber = MaybeUninit::uninit();
        let mut fxnumber = MaybeUninit::uninit();
        let result = unsafe {
            self.low.GetFocusedFX2(
                tracknumber.as_mut_ptr(),
                itemnumber.as_mut_ptr(),
                fxnumber.as_mut_ptr(),
            )
        };
        let fx = self.get_focused_fx_internal(result, tracknumber, itemnumber, fxnumber)?;
        let result = GetFocusedFx2Result {
            is_still_focused: result & 0b100 == 0,
            fx,
        };
        Some(result)
    }

    /// Returns the currently focused FX.
    pub fn get_touched_or_focused_fx_currently_focused_fx(
        &self,
    ) -> Option<GetTouchedOrFocusedFxCurrentlyFocusedFxResult>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut trackidx = MaybeUninit::uninit();
        let mut itemidx = MaybeUninit::uninit();
        let mut takeidx = MaybeUninit::uninit();
        let mut fxidx = MaybeUninit::uninit();
        let mut parm = MaybeUninit::uninit();
        let successful = unsafe {
            self.low.GetTouchedOrFocusedFX(
                1,
                trackidx.as_mut_ptr(),
                itemidx.as_mut_ptr(),
                takeidx.as_mut_ptr(),
                fxidx.as_mut_ptr(),
                parm.as_mut_ptr(),
            )
        };
        if !successful {
            return None;
        }
        let trackidx = unsafe { trackidx.assume_init() };
        let itemidx = unsafe { itemidx.assume_init() };
        let takeidx = unsafe { takeidx.assume_init() };
        let fxidx = unsafe { fxidx.assume_init() };
        let parm = unsafe { parm.assume_init() as u32 };
        let result = GetTouchedOrFocusedFxCurrentlyFocusedFxResult {
            is_still_focused: parm & 1 == 0,
            fx: match itemidx {
                -1 => FxLocation::TrackFx {
                    track_location: match trackidx {
                        -1 => TrackLocation::MasterTrack,
                        x if x >= 0 => TrackLocation::NormalTrack(x as u32),
                        _ => panic!("encountered negative track index"),
                    },
                    fx_location: TrackFxLocation::from_raw(fxidx),
                },
                x if x >= 0 => FxLocation::TakeFx {
                    track_index: if trackidx >= 0 {
                        trackidx as u32
                    } else {
                        panic!("encountered negative track index");
                    },
                    item_index: x as u32,
                    take_index: if takeidx >= 0 {
                        takeidx as u32
                    } else {
                        panic!("encountered negative take index");
                    },
                    fx_index: if fxidx >= 0 {
                        // TODO Support FX in containers
                        fxidx as u32
                    } else {
                        panic!("encountered negative FX index");
                    },
                },
                _ => panic!("encountered negative item index"),
            },
        };
        Some(result)
    }

    /// `result` can be either from `GetFocusedFx` or `GetFocusedFx2`. It only looks at the first
    /// two bits.
    fn get_focused_fx_internal(
        &self,
        result: i32,
        tracknumber: MaybeUninit<c_int>,
        itemnumber: MaybeUninit<c_int>,
        fxnumber: MaybeUninit<c_int>,
    ) -> Option<GetFocusedFxResult>
    where
        UsageScope: MainThreadOnly,
    {
        let kind = result & 0b11;
        let tracknumber = unsafe { tracknumber.assume_init() as u32 };
        let fxnumber = unsafe { fxnumber.assume_init() };
        use GetFocusedFxResult::*;
        match kind {
            0 => None,
            1 => Some(TrackFx {
                track_location: convert_tracknumber_to_track_location(tracknumber),
                fx_location: TrackFxLocation::from_raw(fxnumber),
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
            x => Some(Unknown(Hidden(x))),
        }
    }

    /// Returns information about the last touched FX parameter.
    ///
    /// Returns `Some` if an FX parameter has been touched already and that FX is still existing.
    /// Returns `None` otherwise.
    pub fn get_last_touched_fx(&self) -> Option<GetLastTouchedFxResult>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
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
                track_location: convert_tracknumber_to_track_location(tracknumber),
                fx_location: TrackFxLocation::from_raw(fxnumber),
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
        source: (MediaTrack, TrackFxLocation),
        destination: (MediaTrack, TrackFxLocation),
        transfer_behavior: TransferBehavior,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.TrackFX_CopyToTrack(
            source.0.as_ptr(),
            source.1.to_raw(),
            destination.0.as_ptr(),
            destination.1.to_raw(),
            transfer_behavior == TransferBehavior::Move,
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
        fx_location: TrackFxLocation,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let succesful = self
            .low
            .TrackFX_Delete(track.as_ptr(), fx_location.to_raw());
        if !succesful {
            return Err(ReaperFunctionError::new(
                "couldn't delete FX (probably FX doesn't exist)",
            ));
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
    /// - REAPER can crash if you pass an invalid track.
    /// - Calling this from any other thread than the main thread causes undefined behavior!
    /// - However, there's one exception: Calling it in a real-time thread directly "from the track"
    ///   which is currently processing should be okay.
    //
    // Option makes more sense than Result here because this function is at the same time the
    // correct function to be used to determine *if* a parameter reports step sizes. So
    // "parameter doesn't report step sizes" is a valid result.
    pub unsafe fn track_fx_get_parameter_step_sizes(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        param_index: u32,
    ) -> Option<GetParameterStepSizesResult>
    where
        UsageScope: AnyThread,
    {
        // It's important to zero these variables (could also do that without MaybeUninit) because
        // if REAPER returns true, that doesn't always mean that it initialized all of the variables
        // correctly. Learned this the hard way with some super random results coming up.
        let mut step = MaybeUninit::zeroed();
        let mut small_step = MaybeUninit::zeroed();
        let mut large_step = MaybeUninit::zeroed();
        let mut is_toggle = MaybeUninit::zeroed();
        let successful = self.low.TrackFX_GetParameterStepSizes(
            track.as_ptr(),
            fx_location.to_raw(),
            param_index as i32,
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
                normal_step: step.assume_init(),
                small_step: make_some_if_greater_than_zero(small_step.assume_init()),
                large_step: make_some_if_greater_than_zero(large_step.assume_init()),
            })
        }
    }

    /// Returns the current value and min/mid/max values of the given track FX.
    ///
    /// # Safety
    ///
    /// - REAPER can crash if you pass an invalid track.
    /// - Calling this from any other thread than the main thread causes undefined behavior!
    /// - However, there's one exception: Calling it in a real-time thread directly "from the track"
    ///   which is currently processing should be okay.
    pub unsafe fn track_fx_get_param_ex(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        param_index: u32,
    ) -> GetParamExResult
    where
        UsageScope: AnyThread,
    {
        let mut min_val = MaybeUninit::uninit();
        let mut max_val = MaybeUninit::uninit();
        let mut mid_val = MaybeUninit::uninit();
        let value = self.low.TrackFX_GetParamEx(
            track.as_ptr(),
            fx_location.to_raw(),
            param_index as i32,
            min_val.as_mut_ptr(),
            max_val.as_mut_ptr(),
            mid_val.as_mut_ptr(),
        );
        GetParamExResult {
            current_value: value,
            min_value: min_val.assume_init(),
            mid_value: mid_val.assume_init(),
            max_value: max_val.assume_init(),
        }
    }

    /// Gets a plug-in specific named configuration value.
    ///
    /// With `buffer_size` you can tell REAPER and the FX how many bytes of the value you want.
    ///
    /// Named parameters are a vendor-specific VST extension from Cockos (see
    /// <http://reaper.fm/sdk/vst/vst_ext.php>).
    ///
    /// # Errors
    ///
    /// Returns an error if the given FX doesn't have this named parameter or doesn't support named
    /// parameters.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_named_config_parm<'a>(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        param_name: impl Into<ReaperStringArg<'a>>,
        buffer_size: u32,
    ) -> ReaperFunctionResult<Vec<u8>>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let (buffer, successful) = with_buffer(buffer_size, |buffer, max_size| {
            self.low.TrackFX_GetNamedConfigParm(
                track.as_ptr(),
                fx_location.to_raw(),
                param_name.into().as_ptr(),
                buffer,
                max_size,
            )
        });
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't get named parameter value",
            ));
        }
        Ok(buffer)
    }

    /// Like [`track_fx_get_named_config_parm`](Self::track_fx_get_named_config_parm)
    /// but interpreting the result as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the given FX doesn't have this named parameter, doesn't support named
    /// parameters or if the returned data doesn't resemble a proper string.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_named_config_parm_as_string<'a>(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        param_name: impl Into<ReaperStringArg<'a>>,
        buffer_size: u32,
    ) -> ReaperFunctionResult<ReaperString>
    where
        UsageScope: MainThreadOnly,
    {
        let mut bytes =
            self.track_fx_get_named_config_parm(track, fx_location, param_name, buffer_size)?;
        if let Some(nul_byte_index) = bytes.iter().position(|b| *b == 0) {
            // Crop end of vector so that it doesn't include the nul terminator anymore.
            bytes.resize(nul_byte_index, 0);
            Ok(ReaperString::new(CString::from_vec_unchecked(bytes)))
        } else {
            Err(ReaperFunctionError::new("result is not a string"))
        }
    }

    /// Sets a plug-in specific named configuration value.
    ///
    /// Named parameters are a vendor-specific VST extension from Cockos (see
    /// <http://reaper.fm/sdk/vst/vst_ext.php>).
    ///
    /// # Errors
    ///
    /// Returns an error if the given FX doesn't have this named parameter or doesn't support named
    /// parameters.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track or value.
    pub unsafe fn track_fx_set_named_config_parm<'a>(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        param_name: impl Into<ReaperStringArg<'a>>,
        value: *const c_char,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.TrackFX_SetNamedConfigParm(
            track.as_ptr(),
            fx_location.to_raw(),
            param_name.into().as_ptr(),
            value,
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't set named parameter value",
            ));
        }
        Ok(())
    }

    /// Starts a new undo block.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::{ProjectContext::CurrentProject, UndoScope::Scoped, ProjectPart::*};
    ///
    /// session.reaper().undo_begin_block_2(CurrentProject);
    /// // ... modify something ...
    /// session.reaper().undo_end_block_2(CurrentProject, "Modify something", Scoped(Items | Fx));
    /// ```
    pub fn undo_begin_block_2(&self, project: ProjectContext)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.undo_begin_block_2_unchecked(project) };
    }

    /// Like [`undo_begin_block_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`undo_begin_block_2()`]: #method.undo_begin_block_2
    pub unsafe fn undo_begin_block_2_unchecked(&self, project: ProjectContext)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.Undo_BeginBlock2(project.to_raw());
    }

    /// Ends the current undo block.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn undo_end_block_2<'a>(
        &self,
        project: ProjectContext,
        description: impl Into<ReaperStringArg<'a>>,
        scope: UndoScope,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe {
            self.undo_end_block_2_unchecked(project, description, scope);
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
        project: ProjectContext,
        description: impl Into<ReaperStringArg<'a>>,
        scope: UndoScope,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.Undo_EndBlock2(
            project.to_raw(),
            description.into().as_ptr(),
            scope.to_raw(),
        );
    }

    /// Grants temporary access to the the description of the last undoable operation, if any.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn undo_can_undo_2<R>(
        &self,
        project: ProjectContext,
        use_description: impl FnOnce(&ReaperStr) -> R,
    ) -> Option<R>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.undo_can_undo_2_unchecked(project, use_description) }
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
        project: ProjectContext,
        use_description: impl FnOnce(&ReaperStr) -> R,
    ) -> Option<R>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.Undo_CanUndo2(project.to_raw());
        create_passing_c_str(ptr).map(use_description)
    }

    /// Grants temporary access to the description of the next redoable operation, if any.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn undo_can_redo_2<R>(
        &self,
        project: ProjectContext,
        use_description: impl FnOnce(&ReaperStr) -> R,
    ) -> Option<R>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.undo_can_redo_2_unchecked(project, use_description) }
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
        project: ProjectContext,
        use_description: impl FnOnce(&ReaperStr) -> R,
    ) -> Option<R>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.Undo_CanRedo2(project.to_raw());
        create_passing_c_str(ptr).map(use_description)
    }

    /// Makes the last undoable operation undone.
    ///
    /// Returns `false` if there was nothing to be undone.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn undo_do_undo_2(&self, project: ProjectContext) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.undo_do_undo_2_unchecked(project) }
    }

    /// Like [`undo_do_undo_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`undo_do_undo_2()`]: #method.undo_do_undo_2
    pub unsafe fn undo_do_undo_2_unchecked(&self, project: ProjectContext) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.Undo_DoUndo2(project.to_raw()) != 0
    }

    /// Executes the next redoable action.
    ///
    /// Returns `false` if there was nothing to be redone.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn undo_do_redo_2(&self, project: ProjectContext) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.undo_do_redo_2_unchecked(project) }
    }

    /// Like [`undo_do_redo_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`undo_do_redo_2()`]: #method.undo_do_redo_2
    pub unsafe fn undo_do_redo_2_unchecked(&self, project: ProjectContext) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.Undo_DoRedo2(project.to_raw()) != 0
    }

    /// Marks the given project as dirty.
    ///
    /// *Dirty* means the project needs to be saved. Only makes a difference if "Maximum undo
    /// memory" is not 0 in REAPER's preferences (0 disables undo/prompt to save).
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn mark_project_dirty(&self, project: ProjectContext)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe {
            self.mark_project_dirty_unchecked(project);
        }
    }

    /// Like [`mark_project_dirty()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`mark_project_dirty()`]: #method.mark_project_dirty
    pub unsafe fn mark_project_dirty_unchecked(&self, project: ProjectContext)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.MarkProjectDirty(project.to_raw());
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
    pub fn is_project_dirty(&self, project: ProjectContext) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.is_project_dirty_unchecked(project) }
    }

    /// Like [`is_project_dirty()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`is_project_dirty()`]: #method.is_project_dirty
    pub unsafe fn is_project_dirty_unchecked(&self, project: ProjectContext) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.IsProjectDirty(project.to_raw()) != 0
    }

    /// Notifies all control surfaces that something in the track list has changed.
    ///
    /// Behavior not confirmed.
    pub fn track_list_update_all_external_surfaces(&self)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.TrackList_UpdateAllExternalSurfaces();
    }

    /// Returns the version of the REAPER application in which this plug-in is currently running.
    pub fn get_app_version(&self) -> ReaperVersion<'static>
    where
        UsageScope: AnyThread,
    {
        let ptr = self.low.GetAppVersion();
        let version_str = unsafe { ReaperStr::from_ptr(ptr) };
        ReaperVersion::new(version_str)
    }

    /// Returns the track automation mode, regardless of the global override.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_automation_mode(&self, track: MediaTrack) -> AutomationMode
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let result = self.low.GetTrackAutomationMode(track.as_ptr());
        AutomationMode::from_raw(result)
    }

    /// Extracts an RGB color from the given OS-dependent color.
    pub fn color_from_native(&self, color: NativeColor) -> RgbColor
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let (mut r, mut g, mut b) = (
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
            MaybeUninit::uninit(),
        );
        unsafe {
            self.low.ColorFromNative(
                color.to_raw(),
                r.as_mut_ptr(),
                g.as_mut_ptr(),
                b.as_mut_ptr(),
            );
        }
        RgbColor {
            r: unsafe { r.assume_init() as _ },
            g: unsafe { g.assume_init() as _ },
            b: unsafe { b.assume_init() as _ },
        }
    }

    /// Makes an OS-dependent color from an RGB color.
    pub fn color_to_native(&self, color: RgbColor) -> NativeColor
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let native_color = self
            .low
            .ColorToNative(color.r as _, color.g as _, color.b as _);
        NativeColor(native_color)
    }

    /// Runs the system color chooser dialog.
    ///
    /// Returns `None` if the user cancels the dialog.
    pub fn gr_select_color(
        &self,
        window: WindowContext,
        current_color: NativeColor,
    ) -> Option<NativeColor>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut raw = current_color.to_raw();
        let picked = unsafe { self.low.GR_SelectColor(window.to_raw(), &mut raw) };
        if picked == 0 {
            return None;
        }
        Some(NativeColor::new(raw))
    }

    /// Sets the track automation mode.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_automation_mode(
        &self,
        track: MediaTrack,
        automation_mode: AutomationMode,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .SetTrackAutomationMode(track.as_ptr(), automation_mode.to_raw());
    }

    /// Returns the global track automation override, if any.
    pub fn get_global_automation_override(&self) -> Option<GlobalAutomationModeOverride>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        use GlobalAutomationModeOverride::*;
        match self.low.GetGlobalAutomationOverride() {
            -1 => None,
            6 => Some(Bypass),
            x => Some(Mode(AutomationMode::from_raw(x))),
        }
    }

    /// Sets the global track automation override.
    pub fn set_global_automation_override(
        &self,
        mode_override: Option<GlobalAutomationModeOverride>,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        use GlobalAutomationModeOverride::*;
        let raw = match mode_override {
            None => -1,
            Some(Bypass) => 6,
            Some(Mode(x)) => x.to_raw(),
        };
        self.low.SetGlobalAutomationOverride(raw);
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
        chunk_name: EnvChunkName,
    ) -> Option<TrackEnvelope>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self
            .low
            .GetTrackEnvelopeByChunkName(track.as_ptr(), chunk_name.into_raw().as_ptr());
        TrackEnvelope::new(ptr)
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
        env_name: impl Into<ReaperStringArg<'a>>,
    ) -> Option<TrackEnvelope>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self
            .low
            .GetTrackEnvelopeByName(track.as_ptr(), env_name.into().as_ptr());
        TrackEnvelope::new(ptr)
    }

    /// Returns the current peak volume for the given track channel.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_get_peak_info(&self, track: MediaTrack, channel: u32) -> ReaperVolumeValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let result = self.low.Track_GetPeakInfo(track.as_ptr(), channel as _);
        ReaperVolumeValue::new_panic(result)
    }

    /// Gets a track attribute as numerical value.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_media_track_info_value(
        &self,
        track: MediaTrack,
        attribute_key: TrackAttributeKey,
    ) -> f64
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .GetMediaTrackInfo_Value(track.as_ptr(), attribute_key.into_raw().as_ptr())
    }

    /// Gets a track track send, hardware output send or track receive attribute as numerical value.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_send_info_value(
        &self,
        track: MediaTrack,
        category: TrackSendCategory,
        send_index: u32,
        attribute_key: TrackSendAttributeKey,
    ) -> f64
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.GetTrackSendInfo_Value(
            track.as_ptr(),
            category.to_raw(),
            send_index as i32,
            attribute_key.into_raw().as_ptr(),
        )
    }

    /// Counts the number of items in the given track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn count_track_media_items(&self, track: MediaTrack) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CountTrackMediaItems(track.as_ptr()) as u32
    }

    /// Counts the number of FX parameter knobs displayed on the track control panel.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn count_tcp_fx_parms(&self, project: ProjectContext, track: MediaTrack) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CountTCPFXParms(project.to_raw(), track.as_ptr()) as u32
    }

    /// Returns information about a specific FX parameter knob displayed on the track control panel.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_tcp_fx_parm(
        &self,
        project: ProjectContext,
        track: MediaTrack,
        index: u32,
    ) -> ReaperFunctionResult<GetTcpFxParmResult>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut fx_index = MaybeUninit::uninit();
        let mut param_index = MaybeUninit::uninit();
        let successful = self.low.GetTCPFXParm(
            project.to_raw(),
            track.as_ptr(),
            index as _,
            fx_index.as_mut_ptr(),
            param_index.as_mut_ptr(),
        );
        if !successful {
            return Err(ReaperFunctionError::new("couldn't get TCP FX param info"));
        }
        let fx_index = fx_index.assume_init();
        let result = GetTcpFxParmResult {
            fx_location: TrackFxLocation::from_raw(fx_index),
            param_index: param_index.assume_init() as u32,
        };
        Ok(result)
    }

    /// Returns the media item on the given track at the given index.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_media_item(&self, track: MediaTrack, item_idx: u32) -> Option<MediaItem>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.GetTrackMediaItem(track.as_ptr(), item_idx as _);
        MediaItem::new(ptr)
    }

    /// Gets the number of FX instances on the given track's normal FX chain.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_count(&self, track: MediaTrack) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
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
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
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
        fx_location: TrackFxLocation,
    ) -> ReaperFunctionResult<GUID>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self
            .low
            .TrackFX_GetFXGUID(track.as_ptr(), fx_location.to_raw());
        deref(ptr).ok_or_else(|| {
            ReaperFunctionError::new("couldn't get FX GUID (probably FX doesn't exist)")
        })
    }

    /// Returns the current value of the given track FX in REAPER-normalized form.
    ///
    /// If the returned value is lower than zero, it can mean two things. Either there was an error,
    /// e.g. the FX or parameter doesn't exist, or the parameter can take exotic values. There's no
    /// way to distinguish between both cases. See [`ReaperNormalizedFxParamValue`] for details.
    ///  
    /// # Safety
    ///
    /// - REAPER can crash if you pass an invalid track.
    /// - Calling this from any other thread than the main thread causes undefined behavior!
    /// - However, there's one exception: Calling it in a real-time thread directly "from the track"
    ///   which is currently processing should be okay.
    ///
    /// [`ReaperNormalizedFxParamValue`]: struct.ReaperNormalizedFxParamValue.html
    pub unsafe fn track_fx_get_param_normalized(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        param_index: u32,
    ) -> ReaperNormalizedFxParamValue
    where
        UsageScope: AnyThread,
    {
        let raw_value = self.low.TrackFX_GetParamNormalized(
            track.as_ptr(),
            fx_location.to_raw(),
            param_index as i32,
        );
        ReaperNormalizedFxParamValue::new(raw_value)
    }

    /// Returns the master track of the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_master_track(&self, project: ProjectContext) -> MediaTrack
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.get_master_track_unchecked(project) }
    }

    /// Like [`get_master_track()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_master_track()`]: #method.get_master_track
    pub unsafe fn get_master_track_unchecked(&self, project: ProjectContext) -> MediaTrack
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.GetMasterTrack(project.to_raw());
        require_media_track_panic(ptr)
    }

    /// Converts the given GUID to a string (including braces).
    pub fn guid_to_string(&self, guid: &GUID) -> ReaperString
    where
        UsageScope: AnyThread,
    {
        let (guid_string, _) = with_string_buffer(64, |buffer, _| unsafe {
            self.low.guidToString(guid as *const GUID, buffer)
        });
        guid_string
    }

    /// Converts the given accelerator key to a human-readable name.
    pub fn kbd_format_key_name(&self, accel: Accel) -> ReaperString
    where
        UsageScope: AnyThread,
    {
        let (key_string, _) = with_string_buffer(64, |buffer, _| unsafe {
            let mut accel = accel.to_raw();
            self.low.kbd_formatKeyName(&mut accel as *mut _, buffer)
        });
        key_string
    }

    /// Returns the project recording path.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the resulting path you want.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
    pub fn get_project_path_ex(&self, project: ProjectContext, buffer_size: u32) -> Utf8PathBuf
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.get_project_path_ex_unchecked(project, buffer_size) }
    }

    /// Like [`get_project_path_ex()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_project_path_ex()`]: #method.get_project_path_ex
    pub unsafe fn get_project_path_ex_unchecked(
        &self,
        project: ProjectContext,
        buffer_size: u32,
    ) -> Utf8PathBuf
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let (reaper_string, _) = with_string_buffer(buffer_size, |buffer, max_size| {
            self.low
                .GetProjectPathEx(project.to_raw(), buffer, max_size)
        });
        let owned_string = reaper_string.into_string();
        Utf8PathBuf::from(owned_string)
    }

    /// Creates a marker or region.
    ///
    /// Returns the index of the created marker/region.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn add_project_marker_2<'a>(
        &self,
        project: ProjectContext,
        pos: MarkerOrRegionPosition,
        name: impl Into<ReaperStringArg<'a>>,
        at_index: Option<u32>,
        color: Option<NativeColor>,
    ) -> ReaperFunctionResult<u32>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.add_project_marker_2_unchecked(project, pos, name, at_index, color) }
    }

    /// Like [`add_project_marker_2()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`add_project_marker_2()`]: #method.add_project_marker_2
    pub unsafe fn add_project_marker_2_unchecked<'a>(
        &self,
        project: ProjectContext,
        pos: MarkerOrRegionPosition,
        name: impl Into<ReaperStringArg<'a>>,
        at_index: Option<u32>,
        color: Option<NativeColor>,
    ) -> ReaperFunctionResult<u32>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let (is_region, start, end) = match pos {
            MarkerOrRegionPosition::Marker(p) => (false, p.get(), 0.0),
            MarkerOrRegionPosition::Region(s, e) => (true, s.get(), e.get()),
        };
        let index = self.low.AddProjectMarker2(
            project.to_raw(),
            is_region,
            start,
            end,
            name.into().as_ptr(),
            at_index.map(|i| i as i32).unwrap_or(-1),
            color.map(|c| c.to_raw()).unwrap_or(0),
        );
        if index < 0 {
            return Err(ReaperFunctionError::new("failed to add project marker"));
        }
        Ok(index as u32)
    }

    /// Returns the master tempo of the current project.
    pub fn master_get_tempo(&self) -> Bpm
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        Bpm::new_panic(self.low.Master_GetTempo())
    }

    /// Sets the current tempo of the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn set_current_bpm(&self, project: ProjectContext, tempo: Bpm, undo_behavior: UndoBehavior)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe {
            self.set_current_bpm_unchecked(project, tempo, undo_behavior);
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
        project: ProjectContext,
        tempo: Bpm,
        undo_behavior: UndoBehavior,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.SetCurrentBPM(
            project.to_raw(),
            tempo.get(),
            undo_behavior == UndoBehavior::AddUndoPoint,
        );
    }

    /// Count the number of tempo/time signature markers in the project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn count_tempo_time_sig_markers(&self, project: ProjectContext) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.count_tempo_time_sig_markers_unchecked(project) }
    }

    /// Like [`set_current_bpm()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`set_current_bpm()`]: #method.set_current_bpm
    pub unsafe fn count_tempo_time_sig_markers_unchecked(&self, project: ProjectContext) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CountTempoTimeSigMarkers(project.to_raw()) as u32
    }

    /// Converts the given playback speed factor to a normalized play rate.
    pub fn master_normalize_play_rate_normalize(
        &self,
        value: PlaybackSpeedFactor,
    ) -> NormalizedPlayRate
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let result = self.low.Master_NormalizePlayRate(value.get(), false);
        NormalizedPlayRate::new(result)
    }

    /// Converts the given normalized play rate to a playback speed factor.
    pub fn master_normalize_play_rate_denormalize(
        &self,
        value: NormalizedPlayRate,
    ) -> PlaybackSpeedFactor
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let result = self.low.Master_NormalizePlayRate(value.get(), true);
        PlaybackSpeedFactor::new(result)
    }

    /// Returns the master play rate of the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn master_get_play_rate(&self, project: ProjectContext) -> PlaybackSpeedFactor
    where
        UsageScope: MainThreadOnly,
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
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.Master_GetPlayRate(project.to_raw());
        PlaybackSpeedFactor(raw)
    }

    /// Returns the master play rate of the given project at the given time.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn master_get_play_rate_at_time(
        &self,
        time: PositionInSeconds,
        project: ProjectContext,
    ) -> PlaybackSpeedFactor
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.master_get_play_rate_at_time_unchecked(time, project) }
    }

    /// Like [`master_get_play_rate_at_time()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`master_get_play_rate_at_time()`]: #method.master_get_play_rate_at_time
    pub unsafe fn master_get_play_rate_at_time_unchecked(
        &self,
        time: PositionInSeconds,
        project: ProjectContext,
    ) -> PlaybackSpeedFactor
    where
        UsageScope: AnyThread,
    {
        let raw = self
            .low
            .Master_GetPlayRateAtTime(time.get(), project.to_raw());
        PlaybackSpeedFactor(raw)
    }

    /// Sets the master play rate of the current project.
    pub fn csurf_on_play_rate_change(&self, play_rate: PlaybackSpeedFactor) {
        self.low.CSurf_OnPlayRateChange(play_rate.get());
    }

    /// Shows a message box to the user.
    ///
    /// Blocks the main thread.
    pub fn show_message_box<'a>(
        &self,
        message: impl Into<ReaperStringArg<'a>>,
        title: impl Into<ReaperStringArg<'a>>,
        r#type: MessageBoxType,
    ) -> MessageBoxResult
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let result = unsafe {
            self.low.ShowMessageBox(
                message.into().as_ptr(),
                title.into().as_ptr(),
                r#type.to_raw(),
            )
        };
        MessageBoxResult::from_raw(result)
    }

    /// Displays a text close to the transport bar.
    pub fn help_set<'a>(&self, message: impl Into<ReaperStringArg<'a>>, mode: HelpMode)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        unsafe { self.low.Help_Set(message.into().as_ptr(), mode.to_raw()) };
    }

    /// Parses the given string as GUID.
    ///
    /// # Errors
    ///
    /// Returns an error if the given string is not a valid GUID string.
    pub fn string_to_guid<'a>(
        &self,
        guid_string: impl Into<ReaperStringArg<'a>>,
    ) -> ReaperFunctionResult<GUID>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut guid = MaybeUninit::uninit();
        unsafe {
            self.low
                .stringToGuid(guid_string.into().as_ptr(), guid.as_mut_ptr());
        }
        let guid = unsafe { guid.assume_init() };
        if guid == ZERO_GUID {
            return Err(ReaperFunctionError::new("GUID string is invalid"));
        }
        Ok(guid)
    }

    /// Sets the input monitoring mode of the given track.
    ///
    /// Returns the new value.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_input_monitoring_change_ex(
        &self,
        track: MediaTrack,
        mode: InputMonitoringMode,
        gang_behavior: GangBehavior,
    ) -> InputMonitoringMode
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.CSurf_OnInputMonitorChangeEx(
            track.as_ptr(),
            mode.to_raw(),
            gang_behavior == GangBehavior::AllowGang,
        );
        InputMonitoringMode::from_raw(raw)
    }

    /// Sets the input monitoring mode of the given track.
    ///
    /// Has fewer side effects than [`Reaper::csurf_on_input_monitoring_change_ex`] and allows
    /// more fine-grained control of track grouping behavior.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_ui_input_monitor(
        &self,
        track: MediaTrack,
        mode: InputMonitoringMode,
        flags: BitFlags<SetTrackUiFlags>,
    ) -> InputMonitoringMode
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self
            .low
            .SetTrackUIInputMonitor(track.as_ptr(), mode.to_raw(), flags.bits() as _);
        InputMonitoringMode::from_raw(raw)
    }

    /// Scrolls the mixer so that the given track is the leftmost visible track.
    ///
    /// Returns the leftmost visible track after scrolling, which may be different from the given
    /// track if there are not enough tracks to its right. Not exactly sure what it's supposed to
    /// mean if this returns `None`, but it happens at times.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_mixer_scroll(&self, track: MediaTrack) -> Option<MediaTrack>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.SetMixerScroll(track.as_ptr());
        MediaTrack::new(ptr)
    }

    /// Creates a new media item.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn add_media_item_to_track(
        &self,
        track: MediaTrack,
    ) -> ReaperFunctionResult<MediaItem>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.AddMediaItemToTrack(track.as_ptr());
        MediaItem::new(ptr).ok_or(ReaperFunctionError::new("couldn't add item to track"))
    }

    /// Deletes the given media item.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track or item.
    pub unsafe fn delete_track_media_item(
        &self,
        track: MediaTrack,
        item: MediaItem,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.DeleteTrackMediaItem(track.as_ptr(), item.as_ptr());
        if !successful {
            return Err(ReaperFunctionError::new(
                "deletion of media item not successful",
            ));
        }
        Ok(())
    }

    /// Creates a new take in an item.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn add_take_to_media_item(
        &self,
        item: MediaItem,
    ) -> ReaperFunctionResult<MediaItemTake>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.AddTakeToMediaItem(item.as_ptr());
        MediaItemTake::new(ptr).ok_or(ReaperFunctionError::new("couldn't add take to item"))
    }

    /// Sets the position of the given item.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn set_media_item_position(
        &self,
        item: MediaItem,
        pos: PositionInSeconds,
        refresh_behavior: UiRefreshBehavior,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.SetMediaItemPosition(
            item.as_ptr(),
            pos.get(),
            refresh_behavior == UiRefreshBehavior::Refresh,
        );
        if !successful {
            return Err(ReaperFunctionError::new("couldn't set item position"));
        }
        Ok(())
    }

    /// Sets the length of the given item.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn set_media_item_length(
        &self,
        item: MediaItem,
        length: DurationInSeconds,
        refresh_behavior: UiRefreshBehavior,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.SetMediaItemLength(
            item.as_ptr(),
            length.get(),
            refresh_behavior == UiRefreshBehavior::Refresh,
        );
        if !successful {
            return Err(ReaperFunctionError::new("couldn't set item length"));
        }
        Ok(())
    }

    /// Selects or unselects the given media item.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn set_media_item_selected(&self, item: MediaItem, selected: bool)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.SetMediaItemSelected(item.as_ptr(), selected);
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
    pub unsafe fn set_media_track_info_value(
        &self,
        track: MediaTrack,
        attribute_key: TrackAttributeKey,
        new_value: f64,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.SetMediaTrackInfo_Value(
            track.as_ptr(),
            attribute_key.into_raw().as_ptr(),
            new_value,
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't set track attribute (maybe attribute key is invalid)",
            ));
        }
        Ok(())
    }

    /// Sets a track track send, hardware output send or track receive attribute as numerical value.
    ///
    /// # Errors
    ///
    /// Returns an error if an invalid (e.g. non-numerical) attribute key is passed.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_send_info_value(
        &self,
        track: MediaTrack,
        category: TrackSendCategory,
        send_index: u32,
        attribute_key: TrackSendAttributeKey,
        new_value: f64,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.SetTrackSendInfo_Value(
            track.as_ptr(),
            category.to_raw(),
            send_index as i32,
            attribute_key.into_raw().as_ptr(),
            new_value,
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't set track send attribute (maybe attribute key is invalid)",
            ));
        }
        Ok(())
    }

    /// Stuffs a 3-byte MIDI message into a queue or send it to an external MIDI hardware.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let session = reaper_medium::ReaperSession::default();
    /// use helgoboss_midi::test_util::note_on;
    /// use reaper_medium::StuffMidiMessageTarget::VirtualMidiKeyboardQueue;
    ///
    /// session.reaper().stuff_midi_message(VirtualMidiKeyboardQueue, note_on(0, 64, 100));
    /// ```
    pub fn stuff_midi_message(&self, target: StuffMidiMessageTarget, message: impl ShortMessage)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let bytes = message.to_bytes();
        self.low.StuffMIDIMessage(
            target.to_raw(),
            bytes.0.into(),
            bytes.1.into(),
            bytes.2.into(),
        );
    }

    /// Converts a decibel value into a volume slider value.
    pub fn db2slider(&self, value: Db) -> VolumeSliderValue
    where
        UsageScope: AnyThread,
    {
        VolumeSliderValue(self.low.DB2SLIDER(value.get()))
    }

    /// Converts a volume slider value into a decibel value.
    pub fn slider2db(&self, value: VolumeSliderValue) -> Db
    where
        UsageScope: AnyThread,
    {
        Db::new_panic(self.low.SLIDER2DB(value.get()))
    }

    /// Returns the given track's volume and incomplete pan. Also returns the correct value during
    /// the process of writing an automation envelope.
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
    ) -> ReaperFunctionResult<VolumeAndPan>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        // We zero them just for being safe.
        let mut volume = MaybeUninit::zeroed();
        let mut pan = MaybeUninit::zeroed();
        let successful =
            self.low
                .GetTrackUIVolPan(track.as_ptr(), volume.as_mut_ptr(), pan.as_mut_ptr());
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't get track volume and pan",
            ));
        }
        Ok(VolumeAndPan {
            volume: ReaperVolumeValue::new_panic(volume.assume_init()),
            pan: ReaperPanValue::new_panic(pan.assume_init()),
        })
    }

    /// Returns the given track's mute state. Also returns the correct value during the process of
    /// writing an automation envelope.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful (unclear when this happens).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_ui_mute(&self, track: MediaTrack) -> ReaperFunctionResult<bool>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        // We zero them just for being safe.
        let mut mute = MaybeUninit::zeroed();
        let successful = self.low.GetTrackUIMute(track.as_ptr(), mute.as_mut_ptr());
        if !successful {
            return Err(ReaperFunctionError::new("couldn't get track mute"));
        }
        Ok(mute.assume_init())
    }

    /// Returns the given track's complete pan. Also returns the correct value during the process of
    /// writing an automation envelope.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful (unclear when this happens).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_ui_pan(
        &self,
        track: MediaTrack,
    ) -> ReaperFunctionResult<GetTrackUiPanResult>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        // We zero them just for being safe.
        let mut pan_1 = MaybeUninit::zeroed();
        let mut pan_2 = MaybeUninit::zeroed();
        let mut pan_mode = MaybeUninit::zeroed();
        let successful = self.low.GetTrackUIPan(
            track.as_ptr(),
            pan_1.as_mut_ptr(),
            pan_2.as_mut_ptr(),
            pan_mode.as_mut_ptr(),
        );
        if !successful {
            return Err(ReaperFunctionError::new("couldn't get track pan"));
        }
        let pan_mode = PanMode::from_raw(pan_mode.assume_init());
        let res = GetTrackUiPanResult {
            pan_mode,
            pan_1: ReaperPanLikeValue(pan_1.assume_init()),
            pan_2: ReaperPanLikeValue(pan_2.assume_init()),
        };
        Ok(res)
    }

    /// Informs control surfaces that the given track's volume has changed.
    ///
    /// Doesn't actually change the volume.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track or an invalid control surface.
    pub unsafe fn csurf_set_surface_volume(
        &self,
        track: MediaTrack,
        volume: ReaperVolumeValue,
        notification_behavior: NotificationBehavior,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CSurf_SetSurfaceVolume(
            track.as_ptr(),
            volume.get(),
            notification_behavior.to_raw(),
        );
    }

    /// Sets the given track's volume, also supports relative changes and gang.
    ///
    /// Returns the new value. I think this only deviates if 0.0 is sent.
    /// Then it returns a slightly higher value - the one which actually corresponds to -150 dB.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_volume_change_ex(
        &self,
        track: MediaTrack,
        value_change: ValueChange<ReaperVolumeValue>,
        gang_behavior: GangBehavior,
    ) -> ReaperVolumeValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.CSurf_OnVolumeChangeEx(
            track.as_ptr(),
            value_change.value(),
            value_change.is_relative(),
            gang_behavior == GangBehavior::AllowGang,
        );
        ReaperVolumeValue::new_panic(raw)
    }

    /// Sets the given track's volume, also supports relative changes and gang.
    ///
    /// Returns the new value. I think this only deviates if 0.0 is sent.
    /// Then it returns a slightly higher value - the one which actually corresponds to -150 dB.
    ///
    /// Has fewer side effects than [`Reaper::csurf_on_volume_change_ex`] and allows more
    /// fine-grained control of track grouping behavior.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_ui_volume(
        &self,
        track: MediaTrack,
        value_change: ValueChange<ReaperVolumeValue>,
        progress: Progress,
        flags: BitFlags<SetTrackUiFlags>,
    ) -> ReaperVolumeValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.SetTrackUIVolume(
            track.as_ptr(),
            value_change.value(),
            value_change.is_relative(),
            progress.to_raw(),
            flags.bits() as _,
        );
        ReaperVolumeValue::new_panic(raw)
    }

    /// Informs control surfaces that the given track's pan has been changed.
    ///
    /// Doesn't actually change the pan.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track or an invalid control surface.
    pub unsafe fn csurf_set_surface_pan(
        &self,
        track: MediaTrack,
        pan: ReaperPanValue,
        notification_behavior: NotificationBehavior,
    ) where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .CSurf_SetSurfacePan(track.as_ptr(), pan.get(), notification_behavior.to_raw());
    }

    /// Sets the given track's pan. Also supports relative changes and gang.
    ///
    /// Returns the new value.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_pan_change_ex(
        &self,
        track: MediaTrack,
        value_change: ValueChange<ReaperPanValue>,
        gang_behavior: GangBehavior,
    ) -> ReaperPanValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.CSurf_OnPanChangeEx(
            track.as_ptr(),
            value_change.value(),
            value_change.is_relative(),
            gang_behavior == GangBehavior::AllowGang,
        );
        ReaperPanValue::new_panic(raw)
    }

    /// Sets the given track's pan. Also supports relative changes and gang.
    ///
    /// Returns the new value.
    ///
    /// Has fewer side effects than [`Reaper::csurf_on_pan_change_ex`] and allows more
    /// fine-grained control of track grouping behavior.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_ui_pan(
        &self,
        track: MediaTrack,
        value_change: ValueChange<ReaperPanValue>,
        progress: Progress,
        flags: BitFlags<SetTrackUiFlags>,
    ) -> ReaperPanValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.SetTrackUIPan(
            track.as_ptr(),
            value_change.value(),
            value_change.is_relative(),
            progress.to_raw(),
            flags.bits() as _,
        );
        ReaperPanValue::new_panic(raw)
    }

    /// Sets the given track's polarity (phase).
    ///
    /// Returns the new value.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_ui_polarity(
        &self,
        track: MediaTrack,
        value: TrackPolarityOperation,
        flags: BitFlags<SetTrackUiFlags>,
    ) -> TrackPolarity
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self
            .low
            .SetTrackUIPolarity(track.as_ptr(), value.to_raw(), flags.bits() as _);
        TrackPolarity::from_raw(raw)
    }

    /// Sets the given track's width. Also supports relative changes and gang.
    ///
    /// Returns the new value.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_width_change_ex(
        &self,
        track: MediaTrack,
        value_change: ValueChange<ReaperWidthValue>,
        gang_behavior: GangBehavior,
    ) -> ReaperWidthValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.CSurf_OnWidthChangeEx(
            track.as_ptr(),
            value_change.value(),
            value_change.is_relative(),
            gang_behavior == GangBehavior::AllowGang,
        );
        ReaperWidthValue::new(raw)
    }

    /// Sets the given track's width. Also supports relative changes and gang.
    ///
    /// Returns the new value.
    ///
    /// Has fewer side effects than [`Reaper::csurf_on_width_change_ex`] and allows more
    /// fine-grained control of track grouping behavior.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_ui_width(
        &self,
        track: MediaTrack,
        value_change: ValueChange<ReaperWidthValue>,
        progress: Progress,
        flags: BitFlags<SetTrackUiFlags>,
    ) -> ReaperWidthValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.SetTrackUIWidth(
            track.as_ptr(),
            value_change.value(),
            value_change.is_relative(),
            progress.to_raw(),
            flags.bits() as _,
        );
        ReaperWidthValue::new(raw)
    }

    /// Counts the number of selected tracks in the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn count_selected_tracks_2(
        &self,
        project: ProjectContext,
        master_track_behavior: MasterTrackBehavior,
    ) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.count_selected_tracks_2_unchecked(project, master_track_behavior) }
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
        project: ProjectContext,
        master_track_behavior: MasterTrackBehavior,
    ) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CountSelectedTracks2(
            project.to_raw(),
            master_track_behavior == MasterTrackBehavior::IncludeMasterTrack,
        ) as u32
    }

    /// Selects or unselects all media items in the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn select_all_media_items(&self, project: ProjectContext, selected: bool)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe {
            self.select_all_media_items_unchecked(project, selected);
        }
    }

    /// Like [`select_all_media_items()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`select_all_media_items()`]: #method.select_all_media_items
    pub unsafe fn select_all_media_items_unchecked(&self, project: ProjectContext, selected: bool)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.SelectAllMediaItems(project.to_raw(), selected);
    }

    /// Counts the number of selected items in the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn count_selected_media_items(&self, project: ProjectContext) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.require_valid_project(project);
        unsafe { self.count_selected_media_items_unchecked(project) }
    }

    /// Like [`count_selected_media_items()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`count_selected_media_items()`]: #method.count_selected_media_items
    pub unsafe fn count_selected_media_items_unchecked(&self, project: ProjectContext) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CountSelectedMediaItems(project.to_raw()) as u32
    }

    /// Selects or deselects the given track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_selected(&self, track: MediaTrack, is_selected: bool)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.SetTrackSelected(track.as_ptr(), is_selected);
    }

    /// Returns a selected track from the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_selected_track_2(
        &self,
        project: ProjectContext,
        selected_track_index: u32,
        master_track_behavior: MasterTrackBehavior,
    ) -> Option<MediaTrack>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe {
            self.get_selected_track_2_unchecked(
                project,
                selected_track_index,
                master_track_behavior,
            )
        }
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
        project: ProjectContext,
        selected_track_index: u32,
        master_track_behavior: MasterTrackBehavior,
    ) -> Option<MediaTrack>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.GetSelectedTrack2(
            project.to_raw(),
            selected_track_index as i32,
            master_track_behavior == MasterTrackBehavior::IncludeMasterTrack,
        );
        MediaTrack::new(ptr)
    }

    /// Returns a selected item from the given project.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_selected_media_item(
        &self,
        project: ProjectContext,
        selected_item_index: u32,
    ) -> Option<MediaItem>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_valid_project(project);
        unsafe { self.get_selected_media_item_unchecked(project, selected_item_index) }
    }

    /// Like [`get_selected_media_item()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_selected_media_item()`]: #method.get_selected_media_item
    pub unsafe fn get_selected_media_item_unchecked(
        &self,
        project: ProjectContext,
        selected_item_index: u32,
    ) -> Option<MediaItem>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self
            .low
            .GetSelectedMediaItem(project.to_raw(), selected_item_index as i32);
        MediaItem::new(ptr)
    }

    /// Returns the media source of the given media item take.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid take.
    pub unsafe fn get_media_item_take_source(&self, take: MediaItemTake) -> Option<PcmSource>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.GetMediaItemTake_Source(take.as_ptr());
        NonNull::new(ptr)
    }

    /// Returns the project which contains this item.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_item_project_context(&self, item: MediaItem) -> Option<ReaProject>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.GetItemProjectContext(item.as_ptr());
        ReaProject::new(ptr)
    }

    /// Returns the track which contains this item.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_media_item_track(&self, item: MediaItem) -> Option<MediaTrack>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.GetMediaItem_Track(item.as_ptr());
        MediaTrack::new(ptr)
    }

    /// Returns the active take in this item.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid item.
    pub unsafe fn get_active_take(&self, item: MediaItem) -> Option<MediaItemTake>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.GetActiveTake(item.as_ptr());
        MediaItemTake::new(ptr)
    }

    /// Returns the take that is currently being edited in the given MIDI editor.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid window.
    pub unsafe fn midi_editor_get_take(
        &self,
        midi_editor: Hwnd,
    ) -> ReaperFunctionResult<MediaItemTake>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.MIDIEditor_GetTake(midi_editor.as_ptr());
        MediaItemTake::new(ptr).ok_or(ReaperFunctionError::new("couldn't get MIDI editor take"))
    }

    /// Selects exactly one track and deselects all others.
    ///
    /// If `None` is passed, deselects all tracks.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_only_track_selected(&self, track: Option<MediaTrack>)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
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
    pub unsafe fn delete_track(&self, track: MediaTrack)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.DeleteTrack(track.as_ptr());
    }

    /// Returns the number of track sends, hardware output sends or track receives of the given
    /// track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_num_sends(&self, track: MediaTrack, category: TrackSendCategory) -> u32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.GetTrackNumSends(track.as_ptr(), category.to_raw()) as u32
    }

    /// Gets or sets an attribute of the given track send, hardware output send or track receive.
    ///
    /// Returns the current value if `new_value` is `null_mut()`.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track or invalid new value.
    pub unsafe fn get_set_track_send_info(
        &self,
        track: MediaTrack,
        category: TrackSendCategory,
        send_index: u32,
        attribute_key: TrackSendAttributeKey,
        new_value: *mut c_void,
    ) -> *mut c_void
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.GetSetTrackSendInfo(
            track.as_ptr(),
            category.to_raw(),
            send_index as i32,
            attribute_key.into_raw().as_ptr(),
            new_value,
        )
    }

    /// Convenience function which returns the destination track (`P_SRCTRACK`) of the given track
    /// send or track receive.
    ///
    /// The given index starts at zero for both track sends and receives.
    ///
    /// # Errors
    ///
    /// Returns an error e.g. if the send or receive doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_send_info_srctrack(
        &self,
        track: MediaTrack,
        direction: TrackSendDirection,
        send_index: u32,
    ) -> ReaperFunctionResult<MediaTrack>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_track_send_info(
            track,
            direction.into(),
            send_index,
            TrackSendAttributeKey::SrcTrack,
            null_mut(),
        ) as *mut raw::MediaTrack;
        MediaTrack::new(ptr).ok_or_else(|| {
            ReaperFunctionError::new("couldn't get source track (maybe send doesn't exist)")
        })
    }

    /// Convenience function which returns the destination track (`P_DESTTRACK`) of the given track
    /// send or track receive.
    ///
    /// The given index starts at zero for both track sends and receives.
    ///
    /// # Errors
    ///
    /// Returns an error e.g. if the send or receive doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_send_info_desttrack(
        &self,
        track: MediaTrack,
        direction: TrackSendDirection,
        send_index: u32,
    ) -> ReaperFunctionResult<MediaTrack>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.get_set_track_send_info(
            track,
            direction.into(),
            send_index,
            TrackSendAttributeKey::DestTrack,
            null_mut(),
        ) as *mut raw::MediaTrack;
        MediaTrack::new(ptr).ok_or_else(|| {
            ReaperFunctionError::new("couldn't get destination track (maybe send doesn't exist)")
        })
    }

    /// Returns the RPPXML state of the given track.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the chunk you want.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
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
        buffer_size: u32,
        cache_hint: ChunkCacheHint,
    ) -> ReaperFunctionResult<ReaperString>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        assert!(buffer_size > 0);
        let (chunk_content, successful) = with_string_buffer(buffer_size, |buffer, max_size| {
            self.low.GetTrackStateChunk(
                track.as_ptr(),
                buffer,
                max_size,
                cache_hint == ChunkCacheHint::UndoMode,
            )
        });
        if !successful {
            return Err(ReaperFunctionError::new("couldn't get track chunk"));
        }
        Ok(chunk_content)
    }

    /// Prompts the user for string values.
    ///
    /// If a caption begins with `*`, for example `*password`, the edit field will not display the
    /// input text. The maximum number of fields is 16. Values are returned as a comma-separated
    /// string.
    ///
    /// You can supply special extra information via additional caption fields:
    /// - `extrawidth=XXX` to increase text field width
    /// - `separator=X` to use a different separator for returned fields
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the resulting CSV you want.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
    pub fn get_user_inputs<'a>(
        &self,
        title: impl Into<ReaperStringArg<'a>>,
        num_inputs: u32,
        captions_csv: impl Into<ReaperStringArg<'a>>,
        initial_csv: impl Into<ReaperStringArg<'a>>,
        buffer_size: u32,
    ) -> Option<ReaperString>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        assert!(buffer_size > 0);
        let (csv, successful) =
            with_string_buffer_prefilled(initial_csv, buffer_size, |buffer, max_size| unsafe {
                self.low.GetUserInputs(
                    title.into().as_ptr(),
                    num_inputs as _,
                    captions_csv.into().as_ptr(),
                    buffer,
                    max_size,
                )
            });
        if !successful {
            return None;
        }
        Some(csv)
    }

    /// Creates a track send, track receive or hardware output send for the given track.
    ///
    /// Returns the index of the created track send (starting from 0) or of the created hardware
    /// output send (also starting from 0).
    ///
    /// # Errors
    ///
    /// Returns an error if not successful (unclear when this happens).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::{ProjectContext::CurrentProject, SendTarget::HardwareOutput};
    ///
    /// let src_track = session.reaper().get_track(CurrentProject, 0).ok_or("no tracks")?;
    /// let send_index = unsafe {
    ///     session.reaper().create_track_send(src_track, HardwareOutput)?;
    /// };
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub unsafe fn create_track_send(
        &self,
        track: MediaTrack,
        target: SendTarget,
    ) -> ReaperFunctionResult<u32>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let result = self.low.CreateTrackSend(track.as_ptr(), target.to_raw());
        if result < 0 {
            return Err(ReaperFunctionError::new("couldn't create track send"));
        }
        Ok(result as u32)
    }

    /// Removes a track send, track receive or hardware output send from the given track.
    ///
    /// # Errors
    ///
    /// Returns an error if not successful (unclear when this happens).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn remove_track_send(
        &self,
        track: MediaTrack,
        category: TrackSendCategory,
        send_index: u32,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful =
            self.low
                .RemoveTrackSend(track.as_ptr(), category.to_raw(), send_index as i32);
        if !successful {
            return Err(ReaperFunctionError::new("couldn't remove track send"));
        }
        Ok(())
    }

    /// Arms or disarms the given track for recording.
    ///
    /// Returns the new value.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_rec_arm_change_ex(
        &self,
        track: MediaTrack,
        mode: RecordArmMode,
        gang_behavior: GangBehavior,
    ) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CSurf_OnRecArmChangeEx(
            track.as_ptr(),
            mode.to_raw(),
            gang_behavior == GangBehavior::AllowGang,
        )
    }

    /// Arms or disarms the given track for recording.
    ///
    /// Returns the new value.
    ///
    /// Has fewer side effects than [`Reaper::csurf_on_rec_arm_change_ex`] and allows more
    /// fine-grained control of track grouping behavior.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_ui_rec_arm(
        &self,
        track: MediaTrack,
        value: TrackRecArmOperation,
        flags: BitFlags<SetTrackUiFlags>,
    ) -> RecordArmMode
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self
            .low
            .SetTrackUIRecArm(track.as_ptr(), value.to_raw(), flags.bits() as _);
        RecordArmMode::from_raw(raw)
    }

    /// Mutes or unmutes the given track.
    ///
    /// Seems to return the mute state that has been set.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_mute_change_ex(
        &self,
        track: MediaTrack,
        mute: bool,
        gang_behavior: GangBehavior,
    ) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CSurf_OnMuteChangeEx(
            track.as_ptr(),
            i32::from(mute),
            gang_behavior == GangBehavior::AllowGang,
        )
    }

    /// Mutes or unmutes the given track.
    ///
    /// Returns the new value.
    ///
    /// Has fewer side effects than [`Reaper::csurf_on_mute_change_ex`] and allows more
    /// fine-grained control of track grouping behavior.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_ui_mute(
        &self,
        track: MediaTrack,
        mute: TrackMuteOperation,
        flags: BitFlags<SetTrackUiFlags>,
    ) -> TrackMuteState
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self
            .low
            .SetTrackUIMute(track.as_ptr(), mute.to_raw(), flags.bits() as _);
        TrackMuteState::from_raw(raw)
    }

    /// Soloes or unsoloes the given track.
    ///
    /// Seems to return the solo state that has been set.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_solo_change_ex(
        &self,
        track: MediaTrack,
        solo: bool,
        gang_behavior: GangBehavior,
    ) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.CSurf_OnSoloChangeEx(
            track.as_ptr(),
            i32::from(solo),
            gang_behavior == GangBehavior::AllowGang,
        )
    }

    /// Soloes or unsoloes the given track.
    ///
    /// TODO-high The return value will change in future. Not clear yet how to interpret it.
    ///
    /// Returns the new value.
    ///
    /// Has fewer side effects than [`Reaper::csurf_on_solo_change_ex`] and allows more
    /// fine-grained control of track grouping behavior.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_ui_solo(
        &self,
        track: MediaTrack,
        value: TrackSoloOperation,
        flags: BitFlags<SetTrackUiFlags>,
    ) -> i32
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .SetTrackUISolo(track.as_ptr(), value.to_raw(), flags.bits() as _)
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
        chunk: impl Into<ReaperStringArg<'a>>,
        cache_hint: ChunkCacheHint,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.SetTrackStateChunk(
            track.as_ptr(),
            chunk.into().as_ptr(),
            cache_hint == ChunkCacheHint::UndoMode,
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't set track chunk (maybe chunk was invalid)",
            ));
        }
        Ok(())
    }

    /// Shows or hides an FX user interface.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_show(&self, track: MediaTrack, instruction: FxShowInstruction)
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.TrackFX_Show(
            track.as_ptr(),
            instruction.location_to_raw(),
            instruction.instruction_to_raw(),
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
        fx_location: TrackFxLocation,
    ) -> Option<Hwnd>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self
            .low
            .TrackFX_GetFloatingWindow(track.as_ptr(), fx_location.to_raw());
        Hwnd::new(ptr)
    }

    /// Returns whether the user interface of the given FX is open.
    ///
    /// *Open* means either visible in the FX chain window or visible in a floating window.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_open(&self, track: MediaTrack, fx_location: TrackFxLocation) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low
            .TrackFX_GetOpen(track.as_ptr(), fx_location.to_raw())
    }

    /// Returns the visibility state of the given track's normal FX chain.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_chain_visible(&self, track: MediaTrack) -> FxChainVisibility
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.TrackFX_GetChainVisible(track.as_ptr());
        FxChainVisibility::from_raw(raw)
    }

    /// Returns the visibility state of the master track.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub fn get_master_track_visibility(&self) -> BitFlags<TrackArea>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.GetMasterTrackVisibility();
        BitFlags::from_bits_truncate(raw as u32)
    }

    /// Sets the visibility state of the master track and returns the previous one.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub fn set_master_track_visibility(&self, areas: BitFlags<TrackArea>) -> BitFlags<TrackArea>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.SetMasterTrackVisibility(areas.bits() as _);
        BitFlags::from_bits_truncate(raw as u32)
    }

    /// Returns the visibility state of the given track's input FX chain.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_rec_chain_visible(&self, track: MediaTrack) -> FxChainVisibility
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.TrackFX_GetRecChainVisible(track.as_ptr());
        FxChainVisibility::from_raw(raw)
    }

    /// Sets the volume of the given track send or hardware output send.
    ///
    /// When choosing the send index, keep in mind that the hardware output sends (if any) come
    /// first.
    ///
    /// Returns the new value. If the send doesn't exist, returns 0.0 (which
    /// can also be a valid value that has been set, so that's not very useful).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_send_volume_change(
        &self,
        track: MediaTrack,
        send_index: u32,
        value_change: ValueChange<ReaperVolumeValue>,
    ) -> ReaperVolumeValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.CSurf_OnSendVolumeChange(
            track.as_ptr(),
            send_index as i32,
            value_change.value(),
            value_change.is_relative(),
        );
        ReaperVolumeValue::new_panic(raw)
    }

    /// Sets the pan of the given track send or hardware output send.
    ///
    /// When choosing the send index, keep in mind that the hardware output sends (if any) come
    /// first.
    ///
    /// Returns the new value.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn csurf_on_send_pan_change(
        &self,
        track: MediaTrack,
        send_index: u32,
        value_change: ValueChange<ReaperPanValue>,
    ) -> ReaperPanValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw = self.low.CSurf_OnSendPanChange(
            track.as_ptr(),
            send_index as i32,
            value_change.value(),
            value_change.is_relative(),
        );
        ReaperPanValue::new_panic(raw)
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
        command_id: CommandId,
        section: SectionContext,
        use_action_name: impl FnOnce(&ReaperStr) -> R,
    ) -> Option<R>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self
            .low
            .kbd_getTextFromCmd(command_id.get() as _, section.to_raw());
        create_passing_c_str(ptr)
            // Removed action returns empty string for some reason. We want None in this case!
            .filter(|s| !s.as_c_str().to_bytes().is_empty())
            .map(use_action_name)
    }

    /// Grants temporary access to the name of the given input channel.
    ///
    /// I encountered situations in which the name is not properly UTF-8 encoded, therefore, it's exposed as
    /// `CStr` instead of `ReaperStr`.
    pub fn get_input_channel_name<R>(
        &self,
        channel_index: u32,
        use_name: impl FnOnce(Option<&CStr>) -> R,
    ) -> R
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.GetInputChannelName(channel_index as _);
        let passing_c_str = if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        };
        use_name(passing_c_str)
    }

    /// Grants temporary access to the REAPER resource path.
    ///
    /// This is the path to the directory where INI files are stored and other things in
    /// subdirectories.
    pub fn get_resource_path<R>(&self, use_resource_path: impl FnOnce(&Utf8Path) -> R) -> R
    where
        UsageScope: AnyThread,
    {
        let ptr = self.low.GetResourcePath();
        let reaper_str =
            unsafe { create_passing_c_str(ptr).expect("should always return resource path") };
        let path = Utf8Path::new(reaper_str.to_str());
        use_resource_path(path)
    }

    /// Grants temporary access to the name of the given take.
    ///
    /// # Error
    ///
    /// Returns an error if the take is not valid.
    pub fn get_take_name<R>(
        &self,
        take: MediaItemTake,
        use_name: impl FnOnce(ReaperFunctionResult<&ReaperStr>) -> R,
    ) -> R
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let passing_c_str = unsafe {
            let ptr = self.low.GetTakeName(take.as_ptr());
            create_passing_c_str(ptr as *const c_char)
        };
        use_name(passing_c_str.ok_or_else(|| ReaperFunctionError::new("invalid take")))
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
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let result = self
            .low
            .GetToggleCommandState2(section.to_raw(), command_id.to_raw());
        if result == -1 {
            return None;
        }
        Some(result != 0)
    }

    /// Returns the current on/off state of a toggleable action, taking the section ID.
    ///
    /// Returns `None` if the action doesn't support on/off states (or if the action doesn't exist).
    pub fn get_toggle_command_state_ex(
        &self,
        section_id: SectionId,
        command_id: CommandId,
    ) -> Option<bool>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let result = self
            .low
            .GetToggleCommandStateEx(section_id.to_raw(), command_id.to_raw());
        if result == -1 {
            return None;
        }
        Some(result != 0)
    }

    /// Grants temporary access to the name of the command registered under the given command ID.
    ///
    /// The string will *not* start with `_` (e.g. it will return `SWS_ABOUT`).
    ///
    /// Returns `None` if the given command ID is a built-in action or if there's no such ID.
    pub fn reverse_named_command_lookup<R>(
        &self,
        command_id: CommandId,
        use_command_name: impl FnOnce(&ReaperStr) -> R,
    ) -> Option<R>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let ptr = self.low.ReverseNamedCommandLookup(command_id.to_raw());
        unsafe { create_passing_c_str(ptr) }.map(use_command_name)
    }

    /// Returns the volume and pan of the given track send or hardware output send. Also returns the
    /// correct value during the process of writing an automation envelope.
    ///
    /// When choosing the send index, keep in mind that the hardware output sends (if any) come
    /// first.
    ///
    /// # Errors
    ///
    /// Returns an error if the send doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_send_ui_vol_pan(
        &self,
        track: MediaTrack,
        send_index: u32,
    ) -> ReaperFunctionResult<VolumeAndPan>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        // We zero them just for being safe
        let mut volume = MaybeUninit::zeroed();
        let mut pan = MaybeUninit::zeroed();
        let successful = self.low.GetTrackSendUIVolPan(
            track.as_ptr(),
            send_index as i32,
            volume.as_mut_ptr(),
            pan.as_mut_ptr(),
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't get track send volume and pan (probably send doesn't exist)",
            ));
        }
        Ok(VolumeAndPan {
            volume: ReaperVolumeValue::new_panic(volume.assume_init()),
            pan: ReaperPanValue::new_panic(pan.assume_init()),
        })
    }

    /// Returns the volume and pan of the given track receive. Also returns the correct value during
    /// the process of writing an automation envelope.
    ///
    /// # Errors
    ///
    /// Returns an error if the send doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_receive_ui_vol_pan(
        &self,
        track: MediaTrack,
        receive_index: u32,
    ) -> ReaperFunctionResult<VolumeAndPan>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        // We zero them just for being safe
        let mut volume = MaybeUninit::zeroed();
        let mut pan = MaybeUninit::zeroed();
        let successful = self.low.GetTrackReceiveUIVolPan(
            track.as_ptr(),
            receive_index as i32,
            volume.as_mut_ptr(),
            pan.as_mut_ptr(),
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't get track receive volume and pan (probably receive doesn't exist)",
            ));
        }
        Ok(VolumeAndPan {
            volume: ReaperVolumeValue::new_panic(volume.assume_init()),
            pan: ReaperPanValue::new_panic(pan.assume_init()),
        })
    }

    /// Returns whether the given track send or hardware output send is muted. Also returns the
    /// correct value during the process of writing an automation envelope.
    ///
    /// When choosing the send index, keep in mind that the hardware output sends (if any) come
    /// first.
    ///
    /// # Errors
    ///
    /// Returns an error if the send doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_send_ui_mute(
        &self,
        track: MediaTrack,
        send_index: u32,
    ) -> ReaperFunctionResult<bool>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        // We zero them just for being safe
        let mut muted = MaybeUninit::zeroed();
        let successful =
            self.low
                .GetTrackSendUIMute(track.as_ptr(), send_index as i32, muted.as_mut_ptr());
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't get track send mute state (probably send doesn't exist)",
            ));
        }
        Ok(muted.assume_init())
    }

    /// Returns whether the given track receive is muted. Also returns the correct value during the
    /// process of writing an automation envelope.
    ///
    /// # Errors
    ///
    /// Returns an error if the send doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn get_track_receive_ui_mute(
        &self,
        track: MediaTrack,
        receive_index: u32,
    ) -> ReaperFunctionResult<bool>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        // We zero them just for being safe
        let mut muted = MaybeUninit::zeroed();
        let successful = self.low.GetTrackReceiveUIMute(
            track.as_ptr(),
            receive_index as i32,
            muted.as_mut_ptr(),
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't get track receive mute state (probably receive doesn't exist)",
            ));
        }
        Ok(muted.assume_init())
    }

    /// Toggles the mute state of the given track send, hardware output send or track receive.
    ///
    /// # Errors
    ///
    /// Returns an error if the send doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn toggle_track_send_ui_mute(
        &self,
        track: MediaTrack,
        send: TrackSendRef,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self
            .low
            .ToggleTrackSendUIMute(track.as_ptr(), send.to_raw());
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't toggle track send mute state (probably send doesn't exist)",
            ));
        }
        Ok(())
    }

    /// Sets the volume of the given track send, hardware output send or track receive.
    ///
    /// # Errors
    ///
    /// Returns an error if the send doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_send_ui_vol(
        &self,
        track: MediaTrack,
        send: TrackSendRef,
        volume: ReaperVolumeValue,
        edit_mode: EditMode,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.SetTrackSendUIVol(
            track.as_ptr(),
            send.to_raw(),
            volume.get(),
            edit_mode.to_raw(),
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't set track send volume (probably send doesn't exist)",
            ));
        }
        Ok(())
    }

    /// Sets the pan of the given track send, hardware output send or track receive.
    ///
    /// # Errors
    ///
    /// Returns an error if the send doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn set_track_send_ui_pan(
        &self,
        track: MediaTrack,
        send: TrackSendRef,
        pan: ReaperPanValue,
        edit_mode: EditMode,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.SetTrackSendUIPan(
            track.as_ptr(),
            send.to_raw(),
            pan.get(),
            edit_mode.to_raw(),
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't set track send pan (probably send doesn't exist)",
            ));
        }
        Ok(())
    }

    /// Returns the index of the currently selected FX preset as well as the total preset count.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    #[allow(clippy::if_same_then_else)]
    pub unsafe fn track_fx_get_preset_index(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
    ) -> TrackFxGetPresetIndexResult
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        // We zero this just for being safe
        let mut num_presets = MaybeUninit::zeroed();
        let index = self.low.TrackFX_GetPresetIndex(
            track.as_ptr(),
            fx_location.to_raw(),
            num_presets.as_mut_ptr(),
        );
        let num_presets = num_presets.assume_init();
        TrackFxGetPresetIndexResult {
            index: if index == -1 {
                // This either means the FX doesn't exist or it's a VST3 plug-in and the factory
                // preset is active. We can't distinguish between that. Justin says that querying of
                // the active VST3 presets is poorly defined by the spec so this can happen.
                None
            } else if index == num_presets {
                // For VST2 this means the factory preset is active.
                None
            } else {
                Some(index as u32)
            },
            count: num_presets as u32,
        }
    }

    /// Activates a preset of the given track FX by its index.
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
        fx_location: TrackFxLocation,
        preset: FxPresetRef,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.TrackFX_SetPresetByIndex(
            track.as_ptr(),
            fx_location.to_raw(),
            preset.to_raw(),
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't activate FX preset by index (maybe FX or preset doesn't exist)",
            ));
        }
        Ok(())
    }

    /// Activates a preset with the name shown in the REAPER dropdown.
    ///
    /// Full paths to `.vstpreset` files are also supported for VST3 plug-ins.
    ///
    /// # Errors
    ///
    /// Returns an error e.g. if the FX doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_set_preset<'a>(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        presetname: impl Into<ReaperStringArg<'a>>,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful = self.low.TrackFX_SetPreset(
            track.as_ptr(),
            fx_location.to_raw(),
            presetname.into().as_ptr(),
        );
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't select FX preset by its name (maybe FX or preset doesn't exist)",
            ));
        }
        Ok(())
    }

    /// Navigates within the presets of the given track FX.
    ///
    /// # Errors
    ///
    /// Returns an error e.g. if the FX doesn't exist.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let session = reaper_medium::ReaperSession::default();
    /// use reaper_medium::ProjectContext::CurrentProject;
    /// use reaper_medium::TrackFxLocation::NormalFxChain;
    ///
    /// let track = session.reaper().get_track(CurrentProject, 0).ok_or("no tracks")?;
    /// // Navigate 2 presets "up"
    /// unsafe {
    ///     session.reaper().track_fx_navigate_presets(track, NormalFxChain(0), -2)?
    /// };
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    pub unsafe fn track_fx_navigate_presets(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        increment: i32,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let successful =
            self.low
                .TrackFX_NavigatePresets(track.as_ptr(), fx_location.to_raw(), increment);
        if !successful {
            return Err(ReaperFunctionError::new(
                "couldn't navigate FX presets (maybe FX doesn't exist)",
            ));
        }
        Ok(())
    }

    /// Returns information about the currently selected preset of the given FX.
    ///
    /// *Currently selected* means the preset which is currently showing in the REAPER dropdown.
    ///
    /// With `buffer size` you can tell REAPER how many bytes of the preset name you want. If
    /// you are not interested in the preset name at all, pass 0.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid track.
    pub unsafe fn track_fx_get_preset(
        &self,
        track: MediaTrack,
        fx_location: TrackFxLocation,
        buffer_size: u32,
    ) -> TrackFxGetPresetResult
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        if buffer_size == 0 {
            let state_matches_preset =
                self.low
                    .TrackFX_GetPreset(track.as_ptr(), fx_location.to_raw(), null_mut(), 0);
            TrackFxGetPresetResult {
                state_matches_preset,
                name: None,
            }
        } else {
            let (name, state_matches_preset) =
                with_string_buffer(buffer_size, |buffer, max_size| {
                    self.low.TrackFX_GetPreset(
                        track.as_ptr(),
                        fx_location.to_raw(),
                        buffer,
                        max_size,
                    )
                });
            if name.is_empty() {
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
    /// Passes `None` to the given function if the device doesn't exist, is not connected or is not
    /// already opened. The device must be enabled in REAPER's MIDI preferences.
    ///
    /// This function is typically called in the [audio hook]. But it's also okay to call it in a
    /// VST plug-in as long as [`is_in_real_time_audio()`] returns `true`. If you are in the main
    /// thread and want to check if the device is open, use [`get_midi_input_is_open()`].
    ///
    /// See [audio hook] for an example.
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
    /// [audio hook]: struct.ReaperSession.html#method.audio_reg_hardware_hook_add
    /// [`is_in_real_time_audio()`]: #method.is_in_real_time_audio
    /// [`get_read_buf()`]: struct.MidiInput.html#method.get_read_buf
    /// [`get_midi_input_is_open()`]: #method.get_midi_input_is_open
    pub fn get_midi_input<R>(
        &self,
        device_id: MidiInputDeviceId,
        use_device: impl FnOnce(Option<&mut MidiInput>) -> R,
    ) -> R
    where
        UsageScope: AudioThreadOnly,
    {
        let ptr = self.low.GetMidiInput(device_id.to_raw());
        let mut arg = NonNull::new(ptr).map(MidiInput);
        use_device(arg.as_mut())
    }

    /// Returns if the given device is open (enabled in REAPER's MIDI preferences).
    pub fn get_midi_input_is_open(&self, device_id: MidiInputDeviceId) -> bool
    where
        UsageScope: AnyThread,
    {
        !self.low.GetMidiInput(device_id.to_raw()).is_null()
    }

    /// Grants temporary access to an already open MIDI output device.
    ///
    /// Passes `None` to the given function if the device doesn't exist, is not connected or is not
    /// already opened. The device must be enabled in REAPER's MIDI preferences.
    ///
    /// This function is typically called in the [audio hook]. But it's also okay to call it in a
    /// VST plug-in as long as [`is_in_real_time_audio()`] returns `true`. If you are in the main
    /// thread and want to check if the device is open, use [`get_midi_output_is_open()`].
    ///
    /// See [audio hook] for an example.
    ///
    /// [audio hook]: struct.ReaperSession.html#method.audio_reg_hardware_hook_add
    /// [`is_in_real_time_audio()`]: #method.is_in_real_time_audio
    /// [`get_read_buf()`]: struct.MidiInput.html#method.get_read_buf
    /// [`get_midi_output_is_open()`]: #method.get_midi_output_is_open
    pub fn get_midi_output<R>(
        &self,
        device_id: MidiOutputDeviceId,
        use_device: impl FnOnce(Option<&MidiOutput>) -> R,
    ) -> R
    where
        UsageScope: AudioThreadOnly,
    {
        let ptr = self.low.GetMidiOutput(device_id.to_raw());
        let arg = NonNull::new(ptr).map(MidiOutput);
        use_device(arg.as_ref())
    }

    // /// Grants temporary access to a MIDI output device that is not open yet.
    // pub fn create_midi_output<R>(
    //     &self,
    //     device_id: MidiOutputDeviceId,
    //     stream_mode: bool,
    //     use_device: impl FnOnce(Option<&MidiOutput>) -> R,
    // ) -> R {
    //     let ptr = unsafe {
    //         self.low
    //             .CreateMIDIOutput(device_id.to_raw(), stream_mode, null_mut())
    //     };
    //     let arg = NonNull::new(ptr).map(MidiOutput);
    //     let result = use_device(arg.as_ref());
    //     if let Some(output) = arg {
    //         unsafe {
    //             reaper_low::delete_midi_output(output.0);
    //         }
    //     }
    //     result
    // }

    /// Returns if the given device is open (enabled in REAPER's MIDI preferences).
    pub fn get_midi_output_is_open(&self, device_id: MidiOutputDeviceId) -> bool
    where
        UsageScope: AnyThread,
    {
        !self.low.GetMidiOutput(device_id.to_raw()).is_null()
    }

    /// Opens the given file in the Media Explorer.
    ///
    /// play=true will play the file immediately (or toggle playback if mediafn was already open), =false will just select it.
    ///
    /// When in doubt, it returns 0.0 (center).
    pub fn open_media_explorer(
        &self,
        file_name: &Utf8Path,
        mode: OpenMediaExplorerMode,
    ) -> Option<Hwnd>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let file_name_reaper_string = convert_path_to_reaper_string(file_name);
        let ptr = unsafe {
            self.low
                .OpenMediaExplorer(file_name_reaper_string.as_ptr(), mode.to_raw())
        };
        Hwnd::new(ptr)
    }

    // TODO-high document
    pub fn add_custom_menu_or_toolbar_item_separator<'a>(
        &self,
        menu_name: impl Into<ReaperStringArg<'a>>,
        pos: PositionDescriptor,
        refresh_behavior: UiRefreshBehavior,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.add_custom_menu_or_toolbar_item_internal(
            menu_name,
            pos,
            -1,
            0,
            null(),
            null(),
            refresh_behavior,
        )
    }

    // TODO-high document and improve design
    #[allow(clippy::too_many_arguments)]
    pub fn add_custom_menu_or_toolbar_item_command<'a>(
        &self,
        menu_name: impl Into<ReaperStringArg<'a>>,
        pos: PositionDescriptor,
        command_id: CommandId,
        toolbar_flags: u32,
        label: impl Into<ReaperStringArg<'a>>,
        // "toolbar_*.png"
        icon_file_name: Option<&Utf8Path>,
        // TODO-high This is a flag originally
        refresh_behavior: UiRefreshBehavior,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        let icon_file_name = icon_file_name.map(convert_path_to_reaper_string);
        self.add_custom_menu_or_toolbar_item_internal(
            menu_name,
            pos,
            command_id.to_raw(),
            toolbar_flags as i32,
            label.into().as_ptr(),
            icon_file_name
                .as_ref()
                .map(|n| n.as_ptr())
                .unwrap_or(null()),
            refresh_behavior,
        )
    }

    // TODO-high document and improve design
    #[allow(clippy::too_many_arguments)]
    fn add_custom_menu_or_toolbar_item_internal<'a>(
        &self,
        menu_name: impl Into<ReaperStringArg<'a>>,
        pos: PositionDescriptor,
        command_id: c_int,
        toolbarflags: c_int,
        str: *const c_char,
        iconfn: *const c_char,
        refresh_behavior: UiRefreshBehavior,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let extra_flags = match refresh_behavior {
            UiRefreshBehavior::NoRefresh => 1,
            UiRefreshBehavior::Refresh => 0,
        };
        let successful = unsafe {
            self.low.AddCustomMenuOrToolbarItem(
                menu_name.into().as_ptr(),
                pos.to_raw(),
                command_id,
                toolbarflags,
                str,
                iconfn,
                extra_flags,
            )
        };
        if !successful {
            return Err("couldn't add toolbar item".into());
        }
        Ok(())
    }

    // TODO-high document
    pub fn delete_custom_menu_or_toolbar_item<'a>(
        &self,
        menu_name: impl Into<ReaperStringArg<'a>>,
        pos: u32,
        // TODO-high This is a flag originally
        refresh_behavior: UiRefreshBehavior,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let extra_flags = match refresh_behavior {
            UiRefreshBehavior::NoRefresh => 1,
            UiRefreshBehavior::Refresh => 0,
        };
        let successful = unsafe {
            self.low.DeleteCustomMenuOrToolbarItem(
                menu_name.into().as_ptr(),
                pos as i32,
                extra_flags,
            )
        };
        if !successful {
            return Err("couldn't delete toolbar item".into());
        }
        Ok(())
    }

    /// Returns the current play loop count.
    ///
    /// # Panics
    ///
    /// Panics if the given project is not valid anymore.
    pub fn get_play_loop_cnt(&self, project: ProjectContext) -> i64
    where
        UsageScope: AnyThread,
    {
        self.require_valid_project(project);
        unsafe { self.get_play_loop_cnt_unchecked(project) }
    }

    /// Like [`get_play_loop_cnt()`] but doesn't check if project is valid.
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    ///
    /// [`get_play_loop_cnt()`]: #method.get_play_loop_cnt
    pub unsafe fn get_play_loop_cnt_unchecked(&self, project: ProjectContext) -> i64
    where
        UsageScope: AnyThread,
    {
        self.low.GetPlayLoopCnt(project.to_raw(), null_mut())
    }

    /// You can use this to step through times ahead of the current playback time, loopcnt will get updated on a loop or autoseek etc.
    ///
    /// ```ignore
    /// int asflags = 0; // running internal state for AdvancePlaybackPosition
    /// INT64 lc = GetPlayLoopCnt(proj, NULL);
    ///
    /// loop:
    ///   double nextpos = old_pos + len / srate;
    ///   int thislen = len;
    ///   int ret = AdvancePlaybackPosition(proj, old_pos, &nextpos, &lc, srate, &thislen, &asflags);
    ///   // ret 1 if looped sel, 2 if looped project, 4 if loopendskip, 8 if smoothseek, 16 if fade audition (all during this block)
    ///   // thislen may be decreased if partial block
    ///   oldpos = nextpos;
    /// goto loop
    /// ```
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid project.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn advance_playback_position_unchecked(
        &self,
        project: ProjectContext,
        old_pos: PositionInSeconds,
        next_pos: &mut PositionInSeconds,
        loop_count: &mut i64,
        sample_rate: Hz,
        max_slps: &mut i32,
        sf: &mut i32,
    ) -> BitFlags<AdvancePlaybackPositionEvent>
    where
        UsageScope: AnyThread,
    {
        let mut raw_next_pos = next_pos.get();
        let ret = self.low.AdvancePlaybackPosition(
            project.to_raw(),
            old_pos.get(),
            &mut raw_next_pos as *mut _,
            loop_count as *mut _,
            sample_rate.get(),
            max_slps as *mut _,
            sf as *mut _,
        );
        *next_pos = PositionInSeconds::new_panic(raw_next_pos);
        BitFlags::from_bits_truncate(ret as u32)
    }

    /// Returns `true` if the given window is a text field or should behave as such (JSFX editor, hooked via
    /// `hwnd_info`, etc.).
    ///
    /// # Safety
    ///
    /// REAPER can crash if you pass an invalid window handle.
    pub unsafe fn is_window_text_field(&self, window: Hwnd) -> bool
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        self.low.IsWindowTextField(window.as_ptr())
    }

    // TODO-high document
    pub fn get_custom_menu_or_toolbar_item<'a, R>(
        &self,
        menu_name: impl Into<ReaperStringArg<'a>>,
        pos: u32,
        use_result: impl FnOnce(Option<MenuOrToolbarItem<&ReaperStr, &ReaperStr>>) -> R,
    ) -> R
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let mut command_id = MaybeUninit::zeroed();
        let mut toolbar_flags = MaybeUninit::zeroed();
        let mut label = MaybeUninit::zeroed();
        let mut icon_file_name = MaybeUninit::zeroed();
        let successful = unsafe {
            self.low.GetCustomMenuOrToolbarItem(
                menu_name.into().as_ptr(),
                pos as i32,
                command_id.as_mut_ptr(),
                toolbar_flags.as_mut_ptr(),
                label.as_mut_ptr(),
                icon_file_name.as_mut_ptr(),
            )
        };
        if !successful {
            // No more items
            return use_result(None);
        }
        // Extract values
        let command_id = unsafe { command_id.assume_init() };
        let toolbar_flags = unsafe { toolbar_flags.assume_init() };
        let label = unsafe {
            let label_ptr = label.assume_init();
            if label_ptr.is_null() {
                reaper_str!("")
            } else {
                ReaperStr::from_ptr(label_ptr)
            }
        };
        let icon_file_name = unsafe {
            let icon_file_name_ptr = icon_file_name.assume_init();
            if icon_file_name_ptr.is_null() {
                reaper_str!("")
            } else {
                ReaperStr::from_ptr(icon_file_name.assume_init())
            }
        };
        let item = match command_id {
            -3 => MenuOrToolbarItem::SubMenuEnd,
            -2 => MenuOrToolbarItem::SubMenuStart(SubMenuStart { label }),
            -1 => MenuOrToolbarItem::Separator,
            id if id > 0 => MenuOrToolbarItem::Command(CommandItem {
                command_id: CommandId::new(command_id as _),
                toolbar_flags: toolbar_flags as u32,
                label,
                icon_file_name: if icon_file_name.to_str().is_empty() {
                    None
                } else {
                    Some(icon_file_name)
                },
            }),
            _ => panic!("unexpected menu or toolbar item type"),
        };
        use_result(Some(item))
    }

    /// Parses the given string as pan value.
    ///
    /// When in doubt, it returns 0.0 (center).
    pub fn parse_pan_str<'a>(&self, pan_string: impl Into<ReaperStringArg<'a>>) -> ReaperPanValue
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let raw_pan = unsafe { self.low.parsepanstr(pan_string.into().as_ptr()) };
        ReaperPanValue::new_panic(raw_pan)
    }

    /// Formats the given pan value.
    pub fn mk_pan_str(&self, value: ReaperPanValue) -> ReaperString
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let (pan_string, _) = with_string_buffer(64, |buffer, _| unsafe {
            self.low.mkpanstr(buffer, value.get());
        });
        pan_string
    }

    /// Formats the given volume value.
    pub fn mk_vol_str(&self, value: ReaperVolumeValue) -> ReaperString
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let (volume_string, _) = with_string_buffer(64, |buffer, _| unsafe {
            self.low.mkvolstr(buffer, value.get());
        });
        volume_string
    }

    /// Formats the given position in time.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the time string you want.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
    pub fn format_timestr_pos(
        &self,
        tpos: PositionInSeconds,
        buffer_size: u32,
        mode_override: TimeModeOverride,
    ) -> ReaperString
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let (time_string, _) = with_string_buffer(buffer_size, |buffer, max| unsafe {
            self.low
                .format_timestr_pos(tpos.get(), buffer, max, mode_override.to_raw());
        });
        time_string
    }

    /// Returns information about the currently open audio device.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the information you want.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
    pub fn get_audio_device_info(
        &self,
        key: AudioDeviceAttributeKey,
        buffer_size: u32,
    ) -> ReaperFunctionResult<ReaperString>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let (info, successful) = with_string_buffer(buffer_size, |buffer, max| unsafe {
            self.low
                .GetAudioDeviceInfo(key.into_raw().as_ptr(), buffer, max)
        });
        if !successful {
            return Err(ReaperFunctionError::new(
                "audio device not open or attribute doesn't exist",
            ));
        }
        Ok(info)
    }

    /// Formats the given duration, starting from the given timeline position offset.
    ///
    /// With `buffer_size` you can tell REAPER how many bytes of the time string you want.
    ///
    /// # Panics
    ///
    /// Panics if the given buffer size is 0.
    pub fn format_timestr_len(
        &self,
        tpos: DurationInSeconds,
        buffer_size: u32,
        offset: PositionInSeconds,
        mode_override: TimeModeOverride,
    ) -> ReaperString
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let (time_string, _) = with_string_buffer(buffer_size, |buffer, max| unsafe {
            self.low.format_timestr_len(
                tpos.get(),
                buffer,
                max,
                offset.get(),
                mode_override.to_raw(),
            );
        });
        time_string
    }

    /// Inserts the given file as new media item.
    ///
    /// # Errors
    ///
    /// Returns an error when inserting the file failed.
    pub fn insert_media(
        &self,
        file: impl AsRef<Utf8Path>,
        mode: InsertMediaMode,
        flags: BitFlags<InsertMediaFlag>,
    ) -> ReaperFunctionResult<()>
    where
        UsageScope: MainThreadOnly,
    {
        self.require_main_thread();
        let path_str_c = CString::new(file.as_ref().as_str().as_bytes()).unwrap();
        let result = unsafe {
            self.low
                .InsertMedia(path_str_c.as_ptr(), mode.to_raw(flags))
        };
        if result == 0 {
            return Err(ReaperFunctionError::new("couldn't insert media"));
        }
        Ok(())
    }

    fn require_main_thread(&self)
    where
        UsageScope: AnyThread,
    {
        self.plugin_context().require_main_thread();
    }

    pub(crate) fn require_valid_project(&self, project: ProjectContext)
    where
        UsageScope: AnyThread,
    {
        assert!(
            self.project_is_valid(project),
            "ReaProject doesn't exist anymore"
        );
    }

    pub(crate) fn project_is_valid(&self, project: ProjectContext) -> bool
    where
        UsageScope: AnyThread,
    {
        if let ProjectContext::Proj(p) = project {
            self.validate_ptr_2(CurrentProject, p)
        } else {
            true
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GetParameterStepSizesResult {
    /// Normal (non-toggleable) parameter.
    ///
    /// Each of the decimal numbers are > 0. They relate to the value range reported by
    /// [`track_fx_get_param_ex()`], so don't just interpret them as normalized values (step sizes
    /// within the unit interval).
    ///
    /// [`track_fx_get_param_ex()`]: struct.Reaper.html#method.track_fx_get_param_ex
    Normal {
        normal_step: f64,
        small_step: Option<f64>,
        large_step: Option<f64>,
    },
    /// Toggleable parameter.
    Toggle,
}

/// Each of these values can be negative! They are not normalized.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct GetParamExResult {
    /// Current value.
    pub current_value: f64,
    /// Minimum possible value.
    pub min_value: f64,
    /// Center value.
    pub mid_value: f64,
    /// Maximum possible value.
    pub max_value: f64,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct EnumProjectsResult {
    /// Project pointer.
    pub project: ReaProject,
    /// Path to project file (only if project saved and path requested).
    pub file_path: Option<Utf8PathBuf>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct GetMidiDevNameResult {
    /// Whether the device is currently connected.
    pub is_present: bool,
    /// Name of the device (only if name requested and device known).
    ///
    /// This is a [`CString`] instead of a [`ReaperString`] because REAPER versions at least up to < v7.19 have
    /// a bug that can cause the resulting string to not be proper UTF-8 if the MIDI device contains non-ASCII
    /// characters. So one must be careful when interpreting the result.
    pub name: Option<CString>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum EnumPitchShiftModesResult<'a> {
    /// Pitch shift mode exists but is currently unsupported.
    Unsupported,
    /// Pitch shift mode exists and is supported.
    Supported {
        /// Name of the pitch shift mode.
        name: &'a ReaperStr,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GetInputOutputLatencyResult {
    pub input_latency: u32,
    pub output_latency: u32,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TrackFxGetPresetResult {
    /// Whether the current state of the FX matches the preset.
    ///
    /// `false` if the current FX parameters do not exactly match the preset (in other words, if
    /// the user loaded the preset but moved the knobs afterwards).
    pub state_matches_preset: bool,
    /// Name of the preset.
    pub name: Option<ReaperString>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct TrackFxGetPresetIndexResult {
    /// Preset index or `None` if no preset or factory preset is selected or the FX doesn't exist.
    pub index: Option<u32>,
    /// Total number of presets available.
    ///
    /// 0 if the FX doesn't exist.
    pub count: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ProjectConfigVarGetOffsResult {
    /// Offset to pass to [`project_config_var_addr`].
    ///
    /// [`project_config_var_addr`]: struct.Reaper.html#method.project_config_var_addr
    pub offset: u32,
    /// Size of the object.
    pub size: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GetConfigVarResult {
    /// Size of the value.
    pub size: u32,
    /// Pointer to the REAPER preference value.
    pub value: NonNull<c_void>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PlayState {
    /// Is playing.
    pub is_playing: bool,
    /// Is paused.
    pub is_paused: bool,
    /// Is recording.
    pub is_recording: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct EnumProjectMarkers3Result<'a> {
    pub position: PositionInSeconds,
    pub region_end_position: Option<PositionInSeconds>,
    pub name: &'a ReaperStr,
    pub id: BookmarkId,
    pub color: NativeColor,
}

/// The given indexes count both markers and regions.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct GetLastMarkerAndCurRegionResult {
    pub marker_index: Option<u32>,
    pub region_index: Option<u32>,
}

/// The given indexes count both markers and regions.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct GetLoopTimeRange2Result {
    pub start: PositionInSeconds,
    pub end: PositionInSeconds,
}

/// The arrange view start/end time for the given screen coordinates.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct GetSetArrangeView2Result {
    pub start_time: PositionInSeconds,
    pub end_time: PositionInSeconds,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TimeMap2TimeToBeatsResult {
    /// Position in beats since project start.
    pub full_beats: PositionInBeats,
    /// Index of the measure in which the given position is located.
    pub measure_index: i32,
    /// Position in beats within that measure.
    pub beats_since_measure: PositionInBeats,
    /// Time signature of that measure.
    pub time_signature: TimeSignature,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TimeMapGetMeasureInfoResult {
    /// Start position of the measure in seconds.
    pub start_time: PositionInSeconds,
    /// Start position of the measure in quarter notes.
    pub start_qn: PositionInQuarterNotes,
    /// End position of the measure in quarter notes.
    pub end_qn: PositionInQuarterNotes,
    /// Time signature of that measure.
    pub time_signature: TimeSignature,
    /// Tempo at that measure.
    pub tempo: Bpm,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TimeMapQnToMeasuresResult {
    /// Measure index in project.
    pub measure_index: i32,
    /// Start position of the measure in quarter notes.
    pub start: PositionInQuarterNotes,
    /// End position of the measure in quarter notes.
    pub end: PositionInQuarterNotes,
}

/// Time signature.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TimeSignature {
    /// Measure length in beats.
    pub numerator: NonZeroU32,
    /// What musical unit one beat stands for.
    pub denominator: NonZeroU32,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CountProjectMarkersResult {
    pub total_count: u32,
    pub marker_count: u32,
    pub region_count: u32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct VolumeAndPan {
    /// Volume.
    pub volume: ReaperVolumeValue,
    /// Pan.
    pub pan: ReaperPanValue,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct SetEditCurPosOptions {
    pub move_view: bool,
    pub seek_play: bool,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GetTrackUiPanResult {
    /// The pan mode.
    pub pan_mode: PanMode,
    /// Pan value 1.
    ///
    /// Depending on the mode, this is either the only pan, the main pan or the left pan.
    pub pan_1: ReaperPanLikeValue,
    /// Pan value 2.
    ///
    /// Depending on the mode, this is either the width or the right pan.
    pub pan_2: ReaperPanLikeValue,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GetTcpFxParmResult {
    /// Location of the FX on that track.
    pub fx_location: TrackFxLocation,
    /// Index of the parameter.
    pub param_index: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GetLastTouchedFxResult {
    /// The last touched FX is a track FX.
    TrackFx {
        /// Track on which the FX is located.
        track_location: TrackLocation,
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
pub struct GetFocusedFx2Result {
    /// Whether the FX is still focused (vs. unfocused but still open).
    pub is_still_focused: bool,
    /// Returns the actual FX.
    pub fx: GetFocusedFxResult,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GetFocusedFxResult {
    /// The (last) focused FX is a track FX.
    TrackFx {
        /// Track on which the FX is located.
        track_location: TrackLocation,
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
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GetTouchedOrFocusedFxCurrentlyFocusedFxResult {
    /// Whether the FX is still focused (vs. unfocused but still open).
    pub is_still_focused: bool,
    /// Returns the actual FX.
    pub fx: FxLocation,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FxLocation {
    /// The (last) focused FX is a track FX.
    TrackFx {
        /// Track on which the FX is located.
        track_location: TrackLocation,
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct NativeColorValue {
    /// The OS-dependent color.
    pub color: NativeColor,
    /// Whether the color is actually displayed (vs. just internally stored).
    pub is_used: bool,
}

impl NativeColorValue {
    const USED_OFFSET: i32 = 0x1000000;

    fn from_raw(value: i32) -> Self {
        if value < Self::USED_OFFSET {
            Self {
                color: NativeColor::new(value as _),
                is_used: false,
            }
        } else {
            Self {
                color: NativeColor::new((value - Self::USED_OFFSET) as _),
                is_used: true,
            }
        }
    }

    fn to_raw(self) -> i32 {
        if self.is_used {
            self.color.to_raw() + Self::USED_OFFSET
        } else {
            self.color.to_raw()
        }
    }
}

fn make_some_if_greater_than_zero(value: f64) -> Option<f64> {
    if value <= 0.0 || value.is_nan() {
        return None;
    }
    Some(value)
}

fn make_some_if_not_negative(value: i32) -> Option<u32> {
    if value < 0 {
        return None;
    }
    Some(value as _)
}

unsafe fn deref<T: Copy>(ptr: *const T) -> Option<T> {
    if ptr.is_null() {
        return None;
    }
    Some(*ptr)
}

unsafe fn deref_as<T: Copy>(ptr: *mut c_void) -> Option<T> {
    deref(ptr as *const T)
}

fn convert_tracknumber_to_track_location(tracknumber: u32) -> TrackLocation {
    if tracknumber == 0 {
        TrackLocation::MasterTrack
    } else {
        TrackLocation::NormalTrack(tracknumber - 1)
    }
}

const ZERO_GUID: GUID = GUID {
    Data1: 0,
    Data2: 0,
    Data3: 0,
    Data4: [0; 8],
};

fn convert_path_to_reaper_string(path: &Utf8Path) -> ReaperString {
    ReaperString::from_str(path.as_str())
}

mod private {
    use crate::{MainThreadScope, RealTimeAudioThreadScope};

    pub trait Sealed {}

    impl Sealed for MainThreadScope {}

    impl Sealed for RealTimeAudioThreadScope {}
}
