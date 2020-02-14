use crate::high_level::guid::Guid;
use crate::high_level::{Track, Reaper};
use std::cell::Cell;
use c_str_macro::c_str;
use crate::high_level::fx_parameter::FxParameter;
use crate::high_level::fx_chain::FxChain;

#[derive(Clone, Eq, Debug)]
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

impl PartialEq for Fx {
    fn eq(&self, other: &Self) -> bool {
        if self.track != other.track || self.is_input_fx != other.is_input_fx {
            return false;
        }
        if let (Some(self_guid), Some(other_guid)) = (self.guid, other.guid) {
            // Both FXs are guid-based
            self_guid == other_guid
        } else {
            self.index == other.index
        }
    }
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

    pub fn get_parameter_by_index(&self, index: u32) -> FxParameter {
        FxParameter::new(self.clone(), index)
    }

    pub fn get_track(&self) -> Track {
        self.track.clone()
    }

    pub fn get_query_index(&self) -> i32 {
        get_fx_query_index(self.get_index(), self.is_input_fx)
    }

    pub fn get_index(&self) -> u32 {
        if !self.is_loaded_and_at_correct_index() {
            self.load_by_guid();
        }
        self.index.get().expect("FX index could not be determined")
    }

    fn load_if_necessary_or_complain(&self) {
        if !self.is_loaded_and_at_correct_index() && !self.load_by_guid() {
            panic!("FX not loadable")
        }
    }

    fn is_loaded_and_at_correct_index(&self) -> bool {
        let index = match self.index.get() {
            None => return false, // Not loaded
            Some(index) => index
        };
        if !self.track.is_available() {
            return false;
        }
        match self.guid {
            None => true, // No GUID tracking
            Some(guid) => {
                // Loaded but might be at wrong index
                self.get_guid_by_index(index) == Some(guid)
            }
        }
    }

    // Returns None if no FX at that index anymore
    fn get_guid_by_index(&self, index: u32) -> Option<Guid> {
        get_fx_guid(&self.track, index, self.is_input_fx)
    }

    fn load_by_guid(&self) -> bool {
        if !self.get_chain().is_available() {
            return false;
        }
        let guid = match self.guid {
            None => return false, // No GUID tracking
            Some(guid) => guid
        };
        let found_fx = self.get_chain().get_fxs()
            .find(|fx| fx.get_guid() == Some(guid));
        if let Some(fx) = found_fx {
            self.index.replace(Some(fx.get_index()));
            true
        } else {
            false
        }
    }

    pub fn get_chain(&self) -> FxChain {
        if self.is_input_fx {
            self.track.get_input_fx_chain()
        } else {
            self.track.get_normal_fx_chain()
        }
    }

    pub fn is_input_fx(&self) -> bool {
        self.is_input_fx
    }

    pub fn is_available(&self) -> bool {
        if self.is_loaded_and_at_correct_index() {
            true
        } else {
            // Not yet loaded or at wrong index
            self.load_by_guid()
        }
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

pub fn get_index_from_query_index(query_index: i32) -> (u32, bool) {
    if query_index >= 0x1000000 {
        ((query_index - 0x1000000) as u32, true)
    } else {
        (query_index as u32, false)
    }
}

pub fn get_fx_query_index(index: u32, is_input_fx: bool) -> i32 {
    let addend: i32 = if is_input_fx { 0x1000000 } else { 0 };
    addend + (index as i32)
}