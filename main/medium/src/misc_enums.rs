use crate::{
    BookmarkId, CommandId, Hidden, Hwnd, KbdSectionInfo, MediaTrack, MidiFrameOffset,
    MidiOutputDeviceId, ReaProject, ReaperPanValue, ReaperStr, ReaperStringArg, ReaperWidthValue,
};

use crate::util::concat_reaper_strs;
use helgoboss_midi::{U14, U7};
use reaper_low::raw;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::num::NonZeroU32;
use std::os::raw::{c_char, c_void};
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

/// Describes which kind of time range we are talking about in a REAPER project.
///
/// They are linked by default in REAPER so users might not even be aware that there's a
/// difference, but there is.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TimeRangeType {
    /// The loop points (displayed in the ruler).
    LoopPoints,
    /// The time selection (visualized with different background color in the arrange view).
    TimeSelection,
}

/// Describes whether to allow auto-seek or not.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AutoSeekBehavior {
    /// Prevents auto-seek from happening when setting loop points.
    DenyAutoSeek,
    /// Allows auto-seek to happen when setting loop points.
    AllowAutoSeek,
}

/// Determines how to deal with the master track.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MasterTrackBehavior {
    /// Without master track.
    ExcludeMasterTrack,
    /// With master track.
    IncludeMasterTrack,
}

/// Something which refers to a certain marker or region.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum BookmarkRef {
    /// Counts only regions or only markers depending on the usage context.
    Position(NonZeroU32),
    /// Relates only to regions or only to markers depending on the usage context.
    Id(BookmarkId),
}

impl BookmarkRef {
    pub(crate) fn to_raw(self) -> i32 {
        use BookmarkRef::*;
        match self {
            Position(i) => i.get() as _,
            Id(id) => id.get() as _,
        }
    }

    pub(crate) fn uses_timeline_order(&self) -> bool {
        matches!(self, BookmarkRef::Position(_))
    }
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

/// Determines whether to import MIDI as in-project MIDI events or not.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MidiImportBehavior {
    /// Uses the relevant REAPER preference.
    UsePreference,
    /// Makes sure the MIDI data is not imported as in-project MIDI events.
    ForceNoMidiImport,
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

/// Defines whether to align with measure starts when playing previews.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MeasureAlignment {
    /// Plays immediately.
    PlayImmediately,
    /// Aligns playback with measure start.
    AlignWithMeasureStart,
}

impl MeasureAlignment {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> f64 {
        use MeasureAlignment::*;
        match self {
            PlayImmediately => -1.0,
            AlignWithMeasureStart => 1.0,
        }
    }
}

