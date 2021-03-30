use std::cell::Cell;

use std::ops::Deref;
use std::path::PathBuf;

use crate::fx_chain::FxChain;
use crate::fx_parameter::FxParameter;
use crate::guid::Guid;
use crate::option_util::OptionExt;
use crate::{ChunkRegion, FxChainContext, Project, Reaper, Track};
use reaper_medium::{
    FxPresetRef, FxShowInstruction, Hwnd, ReaperFunctionError, ReaperString, ReaperStringArg,
    TrackFxLocation,
};
use std::hash::{Hash, Hasher};

#[derive(Clone, Eq, Debug)]
pub struct Fx {
    chain: FxChain,
    // Primary identifier, but only for tracked, GUID-based FX instances. Otherwise empty.
    guid: Option<Guid>,
    // For GUID-based FX instances this is the secondary identifier, can become invalid on FX
    // reorderings. For just index-based FX instances this is the primary identifier.
    index: Cell<Option<u32>>,
}

impl PartialEq for Fx {
    fn eq(&self, other: &Self) -> bool {
        if self.chain != other.chain {
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

impl Hash for Fx {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.chain.hash(state);
        if let Some(guid) = self.guid {
            guid.hash(state);
        } else {
            self.index.get().hash(state);
        }
    }
}

impl Fx {
    // Main constructor. Use it if you have the GUID. index will be determined lazily.
    pub(crate) fn from_guid_lazy_index(chain: FxChain, guid: Guid) -> Fx {
        Fx {
            chain,
            guid: Some(guid),
            index: Cell::new(None),
        }
    }

    // Use this constructor if you are sure about the GUID and index
    pub(crate) fn from_guid_and_index(chain: FxChain, guid: Guid, index: u32) -> Fx {
        Fx {
            chain,
            guid: Some(guid),
            index: Cell::new(Some(index)),
        }
    }

    // Use this if you want to create a purely index-based FX without UUID tracking.
    pub(crate) fn from_index_untracked(chain: FxChain, index: u32) -> Fx {
        Fx {
            chain,
            guid: None,
            index: Cell::new(Some(index)),
        }
    }

    pub fn project(&self) -> Option<Project> {
        self.chain.project()
    }

    pub fn name(&self) -> ReaperString {
        self.load_if_necessary_or_complain();
        let buffer_size = 256;
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_fx_name(track.raw(), location, buffer_size)
                        .expect("Couldn't get track FX name")
                }
            }
        }
    }

    pub fn chunk(&self) -> Result<ChunkRegion, &'static str> {
        self.load_if_necessary_or_complain();
        let res = self
            .chain()
            .chunk()?
            .ok_or("FX chain chunk not found")?
            .find_line_starting_with(self.fx_id_line()?.as_str())
            .ok_or("FX ID line not found")?
            .move_left_cursor_left_to_start_of_line_beginning_with("BYPASS ")
            .move_right_cursor_right_to_start_of_line_beginning_with("WAK 0")
            .move_right_cursor_right_to_end_of_current_line();
        Ok(res)
    }

    fn fx_id_line(&self) -> Result<String, &'static str> {
        Ok(get_fx_id_line(&self.guid().ok_or("couldn't get GUID")?))
    }

    pub fn tag_chunk(&self) -> Result<ChunkRegion, &'static str> {
        self.load_if_necessary_or_complain();
        let res = self
            .chain()
            .chunk()?
            .ok_or("FX chain chunk not found")?
            .find_line_starting_with(self.fx_id_line()?.as_str())
            .ok_or("FX ID line not found")?
            .move_left_cursor_left_to_start_of_line_beginning_with("BYPASS ")
            .find_first_tag(0)
            .ok_or("first tag not found")?;
        Ok(res)
    }

    pub fn state_chunk(&self) -> Result<ChunkRegion, &'static str> {
        let res = self
            .tag_chunk()?
            .move_left_cursor_right_to_start_of_next_line()
            .move_right_cursor_left_to_end_of_previous_line();
        Ok(res)
    }

    // Attention: Currently implemented by parsing chunk
    pub fn info(&self) -> Result<FxInfo, &'static str> {
        FxInfo::from_first_line_of_tag_chunk(&self.tag_chunk()?.first_line().content())
    }

    pub fn parameter_count(&self) -> u32 {
        self.load_if_necessary_or_complain();
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_num_params(track.raw(), location)
                }
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_enabled(track.raw(), location)
                }
            }
        }
    }

    pub fn get_named_config_param<'a>(
        &self,
        name: impl Into<ReaperStringArg<'a>>,
        buffer_size: u32,
    ) -> Result<Vec<u8>, ReaperFunctionError> {
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_named_config_parm(track.raw(), location, name, buffer_size)
                }
            }
        }
    }

    pub fn set_named_config_param<'a>(
        &self,
        name: impl Into<ReaperStringArg<'a>>,
        buffer: &[u8],
    ) -> Result<(), ReaperFunctionError> {
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_set_named_config_parm(track.raw(), location, name, buffer)
                }
            }
        }
    }

    pub fn parameters(&self) -> impl Iterator<Item = FxParameter> + ExactSizeIterator + '_ {
        self.load_if_necessary_or_complain();
        (0..self.parameter_count()).map(move |i| self.parameter_by_index(i))
    }

    pub fn guid(&self) -> Option<Guid> {
        self.guid
    }

    pub fn parameter_by_index(&self, index: u32) -> FxParameter {
        FxParameter::new(self.clone(), index)
    }

    /// Will return None if monitoring FX.
    ///
    /// In some scenarios it makes sense to fall back to the master track of the current project.
    pub fn track(&self) -> Option<&Track> {
        self.chain.track()
    }

    pub fn query_index(&self) -> TrackFxLocation {
        get_track_fx_location(self.index(), self.is_input_fx())
    }

    /// Panics if this is a take FX.
    pub(crate) fn track_and_location(&self) -> (Track, TrackFxLocation) {
        get_track_and_location(&self.chain, self.index())
    }

    pub fn index(&self) -> u32 {
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
        if !self.chain().is_available() {
            return false;
        }
        match self.guid {
            None => true, // No GUID tracking
            Some(guid) => {
                // Loaded but might be at wrong index
                self.guid_by_index(index) == Some(guid)
            }
        }
    }

    // Returns None if no FX at that index anymore
    fn guid_by_index(&self, index: u32) -> Option<Guid> {
        get_fx_guid(self.chain(), index)
    }

    fn load_by_guid(&self) -> bool {
        if !self.chain().is_available() {
            return false;
        }
        let guid = match self.guid {
            None => return false, // No GUID tracking
            Some(guid) => guid,
        };
        let found_fx = self.chain().fxs().find(|fx| fx.guid() == Some(guid));
        if let Some(fx) = found_fx {
            self.index.replace(Some(fx.index()));
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

    // TODO-low How much sense does it make to expect a chunk region here? Why not a &str? Type
    // safety?  Probably because a ChunkRegion is a shared owner of what it holds. If we pass
    // just a &str,  we would need to copy to achieve that ownership. We might need to
    // reconsider the ownership  requirement of ChunkRegions as a whole (but then we need to
    // care about lifetimes).
    // TODO-low Supports track FX only
    pub fn set_chunk(&self, chunk_region: ChunkRegion) -> Result<(), &'static str> {
        // First replace GUID in chunk with the one of this FX
        let mut parent_chunk = chunk_region.parent_chunk();
        if let Some(fx_id_line) = chunk_region.find_line_starting_with("FXID ") {
            // TODO-low Mmh. We assume here that this is a guid-based FX!?
            let guid = self.guid().ok_or("FX doesn't have GUID")?;
            parent_chunk.replace_region(&fx_id_line, get_fx_id_line(&guid).as_str());
        }
        // Then set new chunk
        self.replace_track_chunk_region(self.chunk()?, chunk_region.content().deref())?;
        Ok(())
    }

    // TODO-low Supports track FX only
    pub fn set_tag_chunk(&self, chunk: &str) -> Result<(), &'static str> {
        self.replace_track_chunk_region(self.tag_chunk()?, chunk)
    }

    // TODO-low Supports track FX only
    pub fn set_state_chunk(&self, chunk: &str) -> Result<(), &'static str> {
        self.replace_track_chunk_region(self.state_chunk()?, chunk)
    }

    pub fn floating_window(&self) -> Option<Hwnd> {
        self.load_if_necessary_or_complain();
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_floating_window(track.raw(), location)
                }
            }
        }
    }

    pub fn window_is_open(&self) -> bool {
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_open(track.raw(), location)
                }
            }
        }
    }

    pub fn window_has_focus(&self) -> bool {
        OptionExt::contains(&Reaper::get().focused_fx(), self)
    }

    pub fn show_in_floating_window(&self) {
        self.load_if_necessary_or_complain();
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get().medium_reaper().track_fx_show(
                        track.raw(),
                        FxShowInstruction::ShowFloatingWindow(location),
                    );
                }
            }
        }
    }

    pub fn hide_floating_window(&self) {
        self.load_if_necessary_or_complain();
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get().medium_reaper().track_fx_show(
                        track.raw(),
                        FxShowInstruction::HideFloatingWindow(location),
                    );
                }
            }
        }
    }

    pub fn show_in_chain(&self) {
        self.load_if_necessary_or_complain();
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_show(track.raw(), FxShowInstruction::ShowChain(location));
                }
            }
        }
    }

    // TODO-low Supports track FX only
    fn replace_track_chunk_region(
        &self,
        old_chunk_region: ChunkRegion,
        new_content: &str,
    ) -> Result<(), &'static str> {
        let mut old_chunk = old_chunk_region.parent_chunk();
        old_chunk.replace_region(&old_chunk_region, new_content);
        std::mem::drop(old_chunk_region);
        self.track()
            .ok_or("only track FX supported")?
            .set_chunk(old_chunk)?;
        Ok(())
    }

    pub fn chain(&self) -> &FxChain {
        &self.chain
    }

    pub fn enable(&self) {
        self.set_enabled(true);
    }

    pub fn disable(&self) {
        self.set_enabled(false);
    }

    fn set_enabled(&self, enabled: bool) {
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get().medium_reaper().track_fx_set_enabled(
                        track.raw(),
                        location,
                        enabled,
                    );
                }
            }
        }
    }

    pub fn is_input_fx(&self) -> bool {
        self.chain.is_input_fx()
    }

    pub fn is_available(&self) -> bool {
        if self.is_loaded_and_at_correct_index() {
            if self.is_tracked() {
                true
            } else {
                // "Loaded and at correct index" has not much of a meaning if there's no GUID
                // tracking. We need to check the FX count.
                self.index.get().expect("untracked FX always has index") < self.chain().fx_count()
            }
        } else {
            // Not yet loaded or at wrong index
            self.load_by_guid()
        }
    }

    fn is_tracked(&self) -> bool {
        self.guid.is_some()
    }

    pub fn preset_count(&self) -> Result<u32, ReaperFunctionError> {
        self.load_if_necessary_or_complain();
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                let res = unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_preset_index(track.raw(), location)?
                };
                Ok(res.count)
            }
        }
    }

    pub fn preset_index(&self) -> Result<Option<u32>, ReaperFunctionError> {
        self.load_if_necessary_or_complain();
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                let res = unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_preset_index(track.raw(), location)?
                };
                Ok(res.index)
            }
        }
    }

    pub fn activate_preset(&self, preset: FxPresetRef) {
        self.load_if_necessary_or_complain();
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    let _ = Reaper::get().medium_reaper().track_fx_set_preset_by_index(
                        track.raw(),
                        location,
                        preset,
                    );
                }
            }
        }
    }

    pub fn preset_is_dirty(&self) -> bool {
        self.load_if_necessary_or_complain();
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                let result = unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_preset(track.raw(), location, 0)
                };
                !result.state_matches_preset
            }
        }
    }

    pub fn preset_name(&self) -> Option<ReaperString> {
        self.load_if_necessary_or_complain();
        match self.chain.context() {
            FxChainContext::Take(_) => todo!(),
            _ => {
                let (track, location) = self.track_and_location();
                unsafe {
                    Reaper::get()
                        .medium_reaper()
                        .track_fx_get_preset(track.raw(), location, 2000)
                        .name
                }
            }
        }
    }
}

