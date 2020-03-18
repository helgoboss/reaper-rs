use std::cell::Cell;
use std::ffi::CString;
use std::ops::Deref;
use std::path::PathBuf;

use c_str_macro::c_str;

use crate::high_level::fx_chain::FxChain;
use crate::high_level::fx_parameter::FxParameter;
use crate::high_level::guid::Guid;
use crate::high_level::{Chunk, ChunkRegion, Reaper, Track};
use crate::low_level::{GetActiveWindow, HWND};
use rxrust::prelude::PayloadCopy;

#[derive(Clone, Eq, Debug)]
pub struct Fx {
    // TODO-low Save chain instead of track
    track: Track,
    // Primary identifier, but only for tracked, GUID-based FX instances. Otherwise empty.
    guid: Option<Guid>,
    // For GUID-based FX instances this is the secondary identifier, can become invalid on FX reorderings.
    // For just index-based FX instances this is the primary identifier.
    index: Cell<Option<u32>>,
    is_input_fx: bool,
}

impl PayloadCopy for Fx {}

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

    pub fn get_name(&self) -> CString {
        self.load_if_necessary_or_complain();
        Reaper::instance()
            .medium
            .track_fx_get_fx_name(self.track.get_media_track(), self.get_query_index(), 256)
            .expect("Couldn't get track name")
    }

    pub fn get_chunk(&self) -> ChunkRegion {
        self.load_if_necessary_or_complain();
        self.get_chain()
            .get_chunk()
            .unwrap()
            .find_line_starting_with(self.get_fx_id_line().as_str())
            .unwrap()
            .move_left_cursor_left_to_start_of_line_beginning_with("BYPASS ")
            .move_right_cursor_right_to_start_of_line_beginning_with("WAK 0")
            .move_right_cursor_right_to_end_of_current_line()
    }

    fn get_fx_id_line(&self) -> String {
        get_fx_id_line(&self.get_guid().expect("Couldn't get GUID"))
    }

    pub fn get_tag_chunk(&self) -> ChunkRegion {
        self.load_if_necessary_or_complain();
        self.get_chain()
            .get_chunk()
            .unwrap()
            .find_line_starting_with(self.get_fx_id_line().as_str())
            .unwrap()
            .move_left_cursor_left_to_start_of_line_beginning_with("BYPASS ")
            .find_first_tag(0)
            .unwrap()
    }

    pub fn get_state_chunk(&self) -> ChunkRegion {
        self.get_tag_chunk()
            .move_left_cursor_right_to_start_of_next_line()
            .move_right_cursor_left_to_end_of_previous_line()
    }

    // Attention: Currently implemented by parsing chunk
    pub fn get_info(&self) -> FxInfo {
        FxInfo::new(&self.get_tag_chunk().get_first_line().get_content())
    }

    pub fn get_parameter_count(&self) -> u32 {
        self.load_if_necessary_or_complain();
        Reaper::instance()
            .medium
            .track_fx_get_num_params(self.track.get_media_track(), self.get_query_index())
            as u32
    }

    pub fn is_enabled(&self) -> bool {
        Reaper::instance()
            .medium
            .track_fx_get_enabled(self.track.get_media_track(), self.get_query_index())
    }

    pub fn get_parameters(&self) -> impl Iterator<Item = FxParameter> + '_ {
        self.load_if_necessary_or_complain();
        (0..self.get_parameter_count()).map(move |i| self.get_parameter_by_index(i))
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
            Some(index) => index,
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
            Some(guid) => guid,
        };
        let found_fx = self
            .get_chain()
            .get_fxs()
            .find(|fx| fx.get_guid() == Some(guid));
        if let Some(fx) = found_fx {
            self.index.replace(Some(fx.get_index()));
            true
        } else {
            false
        }
    }

    // To be called if you become aware that this FX might have been affected by a reordering.
    // Note that the Fx also corrects the index itself whenever one of its methods is called.
    pub fn invalidate_index(&self) {
        self.load_by_guid();
    }

    // TODO-low How much sense does it make to expect a chunk region here? Why not a &str? Type safety?
    //  Probably because a ChunkRegion is a shared owner of what it holds. If we pass just a &str,
    //  we would need to copy to achieve that ownership. We might need to reconsider the ownership
    //  requirement of ChunkRegions as a whole (but then we need to care about lifetimes).
    pub fn set_chunk(&self, chunk_region: ChunkRegion) {
        // First replace GUID in chunk with the one of this FX
        let mut parent_chunk = chunk_region.get_parent_chunk();
        if let Some(fx_id_line) = chunk_region.find_line_starting_with("FXID ") {
            // TODO-low Mmh. We assume here that this is a guid-based FX!?
            let guid = self.get_guid().expect("FX doesn't have GUID");
            parent_chunk.replace_region(&fx_id_line, get_fx_id_line(&guid).as_str());
        }
        // Then set new chunk
        self.replace_track_chunk_region(self.get_chunk(), chunk_region.get_content().deref());
    }

    pub fn set_tag_chunk(&self, chunk: &str) {
        self.replace_track_chunk_region(self.get_tag_chunk(), chunk);
    }

    pub fn set_state_chunk(&self, chunk: &str) {
        self.replace_track_chunk_region(self.get_state_chunk(), chunk);
    }

    pub fn get_floating_window(&self) -> HWND {
        self.load_if_necessary_or_complain();
        Reaper::instance()
            .medium
            .track_fx_get_floating_window(self.track.get_media_track(), self.get_query_index())
    }

    pub fn window_is_open(&self) -> bool {
        Reaper::instance()
            .medium
            .track_fx_get_open(self.track.get_media_track(), self.get_query_index())
    }

    pub fn window_has_focus(&self) -> bool {
        let hwnd = self.get_floating_window();
        if hwnd.is_null() {
            // FX is not open in floating window. In this case we consider it as focused if the FX
            // chain of that track is open and the currently displayed FX in the FX chain is this FX.
            self.window_is_open()
        } else {
            // FX is open in floating window
            let active_window = unsafe { GetActiveWindow() };
            active_window == hwnd
        }
    }

    pub fn show_in_floating_window(&self) {
        self.load_if_necessary_or_complain();
        Reaper::instance().medium.track_fx_show(
            self.track.get_media_track(),
            self.get_query_index(),
            3,
        );
    }

    fn replace_track_chunk_region(&self, old_chunk_region: ChunkRegion, new_content: &str) {
        let mut old_chunk = old_chunk_region.get_parent_chunk();
        old_chunk.replace_region(&old_chunk_region, new_content);
        std::mem::drop(old_chunk_region);
        self.track.set_chunk(old_chunk);
    }

    pub fn get_chain(&self) -> FxChain {
        if self.is_input_fx {
            self.track.get_input_fx_chain()
        } else {
            self.track.get_normal_fx_chain()
        }
    }

    pub fn enable(&self) {
        Reaper::instance().medium.track_fx_set_enabled(
            self.track.get_media_track(),
            self.get_query_index(),
            true,
        );
    }

    pub fn disable(&self) {
        Reaper::instance().medium.track_fx_set_enabled(
            self.track.get_media_track(),
            self.get_query_index(),
            false,
        );
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

    pub fn get_preset_count(&self) -> u32 {
        self.load_if_necessary_or_complain();
        // TODO-high Use rustfmt everywhere
        // TODO-low Integrate into ReaPlus (current preset index?)
        Reaper::instance()
            .medium
            .track_fx_get_preset_index(self.track.get_media_track(), self.get_query_index())
            .expect("Couldn't get preset count")
            .1
    }

    pub fn preset_is_dirty(&self) -> bool {
        self.load_if_necessary_or_complain();
        let state_matches_preset = Reaper::instance()
            .medium
            .track_fx_get_preset(self.track.get_media_track(), self.get_query_index(), 0)
            .0;
        !state_matches_preset
    }

    pub fn get_preset_name(&self) -> Option<CString> {
        self.load_if_necessary_or_complain();
        Reaper::instance()
            .medium
            .track_fx_get_preset(self.track.get_media_track(), self.get_query_index(), 2000)
            .1
    }
}