/// Determines if and how to show/hide a FX user interface.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FxShowInstruction {
    /// Closes the complete FX chain.
    HideChain(TrackFxChainType),
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
            HideChain(_) => 0,
            ShowChain(_) => 1,
            HideFloatingWindow(_) => 2,
            ShowFloatingWindow(_) => 3,
        }
    }

    /// Converts the FX location part of this value to an integer as expected by the low-level API.
    pub fn location_to_raw(&self) -> i32 {
        use FxShowInstruction::*;
        match self {
            HideChain(t) => {
                let dummy_location = match t {
                    TrackFxChainType::NormalFxChain => TrackFxLocation::NormalFxChain(0),
                    TrackFxChainType::InputFxChain => TrackFxLocation::InputFxChain(0),
                };
                dummy_location.to_raw()
            }
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

/// Defines the kind of route.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackSendCategory {
    /// A receive from another track (a send from that other track's perspective).
    Receive = -1,
    /// A send to another track (a receive from that other track's perspective).
    Send = 0,
    /// A send to a hardware output.
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

/// Defines an edit mode for changing send volume or pan.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum EditMode {
    /// An instant edit such as reset via double-clicking a fader or typing a value in an edit
    /// field.
    InstantEdit = -1,
    /// A normal tweak just like when dragging the mouse.
    NormalTweak = 0,
    /// Marks the end of an edit (mouse up).
    EndOfEdit = 1,
}

impl EditMode {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use EditMode::*;
        match self {
            InstantEdit => -1,
            NormalTweak => 0,
            EndOfEdit => 1,
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

/// Reference to a track send, hardware output send or track receive.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackSendRef {
    /// A receive from another track (a send from that other track's perspective).
    Receive(u32),
    /// A send to another track (a receive from that other track's perspective) or a send to a
    /// hardware output.
    Send(u32),
}

impl TrackSendRef {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use TrackSendRef::*;
        match self {
            Receive(i) => -((i + 1) as i32),
            Send(i) => i as _,
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
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
}

impl TrackFxLocation {
    /// Converts an integer as returned by the low-level API to a track FX location.
    pub fn from_raw(v: i32) -> TrackFxLocation {
        use TrackFxLocation::*;
        if let Ok(v) = u32::try_from(v) {
            if v >= 0x0100_0000 {
                InputFxChain(v - 0x0100_0000)
            } else {
                NormalFxChain(v)
            }
        } else {
            Unknown(Hidden(v))
        }
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use TrackFxLocation::*;
        let positive = match self {
            InputFxChain(idx) => 0x0100_0000 + idx,
            NormalFxChain(idx) => idx,
            Unknown(Hidden(x)) => return x,
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
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<(i32, i32, i32)>),
}

impl ActionValueChange {
    /// Converts this value to the (val, valhw, relmode) triple expected by the low-level API.
    pub(crate) fn to_raw(self) -> (i32, i32, i32) {
        use ActionValueChange::*;
        match self {
            AbsoluteLowRes(v) => (i32::from(v), -1, 0),
            AbsoluteHighRes(v) => (
                ((u32::from(v) >> 7) & 0x7f) as i32,
                (u32::from(v) & 0x7f) as i32,
                0,
            ),
            Relative1(v) => (i32::from(v), -1, 1),
            Relative2(v) => (i32::from(v), -1, 2),
            Relative3(v) => (i32::from(v), -1, 3),
            Unknown(Hidden((a, b, c))) => (a, b, c),
        }
    }

    /// Converts the given low-level API values to this action value change if possible.
    pub(crate) fn from_raw(raw: (i32, i32, i32)) -> ActionValueChange {
        let (val, valhw, relmode) = raw;
        use ActionValueChange::*;
        if let Ok(val) = U7::try_from(val) {
            match (valhw, relmode) {
                (-1, 0) | (-1, 1) | (-1, 2) | (-1, 3) => match relmode {
                    0 => AbsoluteLowRes(val),
                    1 => Relative1(val),
                    2 => Relative2(val),
                    3 => Relative3(val),
                    _ => Unknown(Hidden((raw.0, raw.1, raw.2))),
                },
                (valhw, 0) if valhw >= 0 => {
                    if let Ok(valhw) = U7::try_from(valhw) {
                        let combined = (valhw.get() << 7) | val.get();
                        AbsoluteHighRes(combined.into())
                    } else {
                        Unknown(Hidden((raw.0, raw.1, raw.2)))
                    }
                }
                _ => Unknown(Hidden((raw.0, raw.1, raw.2))),
            }
        } else {
            Unknown(Hidden((raw.0, raw.1, raw.2)))
        }
    }
}

/// A thing that you can register at REAPER.
// TODO-low "Unlock" all uncommented variants as soon as appropriate types are clear
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum RegistrationObject<'a> {
    /// A function that you want to expose to other extensions or plug-ins.
    ///
    /// Extract from `reaper_plugin_functions.h`:
    ///
    /// <pre>
    /// if you have a function called myfunction(..) that you want to expose to other extensions or
    /// plug-ins, use register("API_myfunction",funcaddress), and "-API_myfunction" to remove.
    /// Other extensions then use GetFunc("myfunction") to get the function pointer.
    /// REAPER will also export the function address to ReaScript, so your extension could supply
    /// a Python module that provides a wrapper called RPR_myfunction(..).
    /// </pre>
    Api(Cow<'a, ReaperStr>, *mut c_void),
    /// A function definition that describes a function registered via [`Api`].
    ///
    /// Extract from `reaper_plugin_functions.h`:
    ///
    /// <pre>
    /// register("APIdef_myfunction",defstring) will include your function declaration and help
    /// in the auto-generated REAPER API header and ReaScript documentation.
    /// defstring is four null-separated fields: return type, argument types, argument names, and
    /// help. Example: double myfunction(char* str, int flag) would have
    /// defstring="double\0char*,int\0str,flag\0help text for myfunction"
    /// </pre>
    /// [`Api`]: #variant.Api
    ApiDef(Cow<'a, ReaperStr>, *const c_char),
    /// A var-arg function for exposing a function to ReaScript.
    // TODO-medium Documentation
    ApiVararg(Cow<'a, ReaperStr>, raw::ApiVararg),
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
    HookCommand(raw::HookCommand),
    /// A hook command that supports MIDI CC/mousewheel actions.
    ///
    /// Extract from `reaper_plugin_functions.h`:
    ///
    /// <pre>
    /// you can also register "hookcommand2", which you pass a callback:
    ///  NON_API: bool onAction(KbdSectionInfo *sec, int command, int val, int valhw, int relmode,
    /// HWND hwnd);           register("hookcommand2",onAction);
    /// which returns TRUE to eat (process) the command.
    /// val/valhw are used for actions learned with MIDI/OSC.
    /// val = [0..127] and valhw = -1 for MIDI CC,
    /// valhw >=0 for MIDI pitch or OSC with value = (valhw|val<<7)/16383.0,
    /// relmode absolute(0) or 1/2/3 for relative adjust modes
    /// </pre>
    HookCommand2(raw::HookCommand2),
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
    HookPostCommand(raw::HookPostCommand),
    /// A hook post command 2.
    HookPostCommand2(raw::HookPostCommand2),
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
    ToggleAction(raw::ToggleAction),
    // ActionHelp(*mut c_void),
    /// A command ID for the given command name.
    ///
    /// Extract from `reaper_plugin_functions.h`:
    /// <pre>
    /// you can also register command IDs for actions,
    /// register with "command_id", parameter is a unique string with only A-Z, 0-9,
    /// returns command ID (or 0 if not supported/out of actions)
    /// </pre>
    CommandId(*const c_char),
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
    Custom(Cow<'a, ReaperStr>, *mut c_void),
}

impl<'a> RegistrationObject<'a> {
    /// Convenience function for creating an [`Api`] registration object.
    ///
    /// [`Api`]: #variant.Api
    pub fn api(
        func_name: impl Into<ReaperStringArg<'a>>,
        func: *mut c_void,
    ) -> RegistrationObject<'a> {
        RegistrationObject::Api(func_name.into().into_inner(), func)
    }

    /// Convenience function for creating an [`ApiDef`] registration object.
    ///
    /// [`ApiDef`]: #variant.ApiDef
    pub fn api_def(
        func_name: impl Into<ReaperStringArg<'a>>,
        func_def: *const c_char,
    ) -> RegistrationObject<'a> {
        RegistrationObject::ApiDef(func_name.into().into_inner(), func_def)
    }

    /// Convenience function for creating an [`ApiVararg`] registration object.
    ///
    /// [`ApiVararg`]: #variant.ApiVararg
    pub fn api_vararg(
        func_name: impl Into<ReaperStringArg<'a>>,
        func: raw::ApiVararg,
    ) -> RegistrationObject<'a> {
        RegistrationObject::ApiVararg(func_name.into().into_inner(), func)
    }

    /// Convenience function for creating a [`Custom`] registration object.
    ///
    /// [`Custom`]: #variant.Custom
    pub fn custom(
        key: impl Into<ReaperStringArg<'a>>,
        info_struct: *mut c_void,
    ) -> RegistrationObject<'a> {
        RegistrationObject::Custom(key.into().into_inner(), info_struct)
    }

    /// Returns the values which need to be passed to `plugin_register()`.
    pub(crate) fn into_raw(self) -> PluginRegistration {
        use RegistrationObject::*;
        match self {
            Api(func_name, func) => PluginRegistration {
                key: concat_reaper_strs(reaper_str!("API_"), func_name.as_ref()).into(),
                value: func,
            },
            ApiDef(func_name, func_def) => PluginRegistration {
                key: concat_reaper_strs(reaper_str!("APIdef_"), func_name.as_ref()).into(),
                value: func_def as _,
            },
            ApiVararg(func_name, func) => PluginRegistration {
                key: concat_reaper_strs(reaper_str!("APIvararg_"), func_name.as_ref()).into(),
                value: func as _,
            },
            HookCommand(func) => PluginRegistration {
                key: reaper_str!("hookcommand").into(),
                value: func as _,
            },
            HookCommand2(func) => PluginRegistration {
                key: reaper_str!("hookcommand2").into(),
                value: func as _,
            },
            HookPostCommand(func) => PluginRegistration {
                key: reaper_str!("hookpostcommand").into(),
                value: func as _,
            },
            HookPostCommand2(func) => PluginRegistration {
                key: reaper_str!("hookpostcommand2").into(),
                value: func as _,
            },
            ToggleAction(func) => PluginRegistration {
                key: reaper_str!("toggleaction").into(),
                value: func as _,
            },
            CommandId(command_name) => PluginRegistration {
                key: reaper_str!("command_id").into(),
                value: command_name as _,
            },
            Gaccel(reg) => PluginRegistration {
                key: reaper_str!("gaccel").into(),
                value: reg.as_ptr() as _,
            },
            CsurfInst(inst) => PluginRegistration {
                key: reaper_str!("csurf_inst").into(),
                value: inst.as_ptr() as _,
            },
            Custom(key, value) => PluginRegistration {
                key: key.into_owned().into(),
                value,
            },
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct PluginRegistration {
    /// Please note that this is an owned string, not just a pointer. This owned string needs
    /// to be kept around. During the time of the registration it must not be removed from memory.
    /// In most cases this is no problem because keys like "hookcommand" are just static REAPER
    /// strings and never disappear. But "API_myfunction" is not static because it's a string which
    /// is assembled at runtime (in this function).
    pub(crate) key: Cow<'static, ReaperStr>,
    /// The data where this points to needs to be kept somewhere. It must not be removed from
    /// memory during the time of the registration.
    pub(crate) value: *mut c_void,
}

/// Type and location of a certain track.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TrackLocation {
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
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
}

impl InputMonitoringMode {
    /// Converts an integer as returned by the low-level API to an input monitoring mode.
    pub fn from_raw(v: i32) -> InputMonitoringMode {
        use InputMonitoringMode::*;
        match v {
            0 => Off,
            1 => Normal,
            2 => NotWhenPlaying,
            x => Unknown(Hidden(x)),
        }
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use InputMonitoringMode::*;
        match self {
            Off => 0,
            Normal => 1,
            NotWhenPlaying => 2,
            Unknown(Hidden(x)) => x,
        }
    }
}

/// Track solo mode.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SoloMode {
    Off,
    SoloIgnoreRouting,
    SoloInPlace,
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
}

impl SoloMode {
    /// Converts an integer as returned by the low-level API to a solo mode.
    pub fn from_raw(v: i32) -> SoloMode {
        use SoloMode::*;
        match v {
            0 => Off,
            1 => SoloIgnoreRouting,
            2 => SoloInPlace,
            x => Unknown(Hidden(x)),
        }
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use SoloMode::*;
        match self {
            Off => 0,
            SoloIgnoreRouting => 1,
            SoloInPlace => 2,
            Unknown(Hidden(x)) => x,
        }
    }
}

/// Information about visibility of an FX chain.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FxChainVisibility {
    /// FX chain is not visible.
    Hidden,
    /// FX chain is visible.
    ///
    /// If the argument is `Some`, the FX with that index is selected.
    Visible(Option<u32>),
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a
    /// variant that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
}

impl FxChainVisibility {
    /// Converts an integer as returned by the low-level API to an FX chain visibility.
    pub fn from_raw(v: i32) -> FxChainVisibility {
        match v {
            -2 => Self::Visible(None),
            -1 => Self::Hidden,
            x if x >= 0 => Self::Visible(Some(x as u32)),
            x => Self::Unknown(Hidden(x)),
        }
    }
}

/// Track pan mode.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PanMode {
    /// Classic v1 - v3.
    BalanceV1,
    /// Balance v4+.
    BalanceV4,
    /// Stereo pan.
    StereoPan,
    /// Dual pan.
    DualPan,
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
}

impl PanMode {
    /// Converts an integer as returned by the low-level API to a pan mode.
    pub fn from_raw(v: i32) -> PanMode {
        use PanMode::*;
        match v {
            0 => BalanceV1,
            3 => BalanceV4,
            5 => StereoPan,
            6 => DualPan,
            x => Unknown(Hidden(x)),
        }
    }

    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use PanMode::*;
        match self {
            BalanceV1 => 0,
            BalanceV4 => 3,
            StereoPan => 5,
            DualPan => 6,
            Unknown(Hidden(x)) => x,
        }
    }
}

/// Track pan.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Pan {
    /// Classic v1 - v3.
    BalanceV1(ReaperPanValue),
    /// Balance v4+.
    BalanceV4(ReaperPanValue),
    /// Stereo pan.
    StereoPan {
        pan: ReaperPanValue,
        width: ReaperWidthValue,
    },
    /// Dual pan.
    DualPan {
        left: ReaperPanValue,
        right: ReaperPanValue,
    },
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
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
            // If the unique ID of the given section is zero, then this also corresponds to the
            // main section.
            Sec(i) => i.0.as_ptr(),
        }
    }

    pub(crate) fn from_medium(value: Option<&KbdSectionInfo>) -> SectionContext {
        use SectionContext::*;
        match value {
            None => MainSection,
            Some(info) => {
                if info.unique_id().get() == 0 {
                    MainSection
                } else {
                    Sec(&info)
                }
            }
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

    /// Converts this raw pointer as returned from the low-level API to a window context.
    pub(crate) fn from_raw(raw: raw::HWND) -> WindowContext {
        use WindowContext::*;
        match NonNull::new(raw) {
            None => NoWindow,
            Some(hwnd) => Win(hwnd),
        }
    }
}

/// Defines which action will be preselected when prompting for an action.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum InitialAction {
    /// No action will be preselected.
    NoneSelected,
    /// Action with the given command ID will be preselected.
    Selected(CommandId),
}

