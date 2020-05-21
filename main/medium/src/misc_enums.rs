use crate::{
    Hwnd, KbdSectionInfo, MediaTrack, MidiOutputDeviceId, ReaProject, ReaperStringArg,
    TryFromRawError,
};
use c_str_macro::c_str;

use helgoboss_midi::{U14, U7};
use reaper_low::raw;
use std::borrow::Cow;
use std::convert::TryInto;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::{null_mut, NonNull};

/// Determines the behavior when adding an FX.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AddFxBehavior {
    /// Adds the FX only if it hasn't been found in the FX chain.
    AddIfNotFound,
    /// Adds the FX even if it already exists in the FX chain.
    AlwaysAdd,
}

impl From<AddFxBehavior> for FxAddByNameBehavior {
    fn from(b: AddFxBehavior) -> FxAddByNameBehavior {
        use AddFxBehavior::*;
        match b {
            AddIfNotFound => FxAddByNameBehavior::AddIfNotFound,
            AlwaysAdd => FxAddByNameBehavior::AlwaysAdd,
        }
    }
}

/// Represents the type of a track FX chain.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackFxChainType {
    /// The normal (or output) FX chain.
    NormalFxChain,
    /// The input (or recording) FX chain.
    ///
    /// On the master track this corresponds to the monitoring FX chain.
    InputFxChain,
}

/// Determines how to deal with the master track.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MasterTrackBehavior {
    /// Without master track.
    ExcludeMasterTrack,
    /// With master track.
    IncludeMasterTrack,
}

/// A performance/caching hint which determines how REAPER internally gets or sets a chunk.
///
/// Has implications on both performance and chunk content.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ChunkCacheHint {
    /// This takes longer but is the best choice for most situations.
    NormalMode,
    /// This can be faster but has some drawbacks.
    ///
    /// The following happens when using this mode:
    ///
    /// - When getting a chunk, then FX configurations are cached, e.g. if the plug-in hasn't
    ///   recently notified REAPER of a parameter change, the last configuration state is returned
    ///   (which is faster). The downside is if the plug-in doesn't properly report its state as
    ///   having changed, one wouldn't get the latest version.
    /// - When setting a chunk, envelope lane sizes will not be updated from the configuration
    ///   state.
    /// - The format in which FX GUIDs are encoded is slightly different in this mode (to
    ///   facilitate more efficient re-use of existing plug-in instances).
    /// - The logic in saving the event data for pooled MIDI items is slightly different (in undo
    ///   mode only one of the items in the pool will encode, in normal mode the first instance in
    ///   the chunk will get the data).
    UndoMode,
}

/// Represents a change of a value (e.g. of a parameter).
#[derive(Clone, PartialEq, Debug)]
pub enum ValueChange<T: Copy + Into<f64>> {
    /// Sets the given value absolutely.
    Absolute(T),
    /// Increments or decrements the current value by the given amount.
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

/// Determines whether to create an undo point.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum UndoBehavior {
    /// Doesn't create an undo point.
    OmitUndoPoint,
    /// Creates an undo point.
    AddUndoPoint,
}

/// Determines whether to copy or move something (e.g. an FX).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TransferBehavior {
    /// Copies the thing.
    Copy,
    /// Moves the thing.
    Move,
}

/// Determines how track defaults should be used.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackDefaultsBehavior {
    /// Creates the track without default envelopes and FX.
    ///
    /// Other kinds of track defaults will be applied though!
    OmitDefaultEnvAndFx,
    /// Creates the track with default envelopes and FX.
    AddDefaultEnvAndFx,
}

/// Determines the gang behavior.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GangBehavior {
    /// Change will affect the targeted track only.
    DenyGang,
    /// Change will affect all selected tracks.
    AllowGang,
}

/// Defines whether a track is armed for recording.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum RecordArmMode {
    /// Track is not armed for recording.
    Unarmed,
    /// Track is armed for recording.
    Armed,
}

impl RecordArmMode {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use RecordArmMode::*;
        match self {
            Unarmed => 0,
            Armed => 1,
        }
    }
}

/// Determines if and how to show/hide a FX user interface.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FxShowInstruction {
    /// Closes the complete FX chain.
    HideChain,
    /// Shows the complete FX chain and makes the given FX visible.
    ShowChain(TrackFxLocation),
    /// Closes the floating FX window.
    HideFloatingWindow(TrackFxLocation),
    /// Shows the floating FX window.
    ShowFloatingWindow(TrackFxLocation),
}

