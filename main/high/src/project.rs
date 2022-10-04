use crate::guid::Guid;
use crate::{
    BasicBookmarkInfo, BookmarkType, IndexBasedBookmark, Item, PlayRate, Reaper, ReaperResult,
    Tempo, Track,
};
use std::fmt::Debug;
use std::{iter, mem};

use either::Either;
use reaper_medium::ProjectContext::{CurrentProject, Proj};
use reaper_medium::{
    AutoSeekBehavior, BookmarkId, BookmarkRef, CountProjectMarkersResult, DurationInSeconds,
    GetLastMarkerAndCurRegionResult, GetLoopTimeRange2Result, MasterTrackBehavior, PanMode,
    PlayState, PositionInSeconds, ProjectContext, ProjectRef, ReaProject, ReaperString,
    ReaperStringArg, SetEditCurPosOptions, TimeMap2TimeToBeatsResult, TimeMode, TimeModeOverride,
    TimeRangeType, TrackDefaultsBehavior, TrackLocation, UndoBehavior,
};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Project {
    rea_project: ReaProject,
}

const MAX_PATH_LENGTH: u32 = 5000;

// The pointer will never be dereferenced, so we can safely make it Send and Sync.
unsafe impl Send for Project {}
unsafe impl Sync for Project {}

impl Project {
    pub fn new(rea_project: ReaProject) -> Project {
        Project { rea_project }
    }

    pub fn raw(self) -> ReaProject {
        self.rea_project
    }

    pub fn first_track(self) -> Option<Track> {
        self.track_by_index(0)
    }

    pub fn file(self) -> Option<PathBuf> {
        Reaper::get()
            .medium_reaper()
            .enum_projects(ProjectRef::Tab(self.index().ok()?), MAX_PATH_LENGTH)
            .unwrap()
            .file_path
    }

    pub fn any_solo(self) -> bool {
        Reaper::get().medium_reaper().any_track_solo(self.context())
    }

    pub fn index(self) -> ReaperResult<u32> {
        self.complain_if_not_available()?;
        let rea_project = self.rea_project;
        let index = Reaper::get()
            .projects()
            .enumerate()
            .find(|(_, rp)| rp.rea_project == rea_project)
            .map(|(i, _)| i)
            .unwrap();
        Ok(index as u32)
    }

    /// It's correct that this returns an Option because the index isn't a stable identifier of a
    /// track. The track could move. So this should do a runtime lookup of the track and return a
    /// stable MediaTrack-backed Some(Track) if a track exists at that index. 0 is first normal
    /// track (master track is not obtainable via this method).
    pub fn track_by_index(self, idx: u32) -> Option<Track> {
        self.complain_if_not_available().ok()?;
        let media_track = Reaper::get()
            .medium_reaper()
            .get_track(Proj(self.rea_project), idx)?;
        Some(Track::new(media_track, Some(self.rea_project)))
    }

    // TODO Probably an unnecessary method
    pub fn track_by_ref(self, track_location: TrackLocation) -> Option<Track> {
        use TrackLocation::*;
        match track_location {
            MasterTrack => Some(self.master_track().ok()?),
            NormalTrack(idx) => self.track_by_index(idx),
        }
    }

    // This returns a non-optional in order to support not-yet-loaded tracks. GUID is a perfectly
    // stable identifier of a track!
    pub fn track_by_guid(self, guid: &Guid) -> ReaperResult<Track> {
        self.complain_if_not_available()?;
        Ok(Track::from_guid(self, *guid))
    }

