use crate::{
    concat_c_strs, GaccelRegister, HookCommandFn, HookPostCommandFn, Hwnd, KbdSectionInfo,
    MediaTrack, MidiOutputDeviceId, ReaProject, ReaperControlSurface, ReaperStringArg,
    ToggleActionFn,
};
use c_str_macro::c_str;
use helgoboss_midi::{U14, U7};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use reaper_rs_low::raw;
use reaper_rs_low::raw::HWND;
use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_void;
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PluginRegistration<'a> {
    // TODO-low Refine all c_void's as soon as used
    Api(Cow<'a, CStr>, *mut c_void),
    ApiDef(Cow<'a, CStr>, *mut c_void),
    HookCommand(HookCommandFn),
    HookPostCommand(HookPostCommandFn),
    HookCommand2(*mut c_void),
    ToggleAction(ToggleActionFn),
    ActionHelp(*mut c_void),
    CommandId(Cow<'a, CStr>),
    CommandIdLookup(*mut c_void),
    // TODO-medium Maybe we should not expect the newtype here but the NonNull pointer. Expecting
    //  the newtype is more than we need (this might turn out inflexible in some situations,
    //  especially if the newtype is generic - see AudioHookRegister as example).
    Gaccel(GaccelRegister),
    CsurfInst(ReaperControlSurface),
    Custom(Cow<'a, CStr>, *mut c_void),
}

impl<'a> PluginRegistration<'a> {
    pub fn api(func_name: impl Into<ReaperStringArg<'a>>, func: *mut c_void) -> Self {
        Self::Api(func_name.into().into_inner(), func)
    }

    pub fn api_def(func_name: impl Into<ReaperStringArg<'a>>, func_def: *mut c_void) -> Self {
        Self::ApiDef(func_name.into().into_inner(), func_def)
    }

    pub fn custom(key: impl Into<ReaperStringArg<'a>>, info_struct: *mut c_void) -> Self {
        Self::Custom(key.into().into_inner(), info_struct)
    }

    pub(crate) fn to_owned(self) -> PluginRegistration<'static> {
        use PluginRegistration::*;
        match self {
            Api(func_name, func) => Api(func_name.into_owned().into(), func),
            ApiDef(func_name, func_def) => ApiDef(func_name.into_owned().into(), func_def),
            HookCommand(func) => HookCommand(func),
            HookPostCommand(func) => HookPostCommand(func),
            HookCommand2(func) => HookCommand2(func),
            ToggleAction(func) => ToggleAction(func),
            ActionHelp(info_struct) => ActionHelp(info_struct),
            CommandId(command_name) => CommandId(command_name.into_owned().into()),
            CommandIdLookup(info_struct) => CommandIdLookup(info_struct),
            Gaccel(reg) => Gaccel(reg),
            CsurfInst(inst) => CsurfInst(inst),
            Custom(key, info_struct) => Custom(key.into_owned().into(), info_struct),
        }
    }

    pub(crate) fn infostruct(&self) -> *mut c_void {
        use PluginRegistration::*;
        match self {
            Api(_, func) => *func,
            ApiDef(_, func_def) => *func_def,
            HookCommand(func) => *func as *mut c_void,
            HookPostCommand(func) => *func as *mut c_void,
            HookCommand2(func) => *func as *mut c_void,
            ToggleAction(func) => *func as *mut c_void,
            ActionHelp(info_struct) => *info_struct,
            CommandId(command_name) => command_name.as_ptr() as *mut c_void,
            CommandIdLookup(info_struct) => *info_struct,
            Gaccel(reg) => reg.get().as_ptr() as *mut c_void,
            CsurfInst(inst) => inst.get().as_ptr() as *mut c_void,
            Custom(_, info_struct) => *info_struct,
        }
    }
}

impl<'a> From<PluginRegistration<'a>> for Cow<'a, CStr> {
    fn from(value: PluginRegistration<'a>) -> Self {
        use PluginRegistration::*;
        match value {
            Api(func_name, _) => concat_c_strs(c_str!("API_"), func_name.as_ref()).into(),
            ApiDef(func_name, _) => concat_c_strs(c_str!("APIdef_"), func_name.as_ref()).into(),
            HookCommand(_) => c_str!("hookcommand").into(),
            HookPostCommand(_) => c_str!("hookpostcommand").into(),
            HookCommand2(_) => c_str!("hookcommand2").into(),
            ToggleAction(_) => c_str!("toggleaction").into(),
            ActionHelp(_) => c_str!("action_help").into(),
            CommandId(_) => c_str!("command_id").into(),
            CommandIdLookup(_) => c_str!("command_id_lookup").into(),
            Gaccel(_) => c_str!("gaccel").into(),
            CsurfInst(_) => c_str!("csurf_inst").into(),
            Custom(key, _) => key,
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
            NotifyAllExcept(s) => s.get().as_ptr(),
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
    NoWindow,
    Win(Hwnd),
}

impl From<WindowContext> for HWND {
    fn from(c: WindowContext) -> Self {
        use WindowContext::*;
        match c {
            Win(h) => h.as_ptr(),
            NoWindow => null_mut(),
        }
    }
}