impl FxShowInstruction {
    /// Converts the instruction part of this value to a `showFlag` integer as expected by the
    /// low-level API.
    pub fn instruction_to_raw(&self) -> i32 {
        use FxShowInstruction::*;
        match self {
            HideChain => 0,
            ShowChain(_) => 1,
            HideFloatingWindow(_) => 2,
            ShowFloatingWindow(_) => 3,
        }
    }

    /// Converts the FX location part of this value to an integer as expected by the low-level API.
    pub fn location_to_raw(&self) -> i32 {
        use FxShowInstruction::*;
        match self {
            HideChain => 0,
            ShowChain(l) => l.to_raw(),
            HideFloatingWindow(l) => l.to_raw(),
            ShowFloatingWindow(l) => l.to_raw(),
        }
    }
}

/// Defines whether you are referring to a send or a receive.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackSendDirection {
    /// You are referring to a receive (a send from the other track's perspective).
    Receive,
    /// Refers to a track send (a receive from the other track's perspective).
    Send,
}

/// Defines the kind of link.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackSendCategory {
    /// A receive from another track (a send from that other track's perspective).
    Receive = -1,
    /// A send to another track (a receive from that other track's perspective).
    Send = 0,
    /// A hardware output.
    HardwareOutput = 1,
}

impl TrackSendCategory {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use TrackSendCategory::*;
        match self {
            Receive => -1,
            Send => 0,
            HardwareOutput => 1,
        }
    }
}

impl From<TrackSendDirection> for TrackSendCategory {
    fn from(v: TrackSendDirection) -> TrackSendCategory {
        use TrackSendDirection::*;
        match v {
            Receive => TrackSendCategory::Receive,
            Send => TrackSendCategory::Send,
        }
    }
}

/// Determines where to route a MIDI message.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum StuffMidiMessageTarget {
    /// Routes the message to REAPER's virtual MIDI keyboard.
    VirtualMidiKeyboardQueue,
    /// Routes the message to REAPER's control path.
    ///
    /// That means it can be used in turn to control actions, FX parameters and so on.
    MidiAsControlInputQueue,
    /// Routes the message to REAPER's virtual MIDI keyboard on its current channel.
    VirtualMidiKeyboardQueueOnCurrentChannel,
    /// Sends the message directly to an external MIDI device.
    MidiOutputDevice(MidiOutputDeviceId),
}

impl StuffMidiMessageTarget {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use StuffMidiMessageTarget::*;
        match self {
            VirtualMidiKeyboardQueue => 0,
            MidiAsControlInputQueue => 1,
            VirtualMidiKeyboardQueueOnCurrentChannel => 2,
            MidiOutputDevice(id) => 16 + id.0 as i32,
        }
    }
}

/// Describes the current location of a track FX (assuming the track is already known).
///
/// This is not a stable identifier because track FX locations can change!
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackFxLocation {
    /// FX index in the normal FX chain.
    NormalFxChain(u32),
    /// FX index in the input FX chain.
    ///
    /// On the master track (if applicable) this represents an index in the monitoring FX chain.
    InputFxChain(u32),
}

impl TrackFxLocation {
    /// Converts an integer as returned by the low-level API to a track FX location.
    pub fn try_from_raw(v: i32) -> Result<TrackFxLocation, TryFromRawError<i32>> {
        use TrackFxLocation::*;
        let v: u32 = v
            .try_into()
            .map_err(|_| TryFromRawError::new("FX index shouldn't be negative", v))?;
        let result = if v >= 0x0100_0000 {
            InputFxChain(v - 0x0100_0000)
        } else {
            NormalFxChain(v)
        };
        Ok(result)
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use TrackFxLocation::*;
        let positive = match self {
            InputFxChain(idx) => 0x0100_0000 + idx,
            NormalFxChain(idx) => idx,
        };
        positive as i32
    }
}

/// Determines the behavior when adding or querying FX.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum FxAddByNameBehavior {
    /// Adds the FX even if it already exists in the FX chain.
    AlwaysAdd,
    /// Just queries the FX location.
    Query,
    /// Adds the FX if it hasn't been found in the FX chain.
    AddIfNotFound,
}

