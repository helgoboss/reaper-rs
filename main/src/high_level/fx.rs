use crate::high_level::guid::Guid;
use crate::high_level::{Track, Reaper, LightTrack};
use std::cell::Cell;
use c_str_macro::c_str;


/// The difference to Fx is that this implements Copy (not just Clone). See LightTrack for explanation.
#[derive(Clone, Copy, Debug)]
pub struct LightFx {
    track: LightTrack,
    guid: Option<Guid>,
    index: Option<u32>,
    is_input_fx: bool,
}

impl From<LightFx> for Fx {
    fn from(light: LightFx) -> Self {
        Fx {
            track: light.track.into(),
            guid: light.guid,
            index: Cell::new(light.index),
            is_input_fx: light.is_input_fx
        }
    }
}

impl From<Fx> for LightFx {
    fn from(heavy: Fx) -> Self {
        LightFx {
            track: heavy.track.into(),
            guid: heavy.guid,
            index: heavy.index.get(),
            is_input_fx: heavy.is_input_fx
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Fx {
    // TODO Save chain instead of track
    track: Track,
    // Primary identifier, but only for tracked, GUID-based FX instances. Otherwise empty.
    guid: Option<Guid>,
    // For GUID-based FX instances this is the secondary identifier, can become invalid on FX reorderings.
    // For just index-based FX instances this is the primary identifier.
    index: Cell<Option<u32>>,
    is_input_fx: bool,
}

impl Fx {
    // Main constructor. Use it if you have the GUID. index will be determined lazily.
    pub fn from_guid_lazy_index(track: Track, guid: Guid, is_input_fx: bool) -> Fx {
        Fx {
            track,
            guid: Some(guid),
            is_input_fx,
            index: Cell::new(None),
        }
    }

    // Use this constructor if you are sure about the GUID and index
    pub fn from_guid_and_index(track: Track, guid: Guid, index: u32, is_input_fx: bool) -> Fx {
        Fx {
            track,
            guid: Some(guid),
            is_input_fx,
            index: Cell::new(Some(index)),
        }
    }

    pub fn get_guid(&self) -> Option<Guid> {
        self.guid
    }
}

pub fn get_fx_guid(track: &Track, index: u32, is_input_fx: bool) -> Option<Guid> {
    let query_index = get_fx_query_index(index, is_input_fx);
    let internal = Reaper::instance().medium.track_fx_get_fx_guid(track.get_media_track(), query_index);
    if internal.is_null() {
        None
    } else {
        Some(Guid::new(unsafe { *internal }))
    }
}

pub fn get_fx_query_index(index: u32, is_input_fx: bool) -> i32 {
    let addend: i32 = if is_input_fx { 0x1000000 } else { 0 };
    addend + (index as i32)
}