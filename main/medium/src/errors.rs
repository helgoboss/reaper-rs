use derive_more::*;

/// Creation of track send failed.
// TODO-medium In which cases can this actually happen? E.g. it doesn't happen if a send already
//  exists between two tracks, also if one tries to create a send to the same track.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Display, Error)]
#[display(fmt = "creation of track send failed")]
pub struct CreateTrackSendFailed;