/// Panics if a take FX chain is passed.
fn get_track_and_location(chain: &FxChain, index: u32) -> (Track, TrackFxLocation) {
    match chain.context() {
        FxChainContext::Monitoring => {
            let track = Reaper::get().current_project().master_track();
            let location = TrackFxLocation::InputFxChain(index);
            (track, location)
        }
        FxChainContext::Track { track, is_input_fx } => {
            let location = get_track_fx_location(index, *is_input_fx);
            (track.clone(), location)
        }
        FxChainContext::Take(_) => panic!("not possible for take FX"),
    }
}

pub fn get_fx_guid(chain: &FxChain, index: u32) -> Option<Guid> {
    let raw_guid = match chain.context() {
        FxChainContext::Take(_) => todo!(),
        _ => {
            let (track, location) = get_track_and_location(chain, index);
            unsafe {
                Reaper::get()
                    .medium_reaper()
                    .track_fx_get_fx_guid(track.raw(), location)
                    .ok()
            }
        }
    };
    raw_guid.map(Guid::new)
}

pub fn get_index_from_query_index(query_index: i32) -> (u32, bool) {
    if query_index >= 0x0100_0000 {
        ((query_index - 0x0100_0000) as u32, true)
    } else {
        (query_index as u32, false)
    }
}

