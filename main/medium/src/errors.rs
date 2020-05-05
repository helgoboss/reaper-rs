use derive_more::*;

// TODO-medium In which cases can this actually happen? E.g. it doesn't happen if a send already
//  exists between two tracks, also if one tries to create a send to the same track.
// TODO-medium Maybe group some errors together and just make a different in the msg?
#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "creation of track send failed")]
pub struct CreateTrackSendFailed;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "adding FX failed")]
pub struct AddFxFailed;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "GUID string invalid")]
pub struct GuidStringInvalid;

// TODO-medium with the following, the reason is maybe not exhaustive Maybe also group errors if the
//  reason is not 100% clear (just use msg).
#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "REAPER function failed, reason not exactly clear")]
pub struct ReaperFunctionFailed;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "FX not found")]
pub struct FxNotFound;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "FX or FX parameter not found")]
pub struct FxOrParameterNotFound;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "FX or FX parameter not found or Cockos extensions not supported")]
pub struct FxOrParameterNotFoundOrCockosExtNotSupported;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "invalid track attribute key")]
pub struct InvalidTrackAttributeKey;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "registration failed")]
pub struct RegistrationFailed;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "unregistering failed because this was not registered")]
pub struct NotRegistered;