impl FxAddByNameBehavior {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use FxAddByNameBehavior::*;
        match self {
            AlwaysAdd => -1,
            Query => 0,
            AddIfNotFound => 1,
        }
    }
}

/// Represents a value change targeted to a REAPER action.
///
/// This uses typical MIDI types (7-bit and 14-bit values) because this is supposed
/// to be used for actions which are controllable via MIDI.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ActionValueChange {
    /// Sets the given value absolutely using a low resolution (128 possible values).
    AbsoluteLowRes(U7),
    /// Sets the given value absolutely using a high resolution (16384 different values).
    AbsoluteHighRes(U14),
    /// Increments or decrements the current value using REAPER's CC mode "Relative 1".
    ///
    /// - 127 → -1
    /// - 1 → +1
    Relative1(U7),
    /// Increments or decrements the current value using REAPER's CC mode "Relative 2".
    ///
    /// - 63 → -1
    /// - 65 → +1
    Relative2(U7),
    /// Increments or decrements the current value using REAPER's CC mode "Relative 3".
    ///
    /// - 65 → -1
    /// - 1 → +1
    Relative3(U7),
}

/// A thing that you can register at REAPER.
// TODO-low "Unlock" all uncommented variants as soon as appropriate types are clear
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum RegistrationObject<'a> {
    // Api(Cow<'a, CStr>, *mut c_void),
    // ApiDef(Cow<'a, CStr>, *mut c_void),
    /// A hook command.
    ///
    /// Extract from `reaper_plugin_functions.h`:
    ///
    /// <pre>
    /// another thing you can register is "hookcommand", which you pass a callback:
    ///  NON_API: bool runCommand(int command, int flag);
    ///           register("hookcommand",runCommand);
    /// which returns TRUE to eat (process) the command.
    /// flag is usually 0 but can sometimes have useful info depending on the message.
    /// note: it's OK to call Main_OnCommand() within your runCommand, however you MUST check for
    /// recursion if doing so! > in fact, any use of this hook should benefit from a simple
    /// reentrancy test...
    /// </pre>
    HookCommand(raw::HookCommandFn),
    /// A hook post command.
    ///
    /// Extract from `reaper_plugin_functions.h`:
    ///
    /// <pre>
    /// to get notified when an action of the main section is performed,
    /// you can register "hookpostcommand", which you pass a callback:
    ///  NON_API: void postCommand(int command, int flag);
    ///           register("hookpostcommand",postCommand);
    /// </pre>
    HookPostCommand(raw::HookPostCommandFn),
    // HookCommand2(*mut c_void),
    /// A toggle action.
    ///
    /// Extract from `reaper_plugin.h`:
    ///
    /// <pre>
    /// register("toggleaction", toggleactioncallback) lets you register a callback function
    /// that is called to check if an action registered by an extension has an on/off state.
    ///
    /// callback function:
    ///   int toggleactioncallback(int command_id);
    ///
    /// return:
    ///   -1=action does not belong to this extension, or does not toggle
    ///   0=action belongs to this extension and is currently set to "off"
    ///   1=action belongs to this extension and is currently set to "on"
    /// </pre>
    ToggleAction(raw::ToggleActionFn),
    // ActionHelp(*mut c_void),
    /// A command ID for the given command name.
    ///
    /// Extract from `reaper_plugin_functions.h`:
    /// <pre>
    /// you can also register command IDs for actions,
    /// register with "command_id", parameter is a unique string with only A-Z, 0-9,
    /// returns command ID (or 0 if not supported/out of actions)
    /// </pre>
    CommandId(Cow<'a, CStr>),
    // CommandIdLookup(*mut c_void),
    /// An action description and shortcut.
    ///
    /// Extract from `reaper_plugin.h`:
    /// <pre>
    /// gaccel_register_t allows you to register ("gaccel") an action into the main keyboard
    /// section action list, and at the same time a default binding for it (accel.cmd is the
    /// command ID, desc is the description, and accel's other parameters are the key to bind.
    /// </pre>
    Gaccel(NonNull<raw::gaccel_register_t>),
    /// A hidden control surface (useful for being notified by REAPER about events).
    ///
    /// Extract from `reaper_plugin.h`:
    ///
    /// <pre>
    /// note you can also add a control surface behind the scenes with "csurf_inst"
    /// (IReaperControlSurface*)instance
    /// </pre>
    CsurfInst(NonNull<raw::IReaperControlSurface>),
    /// If a variant is missing in this enum, you can use this custom one as a resort.
    ///
    /// Use [`custom()`] to create this variant.
    ///
    /// [`custom()`]: #method.custom
    Custom(Cow<'a, CStr>, *mut c_void),
}

impl<'a> RegistrationObject<'a> {
    // pub fn api(func_name: impl Into<ReaperStringArg<'a>>, func: *mut c_void) -> Self {
    //     Self::Api(func_name.into().into_inner(), func)
    // }
    //
    // pub fn api_def(func_name: impl Into<ReaperStringArg<'a>>, func_def: *mut c_void) -> Self {
    //     Self::ApiDef(func_name.into().into_inner(), func_def)
    // }

    /// Convenience function for creating a [`Custom`] registration object.
    ///
    /// [`Custom`]: #variant.Custom
    pub fn custom(
        key: impl Into<ReaperStringArg<'a>>,
        info_struct: *mut c_void,
    ) -> RegistrationObject<'a> {
        RegistrationObject::Custom(key.into().into_inner(), info_struct)
    }

    pub(crate) fn into_owned(self) -> RegistrationObject<'static> {
        use RegistrationObject::*;
        match self {
            // Api(func_name, func) => Api(func_name.into_owned().into(), func),
            // ApiDef(func_name, func_def) => ApiDef(func_name.into_owned().into(), func_def),
            HookCommand(func) => HookCommand(func),
            HookPostCommand(func) => HookPostCommand(func),
            // HookCommand2(func) => HookCommand2(func),
            ToggleAction(func) => ToggleAction(func),
            // ActionHelp(info_struct) => ActionHelp(info_struct),
            CommandId(command_name) => CommandId(command_name.into_owned().into()),
            // CommandIdLookup(info_struct) => CommandIdLookup(info_struct),
            Gaccel(reg) => Gaccel(reg),
            CsurfInst(inst) => CsurfInst(inst),
            Custom(key, info_struct) => Custom(key.into_owned().into(), info_struct),
        }
    }

    pub(crate) fn key_into_raw(self) -> Cow<'a, CStr> {
        use RegistrationObject::*;
        match self {
            // Api(func_name, _) => concat_c_strs(c_str!("API_"), func_name.as_ref()).into(),
            // ApiDef(func_name, _) => concat_c_strs(c_str!("APIdef_"), func_name.as_ref()).into(),
            HookCommand(_) => c_str!("hookcommand").into(),
            HookPostCommand(_) => c_str!("hookpostcommand").into(),
            // HookCommand2(_) => c_str!("hookcommand2").into(),
            ToggleAction(_) => c_str!("toggleaction").into(),
            // ActionHelp(_) => c_str!("action_help").into(),
            CommandId(_) => c_str!("command_id").into(),
            // CommandIdLookup(_) => c_str!("command_id_lookup").into(),
            Gaccel(_) => c_str!("gaccel").into(),
            CsurfInst(_) => c_str!("csurf_inst").into(),
            Custom(key, _) => key,
        }
    }

    pub(crate) fn ptr_to_raw(&self) -> *mut c_void {
        use RegistrationObject::*;
        match self {
            // Api(_, func) => *func,
            // ApiDef(_, func_def) => *func_def,
            HookCommand(func) => *func as *mut c_void,
            HookPostCommand(func) => *func as *mut c_void,
            // HookCommand2(func) => *func as *mut c_void,
            ToggleAction(func) => *func as *mut c_void,
            // ActionHelp(info_struct) => *info_struct,
            CommandId(command_name) => command_name.as_ptr() as *mut c_void,
            // CommandIdLookup(info_struct) => *info_struct,
            Gaccel(reg) => reg.as_ptr() as *mut c_void,
            CsurfInst(inst) => inst.as_ptr() as *mut c_void,
            Custom(_, info_struct) => *info_struct,
        }
    }
}

