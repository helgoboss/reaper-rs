use derive_more::*;

#[derive(Debug, Clone, Eq, PartialEq, Display, Error)]
#[display(fmt = "creation of track send failed")]
pub struct CreateTrackSendFailed;