    pub fn tracks(self) -> impl Iterator<Item = Track> + ExactSizeIterator + 'static {
        if self.complain_if_not_available().is_err() {
            return Either::Left(iter::empty());
        }
        let iter = (0..self.track_count()).map(move |i| {
            let media_track = Reaper::get()
                .medium_reaper()
                .get_track(Proj(self.rea_project), i)
                .unwrap();
            Track::new(media_track, Some(self.rea_project))
        });
        Either::Right(iter)
    }

    pub fn items(self) -> impl Iterator<Item = Item> + ExactSizeIterator + 'static {
        if self.complain_if_not_available().is_err() {
            return Either::Left(iter::empty());
        }
        let iter = (0..self.item_count()).map(move |i| {
            let media_item = Reaper::get()
                .medium_reaper()
                .get_media_item(self.context(), i)
                .unwrap();
            Item::new(media_item)
        });
        Either::Right(iter)
    }

    pub fn select_item_exclusively(&self, item: Item) {
        for item in self.items() {
            item.set_selected(false);
        }
        item.set_selected(true);
    }

    pub fn is_available(self) -> bool {
        Reaper::get()
            .medium_reaper()
            .validate_ptr_2(CurrentProject, self.rea_project)
    }

    pub fn selected_track_count(self, want_master: MasterTrackBehavior) -> u32 {
        Reaper::get()
            .medium_reaper()
            .count_selected_tracks_2(Proj(self.rea_project), want_master) as u32
    }

    pub fn first_selected_track(self, want_master: MasterTrackBehavior) -> Option<Track> {
        let media_track = Reaper::get().medium_reaper().get_selected_track_2(
            Proj(self.rea_project),
            0,
            want_master,
        )?;
        Some(Track::new(media_track, Some(self.rea_project)))
    }

    pub fn first_selected_item(self) -> Option<Item> {
        let raw_item = Reaper::get()
            .medium_reaper()
            .get_selected_media_item(self.context(), 0)?;
        Some(Item::new(raw_item))
    }

    pub fn unselect_all_tracks(self) {
        // TODO-low No project context
        unsafe {
            Reaper::get().medium_reaper().set_only_track_selected(None);
        }
    }

    pub fn selected_tracks(
        self,
        want_master: MasterTrackBehavior,
    ) -> impl Iterator<Item = Track> + 'static {
        if self.complain_if_not_available().is_err() {
            return Either::Left(iter::empty());
        }
        let iter = (0..self.selected_track_count(want_master)).map(move |i| {
            let media_track = Reaper::get()
                .medium_reaper()
                .get_selected_track_2(Proj(self.rea_project), i, want_master)
                .unwrap();
            Track::new(media_track, Some(self.rea_project))
        });
        Either::Right(iter)
    }

    pub fn context(self) -> ProjectContext {
        Proj(self.rea_project)
    }

    pub fn track_count(self) -> u32 {
        if self.complain_if_not_available().is_err() {
            return 0;
        }
        Reaper::get().medium_reaper().count_tracks(self.context()) as u32
    }

    pub fn item_count(self) -> u32 {
        if self.complain_if_not_available().is_err() {
            return 0;
        }
        Reaper::get()
            .medium_reaper()
            .count_media_items(self.context())
    }

    // TODO-low Introduce variant that doesn't notify ControlSurface
    pub fn add_track(self) -> ReaperResult<Track> {
        self.insert_track_at(self.track_count())
    }

    pub fn remove_track(self, track: &Track) {
        unsafe {
            Reaper::get().medium_reaper().delete_track(track.raw());
        }
    }

    // TODO-low Introduce variant that doesn't notify ControlSurface
    pub fn insert_track_at(self, index: u32) -> ReaperResult<Track> {
        self.complain_if_not_available()?;
        // TODO-low reaper::InsertTrackAtIndex unfortunately doesn't allow to specify ReaProject :(
        let reaper = Reaper::get().medium_reaper();
        reaper.insert_track_at_index(index, TrackDefaultsBehavior::OmitDefaultEnvAndFx);
        reaper.track_list_update_all_external_surfaces();
        let media_track = reaper.get_track(Proj(self.rea_project), index).unwrap();
        Ok(Track::new(media_track, Some(self.rea_project)))
    }

    pub fn master_track(self) -> ReaperResult<Track> {
        self.complain_if_not_available()?;
        let mt = Reaper::get()
            .medium_reaper()
            .get_master_track(Proj(self.rea_project));
        Ok(Track::new(mt, Some(self.rea_project)))
    }

    pub fn undoable<'a, F, R>(self, label: impl Into<ReaperStringArg<'a>>, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        if Reaper::get()
            .currently_loading_or_saving_project()
            .is_some()
        {
            operation()
        } else {
            let label = label.into().into_inner();
            let undo_block = Reaper::get().enter_undo_block_internal(self, label.as_ref());
            let result = operation();
            std::mem::drop(undo_block);
            result
        }
    }

    pub fn undo(self) -> bool {
        if self.complain_if_not_available().is_err() {
            return false;
        }
        Reaper::get()
            .medium_reaper()
            .undo_do_undo_2(Proj(self.rea_project))
    }

    pub fn redo(self) -> bool {
        if self.complain_if_not_available().is_err() {
            return false;
        }
        Reaper::get()
            .medium_reaper()
            .undo_do_redo_2(Proj(self.rea_project))
    }

    pub fn mark_as_dirty(self) {
        Reaper::get()
            .medium_reaper()
            .mark_project_dirty(Proj(self.rea_project));
    }

    pub fn is_dirty(self) -> bool {
        Reaper::get()
            .medium_reaper()
            .is_project_dirty(Proj(self.rea_project))
    }

    pub fn label_of_last_undoable_action(self) -> Option<ReaperString> {
        self.complain_if_not_available().ok()?;
        Reaper::get()
            .medium_reaper()
            .undo_can_undo_2(Proj(self.rea_project), |s| s.to_owned())
    }

    pub fn label_of_last_redoable_action(self) -> Option<ReaperString> {
        self.complain_if_not_available().ok()?;
        Reaper::get()
            .medium_reaper()
            .undo_can_redo_2(Proj(self.rea_project), |s| s.to_owned())
    }

    pub fn tempo(self) -> Tempo {
        let bpm = if self == Reaper::get().current_project() {
            Reaper::get().medium_reaper().master_get_tempo()
        } else {
            // ReaLearn #283
            Reaper::get()
                .medium_reaper()
                .time_map_2_get_divided_bpm_at_time(self.context(), PositionInSeconds::new(0.0))
        };
        Tempo::from_bpm(bpm)
    }

    pub fn play_rate(self) -> PlayRate {
        let factor = Reaper::get()
            .medium_reaper()
            .master_get_play_rate(Proj(self.raw()));
        PlayRate::from_playback_speed_factor(factor)
    }

    pub fn set_play_rate(self, play_rate: PlayRate) {
        Reaper::get()
            .medium_reaper()
            .csurf_on_play_rate_change(play_rate.playback_speed_factor());
    }

    pub fn set_tempo(self, tempo: Tempo, undo_hint: UndoBehavior) -> ReaperResult<()> {
        self.complain_if_not_available()?;
        Reaper::get().medium_reaper().set_current_bpm(
            Proj(self.rea_project),
            tempo.bpm(),
            undo_hint,
        );
        Ok(())
    }

    pub fn is_playing(self) -> bool {
        self.play_state().is_playing
    }

    pub fn play(self) {
        Reaper::get()
            .medium_reaper()
            .on_play_button_ex(Proj(self.rea_project));
    }

    pub fn is_paused(self) -> bool {
        self.play_state().is_paused
    }

    /// Doesn't toggle!
    pub fn pause(self) {
        if self.is_paused() {
            return;
        }
        Reaper::get()
            .medium_reaper()
            .on_pause_button_ex(Proj(self.rea_project));
    }

    pub fn is_stopped(self) -> bool {
        let state = self.play_state();
        !state.is_playing && !state.is_paused
    }

    pub fn stop(self) {
        Reaper::get()
            .medium_reaper()
            .on_stop_button_ex(Proj(self.rea_project));
    }

    pub fn is_recording(self) -> bool {
        self.play_state().is_recording
    }

    pub fn repeat_is_enabled(self) -> bool {
        Reaper::get()
            .medium_reaper()
            .get_set_repeat_ex_get(Proj(self.rea_project))
    }

    pub fn enable_repeat(self) {
        self.set_repeat_is_enabled(true);
    }

    pub fn disable_repeat(self) {
        self.set_repeat_is_enabled(false);
    }

    pub fn play_state(self) -> PlayState {
        Reaper::get()
            .medium_reaper()
            .get_play_state_ex(Proj(self.rea_project))
    }

    pub fn find_bookmark_by_type_and_index(
        self,
        bookmark_type: BookmarkType,
        index: u32,
    ) -> Option<FindBookmarkResult> {
        self.bookmarks_of_type(bookmark_type)
            .find(|res| res.index_within_type == index)
    }

    pub fn find_bookmark_by_type_and_id(
        self,
        bookmark_type: BookmarkType,
        id: BookmarkId,
    ) -> Option<FindBookmarkResult> {
        self.bookmarks_of_type(bookmark_type)
            .find(|res| res.basic_info.id == id)
    }

    pub fn directory(self) -> Option<PathBuf> {
        let file = self.file()?;
        let dir = file.parent()?;
        Some(dir.to_owned())
    }

    pub fn make_path_relative(self, path: &Path) -> Option<PathBuf> {
        let dir = self.directory()?;
        pathdiff::diff_paths(path, dir)
    }

    pub fn make_path_relative_if_in_project_directory(self, path: &Path) -> Option<PathBuf> {
        let dir = self.directory()?;
        if path.starts_with(&dir) {
            pathdiff::diff_paths(path, dir)
        } else {
            Some(path.to_owned())
        }
    }

    pub fn recording_path(self) -> PathBuf {
        Reaper::get()
            .medium_reaper
            .get_project_path_ex(self.context(), MAX_PATH_LENGTH)
    }

    pub fn make_path_absolute(self, path: &Path) -> Option<PathBuf> {
        if path.is_relative() {
            let dir = self.directory()?;
            Some(dir.join(path))
        } else {
            Some(path.to_owned())
        }
    }

    fn bookmarks_of_type(
        self,
        bookmark_type: BookmarkType,
    ) -> impl Iterator<Item = FindBookmarkResult> {
        self.bookmarks()
            // Enumerate across types
            .enumerate()
            .map(|(i, b)| {
                FindBookmarkResult {
                    index: i as _,
                    // Not yet set
                    index_within_type: 0,
                    bookmark: b,
                    basic_info: b.basic_info(),
                }
            })
            .filter(move |res| res.basic_info.bookmark_type() == bookmark_type)
            // Enumerate within this type
            .enumerate()
            .map(|(i, mut res)| {
                res.index_within_type = i as _;
                res
            })
    }

    // If we make this clean one day, I think this a good way: When wandering from the project to
    // a bookmark, we *should* return an Option if it doesn't exist. If one wants to create a
    // IndexBasedBookmark value - irrelevant of it exists or not - they can just create it
    // directly. That's good because it allows for a fluent, idiomatic API. The methods of the
    // returned object should not return an error if the object is not available - they should
    // panic instead because at this point (the fluent API use) we can safely assume they *are*
    // available - because it was checked in the find() call before. Long-living objects whose
    // methods return results depending on their availability are maybe not a good idea!
    //
    // The returned bookmark should provide methods to dive further in a fluent way (doing
    // REAPER function calls as necessary). It shouldn't contain any snapshot data.
    // There's the related question how to deal with info that is discovered already while
    // finding the bookmark. It's a snapshot only, so it should *not* be part of the actually
    // returned bookmark. But it could be returned as side product.
    pub fn find_bookmark_by_index(self, index: u32) -> Option<IndexBasedBookmark> {
        if index >= self.bookmark_count().total_count {
            return None;
        }
        Some(IndexBasedBookmark::new(self, index))
    }

    pub fn bookmarks(self) -> impl Iterator<Item = IndexBasedBookmark> + ExactSizeIterator {
        (0..self.bookmark_count().total_count).map(move |i| IndexBasedBookmark::new(self, i))
    }

    pub fn bookmark_count(self) -> CountProjectMarkersResult {
        Reaper::get()
            .medium_reaper()
            .count_project_markers(self.context())
    }

    pub fn go_to_marker(self, marker: BookmarkRef) {
        Reaper::get()
            .medium_reaper()
            .go_to_marker(self.context(), marker);
    }

    pub fn go_to_region_with_smooth_seek(self, region: BookmarkRef) {
        Reaper::get()
            .medium_reaper()
            .go_to_region(self.context(), region);
    }

    pub fn current_bookmark_at(self, pos: PositionInSeconds) -> GetLastMarkerAndCurRegionResult {
        Reaper::get()
            .medium_reaper()
            .get_last_marker_and_cur_region(self.context(), pos)
    }

    pub fn current_bookmark(self) -> GetLastMarkerAndCurRegionResult {
        let reference_pos = self.play_or_edit_cursor_position();
        self.current_bookmark_at(reference_pos)
    }

    pub fn play_or_edit_cursor_position(self) -> PositionInSeconds {
        if self.is_playing() {
            self.play_position_latency_compensated()
        } else {
            self.edit_cursor_position()
        }
    }

    pub fn beat_info_at(self, tpos: PositionInSeconds) -> TimeMap2TimeToBeatsResult {
        Reaper::get()
            .medium_reaper
            .time_map_2_time_to_beats(self.context(), tpos)
    }

    pub fn play_position_next_audio_block(self) -> PositionInSeconds {
        Reaper::get()
            .medium_reaper()
            .get_play_position_2_ex(self.context())
    }

    pub fn play_position_latency_compensated(self) -> PositionInSeconds {
        Reaper::get()
            .medium_reaper()
            .get_play_position_ex(self.context())
    }

    pub fn edit_cursor_position(self) -> PositionInSeconds {
        Reaper::get()
            .medium_reaper()
            .get_cursor_position_ex(self.context())
    }

    pub fn time_selection(self) -> Option<GetLoopTimeRange2Result> {
        Reaper::get()
            .medium_reaper
            .get_set_loop_time_range_2_get(self.context(), TimeRangeType::TimeSelection)
    }

    pub fn loop_points(self) -> Option<GetLoopTimeRange2Result> {
        Reaper::get()
            .medium_reaper
            .get_set_loop_time_range_2_get(self.context(), TimeRangeType::LoopPoints)
    }

    pub fn set_time_selection(self, start: PositionInSeconds, end: PositionInSeconds) {
        Reaper::get().medium_reaper.get_set_loop_time_range_2_set(
            self.context(),
            TimeRangeType::TimeSelection,
            start,
            end,
            AutoSeekBehavior::DenyAutoSeek,
        );
    }

    pub fn set_loop_points(
        self,
        start: PositionInSeconds,
        end: PositionInSeconds,
        auto_seek_behavior: AutoSeekBehavior,
    ) {
        Reaper::get().medium_reaper.get_set_loop_time_range_2_set(
            self.context(),
            TimeRangeType::LoopPoints,
            start,
            end,
            auto_seek_behavior,
        );
    }

    pub fn length(self) -> DurationInSeconds {
        Reaper::get()
            .medium_reaper
            .get_project_length(self.context())
    }

    pub fn set_edit_cursor_position(self, time: PositionInSeconds, options: SetEditCurPosOptions) {
        Reaper::get()
            .medium_reaper
            .set_edit_curs_pos_2(self.context(), time, options);
    }

    pub fn pan_mode(self) -> PanMode {
        let raw = unsafe {
            self.get_project_config("panmode")
                .expect("couldn't get panmode")
        };
        PanMode::from_raw(raw)
    }

    pub fn ruler_time_mode(self) -> TimeMode {
        let raw: i32 = unsafe {
            self.get_project_config("projtimemode")
                .expect("couldn't get projtimemode")
        };
        TimeMode::from_raw(raw)
    }

    pub fn transport_time_mode(self) -> TimeModeOverride {
        let raw: i32 = unsafe {
            self.get_project_config("projtimemode2")
                .expect("couldn't get projtimemode2")
        };
        TimeModeOverride::from_raw(raw)
    }

    pub fn time_offset(self) -> PositionInSeconds {
        let raw = unsafe {
            self.get_project_config("projtimeoffs")
                .expect("couldn't get projtimeoffs")
        };
        PositionInSeconds::new(raw)
    }

    pub fn measure_offset(self) -> i32 {
        unsafe {
            self.get_project_config("projmeasoffs")
                .expect("couldn't get projmeasoffs")
        }
    }

    pub fn with_track_grouping<R>(&self, on: bool, f: impl FnOnce() -> R) -> R {
        self.with_temporarily_modified_setting(
            "projtrackgroupdisabled",
            |_| if on { 0 } else { 1 },
            f,
        )
        .unwrap()
    }

    fn with_temporarily_modified_setting<'a, T: Copy + Debug, R>(
        &self,
        name: impl Into<ReaperStringArg<'a>>,
        create_new_value: impl FnOnce(T) -> T,
        f: impl FnOnce() -> R,
    ) -> Result<R, &'static str> {
        let casted_value_ref = self.get_setting_ref(name)?;
        let old_value = *casted_value_ref;
        let new_value = create_new_value(old_value);
        *casted_value_ref = new_value;
        let result = f();
        *casted_value_ref = old_value;
        Ok(result)
    }

    unsafe fn get_project_config<T: Copy>(self, name: &str) -> Option<T> {
        Some(*self.get_setting_ref(name).ok()?)
    }

    fn get_setting_ref<'a, T>(
        &self,
        name: impl Into<ReaperStringArg<'a>>,
    ) -> Result<&mut T, &'static str> {
        let reaper = Reaper::get().medium_reaper();
        let proj_conf_result = reaper
            .project_config_var_get_offs(name)
            .ok_or("setting doesn't exist")?;
        let size_matches = proj_conf_result.size as usize == mem::size_of::<T>();
        if !size_matches {
            return Err("size mismatch");
        }
        let ptr = reaper
            .project_config_var_addr(Proj(self.raw()), proj_conf_result.offset)
            .ok_or("setting exists but null pointer returned")?;
        let mut casted_value_ptr = ptr.cast::<T>();
        let casted_value_ref = unsafe { casted_value_ptr.as_mut() };
        Ok(casted_value_ref)
    }

    fn set_repeat_is_enabled(self, repeat: bool) {
        Reaper::get()
            .medium_reaper()
            .get_set_repeat_ex_set(self.context(), repeat);
    }

    fn complain_if_not_available(self) -> ReaperResult<()> {
        if !self.is_available() {
            return Err("Project not available".into());
        }
        Ok(())
    }
}

pub struct FindBookmarkResult {
    pub index: u32,
    pub index_within_type: u32,
    pub bookmark: IndexBasedBookmark,
    pub basic_info: BasicBookmarkInfo,
}

pub trait OrCurrentProject {
    fn or_current_project(&self) -> Project;
}

impl OrCurrentProject for Option<Project> {
    fn or_current_project(&self) -> Project {
        self.unwrap_or_else(|| Reaper::get().current_project())
    }
}