/// Type and location of a certain track.
/// TODO-medium Rename to TrackLocation?
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackRef {
    /// The master track of a project.
    MasterTrack,
    /// Index of a normal track.
    NormalTrack(u32),
}

/// Describes whether and how the recording input is monitored.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum InputMonitoringMode {
    /// No input monitoring.
    Off,
    /// Monitoring happens always.
    Normal,
    /// Monitoring only happens when playing (tape style).
    NotWhenPlaying,
}

impl InputMonitoringMode {
    /// Converts an integer as returned by the low-level API to an input monitoring mode.
    pub fn try_from_raw(v: i32) -> Result<InputMonitoringMode, TryFromRawError<i32>> {
        use InputMonitoringMode::*;
        match v {
            0 => Ok(Off),
            1 => Ok(Normal),
            2 => Ok(NotWhenPlaying),
            _ => Err(TryFromRawError::new(
                "couldn't convert to input monitoring mode",
                v,
            )),
        }
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use InputMonitoringMode::*;
        match self {
            Off => 0,
            Normal => 1,
            NotWhenPlaying => 2,
        }
    }
}
/// Something which refers to a certain project.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ProjectRef {
    /// Project in the currently open tab.
    Current,
    /// Project which is currently rendering (if there is one).
    CurrentlyRendering,
    /// Project at the given tab index.
    Tab(u32),
}