pub fn get_track_fx_location(index: u32, is_input_fx: bool) -> TrackFxLocation {
    use TrackFxLocation::*;
    if is_input_fx {
        InputFxChain(index)
    } else {
        NormalFxChain(index)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct FxInfo {
    /// e.g. "ReaSynth (Cockos)", currently empty if JS
    pub effect_name: String,
    /// e.g. "VST" or "JS"
    pub type_expression: String,
    /// e.g. "VSTi", currently empty if JS
    pub sub_type_expression: String,
    /// e.g. reasynth.dll or phaser
    pub file_name: PathBuf,
}

impl FxInfo {
    pub(crate) fn from_first_line_of_tag_chunk(line: &str) -> Result<FxInfo, &'static str> {
        // TODO-low Also handle other plugin types (DX, AU, LV2)
        // TODO-low Don't just assign empty strings in case of JS
        let vst_line_regex = regex!(r#"<VST "(.+?): (.+?)" (.+)"#);
        let vst_file_name_with_quotes_regex = regex!(r#""(.+?)".*"#);
        let vst_file_name_without_quotes_regex = regex!(r#"([^ ]+) .*"#);
        let js_file_name_with_quotes_regex = regex!(r#""(.+?)".*"#);
        let js_file_name_without_quotes_regex = regex!(r#"([^ ]+) .*"#);
        let first_space_index = line
            .find(' ')
            .ok_or("Couldn't find space in VST tag line")?;
        let type_expression = &line[1..first_space_index];
        match type_expression {
            "VST" => {
                let captures = vst_line_regex
                    .captures(line)
                    .ok_or("Couldn't parse VST tag line")?;
                assert_eq!(captures.len(), 4);
                Ok(FxInfo {
                    effect_name: captures[2].to_owned(),
                    type_expression: type_expression.to_owned(),
                    sub_type_expression: captures[1].to_owned(),
                    file_name: {
                        let remainder = &captures[3];
                        let remainder_regex = if remainder.starts_with('"') {
                            vst_file_name_with_quotes_regex
                        } else {
                            vst_file_name_without_quotes_regex
                        };
                        let remainder_captures = remainder_regex
                            .captures(remainder)
                            .ok_or("Couldn't parse VST file name")?;
                        assert_eq!(remainder_captures.len(), 2);
                        PathBuf::from(&remainder_captures[1])
                    },
                })
            }
            "JS" => Ok(FxInfo {
                effect_name: "".to_string(),
                type_expression: "".to_string(),
                sub_type_expression: "".to_string(),
                file_name: {
                    let remainder = &line[4..];
                    let remainder_regex = if remainder.starts_with('"') {
                        js_file_name_with_quotes_regex
                    } else {
                        js_file_name_without_quotes_regex
                    };
                    let remainder_captures = remainder_regex
                        .captures(remainder)
                        .ok_or("Couldn't parse JS file name")?;
                    assert_eq!(remainder_captures.len(), 2);
                    PathBuf::from(&remainder_captures[1])
                },
            }),
            _ => Err("Can only handle VST and JS FX types right now"),
        }
    }
}

fn get_fx_id_line(guid: &Guid) -> String {
    format!("FXID {}", guid.to_string_with_braces())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vsti_2() {
        // Given
        let line = r#"<VST "VSTi: ReaLearn (Helgoboss)" ReaLearn-x64.dll 0 "Launchpad EQ" 1751282284<5653546862726C7265616C6561726E00> "#;
        // When
        let result = FxInfo::from_first_line_of_tag_chunk(line);
        // Then
        assert_eq!(
            result,
            Ok(FxInfo {
                effect_name: "ReaLearn (Helgoboss)".into(),
                type_expression: "VST".into(),
                sub_type_expression: "VSTi".into(),
                file_name: "ReaLearn-x64.dll".into()
            })
        )
    }

    #[test]
    fn vst_2() {
        // Given
        let line = r#"<VST "VST: EQ (Nova)" "TDR Nova GE.dll" 0 "EQ (Nova)" 1415853361 """#;
        // When
        let result = FxInfo::from_first_line_of_tag_chunk(line);
        // Then
        assert_eq!(
            result,
            Ok(FxInfo {
                effect_name: "EQ (Nova)".into(),
                type_expression: "VST".into(),
                sub_type_expression: "VST".into(),
                file_name: "TDR Nova GE.dll".into()
            })
        )
    }

    #[test]
    fn vst_2_without_company() {
        // Given
        let line = r#"<VST "VST: BussColors4" BussColors464.dll 0 BussColors4 1651729204<5653546273633462757373636F6C6F72> """#;
        // When
        let result = FxInfo::from_first_line_of_tag_chunk(line);
        // Then
        assert_eq!(
            result,
            Ok(FxInfo {
                effect_name: "BussColors4".into(),
                type_expression: "VST".into(),
                sub_type_expression: "VST".into(),
                file_name: "BussColors464.dll".into()
            })
        )
    }

    #[test]
    fn vsti_3_without_company() {
        // Given
        let line = r#"<VST "VST3i: Hive" Hive(x64).vst3 0 "" 437120294{D39D5B69D6AF42FA1234567868495645} """#;
        // When
        let result = FxInfo::from_first_line_of_tag_chunk(line);
        // Then
        assert_eq!(
            result,
            Ok(FxInfo {
                effect_name: "Hive".into(),
                type_expression: "VST".into(),
                sub_type_expression: "VST3i".into(),
                file_name: "Hive(x64).vst3".into()
            })
        )
    }

    #[test]
    fn vst_3() {
        // Given
        let line = r#"<VST "VST3: Element FX (Kushview) (34ch)" KV_ElementFX.vst3 0 "" 1844386711{565354456C4658656C656D656E742066} """#;
        // When
        let result = FxInfo::from_first_line_of_tag_chunk(line);
        // Then
        assert_eq!(
            result,
            Ok(FxInfo {
                effect_name: "Element FX (Kushview) (34ch)".into(),
                type_expression: "VST".into(),
                sub_type_expression: "VST3".into(),
                file_name: "KV_ElementFX.vst3".into()
            })
        )
    }

    #[test]
    fn vst_3_without_company() {
        // Given
        let line = r#"<VST "VST3: True Iron" "True Iron.vst3" 0 "True Iron" 1519279131{5653544B505472747275652069726F6E} """#;
        // When
        let result = FxInfo::from_first_line_of_tag_chunk(line);
        // Then
        assert_eq!(
            result,
            Ok(FxInfo {
                effect_name: "True Iron".into(),
                type_expression: "VST".into(),
                sub_type_expression: "VST3".into(),
                file_name: "True Iron.vst3".into()
            })
        )
    }
}
