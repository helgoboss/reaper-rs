use crate::high_level::{Track, Reaper, get_media_track_guid};
use crate::high_level::fx::{Fx, get_fx_guid};
use crate::high_level::guid::Guid;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FxChain {
    track: Track,
    is_input_fx: bool,
}

impl FxChain {
    pub(super) fn new(track: Track, is_input_fx: bool) -> FxChain {
        FxChain {
            track,
            is_input_fx,
        }
    }

    pub fn get_fx_count(&self) -> u32 {
        let reaper = Reaper::instance();
        if self.is_input_fx {
            reaper.medium.track_fx_get_rec_count(self.track.get_media_track()) as u32
        } else {
            reaper.medium.track_fx_get_count(self.track.get_media_track()) as u32
        }
    }

    // Returned FX has GUIDs set
    pub fn get_fxs(&self) -> impl Iterator<Item=Fx> + '_ {
        (0..self.get_fx_count()).map(move |i| {
            Fx::from_guid_and_index(
                self.track.clone(),
                get_fx_guid(&self.track, i, self.is_input_fx).expect("Couldn't determine FX GUID"),
                i,
                self.is_input_fx,
            )
        })
    }

    // This returns a non-optional in order to support not-yet-loaded FX. GUID is a perfectly stable
    // identifier of an FX!
    pub fn get_fx_by_guid(&self, guid: &Guid) -> Fx {
        Fx::from_guid_lazy_index(self.track.clone(), *guid, self.is_input_fx)
    }

    // It's correct that this returns an optional because the index isn't a stable identifier of an FX.
    // The FX could move. So this should do a runtime lookup of the FX and return a stable GUID-backed Fx object if
    // an FX exists at that index.
    pub fn get_fx_by_index(&self, index: u32) -> Option<Fx> {
        if index >= self.get_fx_count() {
            return None
        }
        Some(Fx::from_guid_and_index(
            self.track.clone(),
            get_fx_guid(&self.track, index, self.is_input_fx).expect("Couldn't determine FX GUID"),
            index,
            self.is_input_fx
        ))
    }

    pub fn is_available(&self) -> bool {
        self.track.is_available()
    }
}