impl InitialAction {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use InitialAction::*;
        match self {
            NoneSelected => 0,
            Selected(id) => id.to_raw(),
        }
    }
}

/// Possible result when prompting for an action.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PromptForActionResult {
    /// Action window is no longer available.
    ActionWindowGone,
    /// No action is selected.
    NoneSelected,
    /// Action with the given command ID is selected.
    Selected(CommandId),
    /// Represents a variant unknown to *reaper-rs*. Please contribute if you encounter a variant
    /// that is supported by REAPER but not yet by *reaper-rs*. Thanks!
    Unknown(Hidden<i32>),
}

impl PromptForActionResult {
    /// Converts an integer as returned by the low-level API to this result.
    pub fn from_raw(v: i32) -> PromptForActionResult {
        use PromptForActionResult::*;
        match v {
            0 => NoneSelected,
            id if id > 0 => Selected(CommandId::new(id as u32)),
            -1 => ActionWindowGone,
            x => Unknown(Hidden(x)),
        }
    }
}

/// Decides when a MIDI message will be sent.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SendMidiTime {
    /// MIDI message will be sent instantly.
    Instantly,
    /// MIDI messages will be sent at the given frame offset.
    AtFrameOffset(MidiFrameOffset),
}

impl SendMidiTime {
    /// Converts this value to an integer as expected by the low-level API.
    pub fn to_raw(self) -> i32 {
        use SendMidiTime::*;
        match self {
            Instantly => -1,
            AtFrameOffset(o) => o.to_raw(),
        }
    }
}