pub fn get_fx_guid(track: &Track, index: u32, is_input_fx: bool) -> Option<Guid> {
    let query_index = get_fx_query_index(index, is_input_fx);
    let internal = Reaper::instance()
        .medium
        .track_fx_get_fx_guid(track.get_media_track(), query_index);
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

pub struct FxInfo {
    /// e.g. ReaSynth, currently empty if JS
    pub effect_name: String,
    /// e.g. VST or JS
    pub type_expression: String,
    /// e.g. VSTi, currently empty if JS
    pub sub_type_expression: String,
    /// e.g. Cockos, currently empty if JS
    pub vendor_name: String,
    /// e.g. reasynth.dll or phaser
    pub file_name: PathBuf,
}

impl FxInfo {
    pub fn new(first_line_of_tag_chunk: &str) -> FxInfo {
        // TODO-medium try_into() rather than panic
        // TODO-low Also handle other plugin types
        // TODO-low Don't just assign empty strings in case of JS
        let vst_line_regex = regex!(r#"<VST "(.+?): (.+?) \((.+?)\).*?" (.+)"#);
        let vst_file_name_with_quotes_regex = regex!(r#""(.+?)".*"#);
        let vst_file_name_without_quotes_regex = regex!(r#"([^ ]+) .*"#);
        let js_file_name_with_quotes_regex = regex!(r#""(.+?)".*"#);
        let js_file_name_without_quotes_regex = regex!(r#"([^ ]+) .*"#);
        let first_space_index = first_line_of_tag_chunk
            .find(' ')
            .expect("Couldn't find space in VST tag line");
        let type_expression = &first_line_of_tag_chunk[1..first_space_index];
        match type_expression {
            "VST" => {
                let captures = vst_line_regex
                    .captures(first_line_of_tag_chunk)
                    .expect("Couldn't parse VST tag line");
                assert_eq!(captures.len(), 5);
                FxInfo {
                    effect_name: captures[2].to_owned(),
                    type_expression: type_expression.to_owned(),
                    sub_type_expression: captures[1].to_owned(),
                    vendor_name: captures[3].to_owned(),
                    file_name: {
                        let remainder = &captures[4];
                        let remainder_regex = if remainder.starts_with('"') {
                            vst_file_name_with_quotes_regex
                        } else {
                            vst_file_name_without_quotes_regex
                        };
                        let remainder_captures = remainder_regex
                            .captures(remainder)
                            .expect("Couldn't parse VST file name");
                        assert_eq!(remainder_captures.len(), 2);
                        PathBuf::from(&remainder_captures[1])
                    },
                }
            }
            "JS" => FxInfo {
                effect_name: "".to_string(),
                type_expression: "".to_string(),
                sub_type_expression: "".to_string(),
                vendor_name: "".to_string(),
                file_name: {
                    let remainder = &first_line_of_tag_chunk[4..];
                    let remainder_regex = if remainder.starts_with('"') {
                        js_file_name_with_quotes_regex
                    } else {
                        js_file_name_without_quotes_regex
                    };
                    let remainder_captures = remainder_regex
                        .captures(remainder)
                        .expect("Couldn't parse JS file name");
                    assert_eq!(remainder_captures.len(), 2);
                    PathBuf::from(&remainder_captures[1])
                },
            },
            _ => panic!("Can only handle VST and JS FX types right now"),
        }
    }
}

fn get_fx_id_line(guid: &Guid) -> String {
    format!("FXID {}", guid.to_string_with_braces())
}