impl ProjectRef {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use ProjectRef::*;
        match self {
            Current => -1,
            CurrentlyRendering => 0x4000_0000,
            Tab(i) => i as i32,
        }
    }
}

/// Something which refers to a certain FX preset.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FxPresetRef {
    /// Factory preset for that FX.
    FactoryPreset,
    /// Default user preset for that FX.
    DefaultUserPreset,
    /// Preset at the given index.
    Preset(u32),
}

impl FxPresetRef {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use FxPresetRef::*;
        match self {
            FactoryPreset => -2,
            DefaultUserPreset => -1,
            Preset(idx) => idx as i32,
        }
    }
}

/// Determines the project in which a function should be executed.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ProjectContext {
    /// Project in the currently open tab.
    CurrentProject,
    /// A particular project, not necessarily the one in the currently open tab.
    Proj(ReaProject),
}

impl ProjectContext {
    /// Converts this value to a raw pointer as expected by the low-level API.
    pub fn to_raw(self) -> *mut raw::ReaProject {
        use ProjectContext::*;
        match self {
            Proj(p) => p.as_ptr(),
            CurrentProject => null_mut(),
        }
    }
}

/// Determines which control surfaces will be informed.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum NotificationBehavior {
    /// All registered control surfaces.
    NotifyAll,
    /// All registered control surfaces except the given one.
    NotifyAllExcept(NonNull<raw::IReaperControlSurface>),
}

impl NotificationBehavior {
    /// Converts this value to a raw pointer as expected by the low-level API.
    pub fn to_raw(self) -> *mut raw::IReaperControlSurface {
        use NotificationBehavior::*;
        match self {
            NotifyAllExcept(s) => s.as_ptr(),
            NotifyAll => null_mut(),
        }
    }
}

/// Denotes the target of a send.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SendTarget {
    /// A hardware output created with default properties.
    HardwareOutput,
    /// Another track.
    OtherTrack(MediaTrack),
}

impl SendTarget {
    /// Converts this value to a raw pointer as expected by the low-level API.
    pub fn to_raw(self) -> *mut raw::MediaTrack {
        use SendTarget::*;
        match self {
            HardwareOutput => null_mut(),
            OtherTrack(t) => t.as_ptr(),
        }
    }
}

/// Determines the section in which an action is located.
///
/// Command IDs are not globally unique. They are only unique within a particular section.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SectionContext<'a> {
    /// The main section.
    MainSection,
    /// A particular section, not necessarily the main section.
    Sec(&'a KbdSectionInfo),
}

impl<'a> SectionContext<'a> {
    /// Converts this value to a raw pointer as expected by the low-level API.
    pub fn to_raw(self) -> *mut raw::KbdSectionInfo {
        use SectionContext::*;
        match self {
            MainSection => null_mut(),
            Sec(i) => i.0.as_ptr(),
        }
    }
}

/// Allows one to pass a window handle to the action function.
///
/// The concrete meaning of this depends on the action. For many actions this is not relevant at
/// all.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum WindowContext {
    /// Don't pass any window handle.
    NoWindow,
    /// Pass the given window handle.
    Win(Hwnd),
}

impl WindowContext {
    /// Converts this value to a raw pointer as expected by the low-level API.
    pub fn to_raw(self) -> raw::HWND {
        use WindowContext::*;
        match self {
            Win(h) => h.as_ptr(),
            NoWindow => null_mut(),
        }
    }
}
