use crate::{
    concat_c_strs, Hwnd, KbdSectionInfo, MediaTrack, MidiOutputDeviceId, ReaProject,
    ReaperControlSurface, ReaperStringArg,
};
use c_str_macro::c_str;
use helgoboss_midi::{U14, U7};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use reaper_rs_low::raw;
use reaper_rs_low::raw::HWND;
use std::borrow::Cow;
use std::ffi::CStr;
use std::ptr::null_mut;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AddFxBehavior {
    AddIfNotFound,
    AlwaysAdd,
}

impl From<AddFxBehavior> for FxAddByNameBehavior {
    fn from(b: AddFxBehavior) -> Self {
        use AddFxBehavior::*;
        match b {
            AddIfNotFound => FxAddByNameBehavior::AddIfNotFound,
            AlwaysAdd => FxAddByNameBehavior::AlwaysAdd,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackFxChainType {
    NormalFxChain,
    /// On the master track this corresponds to the monitoring FX chain
    InputFxChain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MasterTrackBehavior {
    ExcludeMasterTrack,
    IncludeMasterTrack,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChunkCacheHint {
    NormalMode,
    // Justin Frankel about is_undo parameter:
    //
    // A few notable things that happen with "isundo" set:
    //
    // - if undo is set and getting the chunk, then VST/etc plug-in
    // configurations are cached, e.g. if the plug-in hasn't recently notified
    // of a parameter change, we use the last configuration state (which is
    // faster). The downside is if the plug-in doesn't properly report its
    // state as having changed, you wouldn't get the latest version.
    // - if undo is set and setting a chunk, envelope lane sizes will not be
    // updated from the configuration state
    // - the format in which FX GUIDs are encoded is slightly different in undo
    // vs normal (to facilitate more efficient re-use of existing plug-in
    // instances)
    // - the logic in saving the event data for pooled MIDI items is slightly
    // different (in undo mode only one of the items in the pool will encode,
    // with undo=false the first instance in the GetStateChunk will get the data)
    UndoMode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ValueChange<T: Copy + Into<f64>> {
    Absolute(T),
    Relative(f64),
}

impl<T: Copy + Into<f64>> ValueChange<T> {
    pub(crate) fn value(&self) -> f64 {
        use ValueChange::*;
        match self {
            Absolute(v) => (*v).into(),
            Relative(v) => *v,
        }
    }

    pub(crate) fn is_relative(&self) -> bool {
        matches!(self, ValueChange::Relative(_))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UndoBehavior {
    OmitUndoPoint,
    AddUndoPoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferBehavior {
    Copy,
    Move,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackDefaultsBehavior {
    OmitDefaultEnvAndFx,
    AddDefaultEnvAndFx,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GangBehavior {
    DenyGang,
    AllowGang,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum RecordArmState {
    Unarmed = 0,
    Armed = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum FxShowFlag {
    HideChain = 0,
    ShowChain = 1,
    HideFloatingWindow = 2,
    ShowFloatingWindow = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum TrackSendDirection {
    Receive = -1,
    Send = 0,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum TrackSendCategory {
    Receive = -1,
    Send = 0,
    HardwareOutput = 1,
}

impl From<TrackSendDirection> for TrackSendCategory {
    fn from(v: TrackSendDirection) -> Self {
        use TrackSendDirection::*;
        match v {
            Receive => TrackSendCategory::Receive,
            Send => TrackSendCategory::Send,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StuffMidiMessageTarget {
    VirtualMidiKeyboardQueue,
    MidiAsControlInputQueue,
    VirtualMidiKeyboardQueueOnCurrentChannel,
    MidiOutputDevice(MidiOutputDeviceId),
}

impl From<StuffMidiMessageTarget> for i32 {
    fn from(t: StuffMidiMessageTarget) -> Self {
        use StuffMidiMessageTarget::*;
        match t {
            VirtualMidiKeyboardQueue => 0,
            MidiAsControlInputQueue => 1,
            VirtualMidiKeyboardQueueOnCurrentChannel => 2,
            MidiOutputDevice(id) => 16 + id.0 as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackFxRef {
    NormalFxChain(u32),
    InputFxChain(u32),
}

// Converts directly to the i32 value that is expected by low-level track-FX related functions
impl From<TrackFxRef> for i32 {
    fn from(v: TrackFxRef) -> Self {
        use TrackFxRef::*;
        let positive = match v {
            InputFxChain(idx) => 0x1000000 + idx,
            NormalFxChain(idx) => idx,
        };
        positive as i32
    }
}

// Converts from a value returned by low-level track-FX related functions turned into u32.
impl From<u32> for TrackFxRef {
    fn from(v: u32) -> Self {
        use TrackFxRef::*;
        if v >= 0x1000000 {
            InputFxChain(v - 0x1000000)
        } else {
            NormalFxChain(v)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum FxAddByNameBehavior {
    AlwaysAdd = -1,
    Query = 0,
    AddIfNotFound = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionValueChange {
    AbsoluteLowRes(U7),
    AbsoluteHighRes(U14),
    Relative1(U7),
    Relative2(U7),
    Relative3(U7),
}

#[derive(Clone, Debug)]
pub enum RegistrationType<'a> {
    Api(Cow<'a, CStr>),
    ApiDef(Cow<'a, CStr>),
    HookCommand,
    HookPostCommand,
    HookCommand2,
    ToggleAction,
    ActionHelp,
    CommandId,
    CommandIdLookup,
    GAccel,
    CSurfInst,
    Custom(Cow<'a, CStr>),
}

impl<'a> RegistrationType<'a> {
    pub fn api(func_name: impl Into<ReaperStringArg<'a>>) -> Self {
        Self::Api(func_name.into().into_inner())
    }

    pub fn api_def(func_def: impl Into<ReaperStringArg<'a>>) -> Self {
        Self::ApiDef(func_def.into().into_inner())
    }

    pub fn custom(key: impl Into<ReaperStringArg<'a>>) -> Self {
        Self::Custom(key.into().into_inner())
    }
}

impl<'a> From<RegistrationType<'a>> for Cow<'a, CStr> {
    fn from(value: RegistrationType<'a>) -> Self {
        use RegistrationType::*;
        match value {
            GAccel => c_str!("gaccel").into(),
            CSurfInst => c_str!("csurf_inst").into(),
            Api(func_name) => concat_c_strs(c_str!("API_"), func_name.as_ref()).into(),
            ApiDef(func_def) => concat_c_strs(c_str!("APIdef_"), func_def.as_ref()).into(),
            HookCommand => c_str!("hookcommand").into(),
            HookPostCommand => c_str!("hookpostcommand").into(),
            HookCommand2 => c_str!("hookcommand2").into(),
            ToggleAction => c_str!("toggleaction").into(),
            ActionHelp => c_str!("action_help").into(),
            CommandId => c_str!("command_id").into(),
            CommandIdLookup => c_str!("command_id_lookup").into(),
            Custom(k) => k,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackRef {
    MasterTrack,
    NormalTrack(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum InputMonitoringMode {
    Off = 0,
    Normal = 1,
    /// Tape style
    NotWhenPlaying = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectRef {
    Current,
    CurrentlyRendering,
    Tab(u32),
}

impl From<ProjectRef> for i32 {
    fn from(r: ProjectRef) -> Self {
        use ProjectRef::*;
        match r {
            Current => -1,
            CurrentlyRendering => 0x40000000,
            Tab(i) => i as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FxPresetRef {
    FactoryPreset,
    DefaultUserPreset,
    Preset(u32),
}

impl From<FxPresetRef> for i32 {
    fn from(r: FxPresetRef) -> Self {
        use FxPresetRef::*;
        match r {
            FactoryPreset => -2,
            DefaultUserPreset => -1,
            Preset(idx) => idx as i32,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectContext {
    CurrentProject,
    // Mmh, should we allow passing just a project by using impl Into<ProjectContext>?
    Proj(ReaProject),
}

impl From<ProjectContext> for *mut raw::ReaProject {
    fn from(c: ProjectContext) -> Self {
        use ProjectContext::*;
        match c {
            Proj(p) => p.as_ptr(),
            CurrentProject => null_mut(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NotificationBehavior {
    NotifyAll,
    NotifyAllExcept(ReaperControlSurface),
}

impl From<NotificationBehavior> for *mut raw::IReaperControlSurface {
    fn from(b: NotificationBehavior) -> Self {
        use NotificationBehavior::*;
        match b {
            NotifyAllExcept(s) => s.as_ptr(),
            NotifyAll => null_mut(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendTarget {
    HardwareOutput,
    OtherTrack(MediaTrack),
}

impl From<SendTarget> for *mut raw::MediaTrack {
    fn from(t: SendTarget) -> Self {
        use SendTarget::*;
        match t {
            HardwareOutput => null_mut(),
            OtherTrack(t) => t.as_ptr(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SectionContext<'a> {
    MainSection,
    // We need a reference because KbdSectionInfo can't be copied/cloned.
    Sec(&'a KbdSectionInfo),
}

impl<'a> From<SectionContext<'a>> for *mut raw::KbdSectionInfo {
    fn from(c: SectionContext<'a>) -> Self {
        use SectionContext::*;
        match c {
            MainSection => null_mut(),
            Sec(i) => i.0.as_ptr(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowContext {
    MainWindow,
    Win(Hwnd),
}

impl From<WindowContext> for HWND {
    fn from(c: WindowContext) -> Self {
        use WindowContext::*;
        match c {
            Win(h) => h.as_ptr(),
            MainWindow => null_mut(),
        }
    }
}
