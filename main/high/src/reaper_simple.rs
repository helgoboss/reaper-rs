//! This file should contain all the top-level REAPER functions which can be implemented with
//! just access to `reaper_medium::Reaper` - without all the advanced stuff like subjects,
//! channels etc. Although they end up in the same struct, this gives a little bit of structure.
use crate::error::ReaperResult;
use crate::{
    Action, Fx, FxChain, FxParameter, Guid, MidiInputDevice, MidiOutputDevice, Project, Reaper,
    ReaperError, Section,
};
use camino::Utf8PathBuf;
use helgoboss_midi::ShortMessage;
use reaper_medium::{
    AudioDeviceAttributeKey, CommandId, EnumPitchShiftModesResult, GetLastTouchedFxResult,
    GlobalAutomationModeOverride, Hwnd, Hz, MidiInputDeviceId, MidiOutputDeviceId, PitchShiftMode,
    PitchShiftSubMode, ProjectRef, ReaperStr, ReaperString, ReaperStringArg, ReaperVersion,
    ResampleMode, SectionId, StuffMidiMessageTarget, TrackLocation,
};
use std::fmt::Debug;
use std::path::PathBuf;
use std::{fs, mem, os};

impl Reaper {
    /// Gives access to the medium-level Reaper instance.
    pub fn medium_reaper(&self) -> &reaper_medium::Reaper {
        &self.medium_reaper
    }

    pub fn is_in_main_thread(&self) -> bool {
        self.medium_reaper
            .low()
            .plugin_context()
            .is_in_main_thread()
    }

    pub fn audio_device_sample_rate(&self) -> Result<Hz, &'static str> {
        let sample_rate_string = self
            .medium_reaper
            .get_audio_device_info(AudioDeviceAttributeKey::SRate, 10)
            .map_err(|e| e.message())?;
        let sample_rate_number: f64 = sample_rate_string
            .to_str()
            .parse()
            .map_err(|_| "couldn't parse sample rate")?;
        sample_rate_number
            .try_into()
            .map_err(|_| "invalid sample rate")
    }

    pub fn input_channels(&self) -> impl Iterator<Item = String> + '_ {
        (0..)
            .map(move |i| {
                self.medium_reaper()
                    .get_input_channel_name(i, |name| name.map(|n| n.to_string_lossy().to_string()))
            })
            .take_while(|r| !r.is_none())
            .map(|r| r.unwrap())
    }

    pub fn main_section(&self) -> Section {
        self.section_by_id(SectionId::new(0))
    }

    pub fn section_by_id(&self, id: SectionId) -> Section {
        Section::new(id)
    }

    pub fn monitoring_fx_chain(&self) -> FxChain {
        FxChain::from_monitoring()
    }

    pub fn last_touched_fx_parameter(&self) -> Option<FxParameter> {
        // TODO-low Sucks: We have to assume it was a parameter in the current project
        //  Maybe we should rather rely on our own technique in ControlSurface here!
        // fxQueryIndex is only a real query index since REAPER 5.95, before it didn't say if it's
        // input FX or normal one!
        self.medium_reaper()
            .get_last_touched_fx()
            .and_then(|result| {
                use GetLastTouchedFxResult::*;
                match result {
                    TrackFx {
                        track_location,
                        fx_location,
                        param_index,
                    } => {
                        // Track exists in this project
                        use TrackLocation::*;
                        let track = match track_location {
                            MasterTrack => self.current_project().master_track().ok()?,
                            NormalTrack(idx) => {
                                if idx >= self.current_project().track_count() {
                                    // Must be in another project
                                    return None;
                                }
                                self.current_project().track_by_index(idx).unwrap()
                            }
                        };
                        // TODO We should rethink the query index methods now that we have an FxRef
                        //  enum in medium-level API
                        let fx = match track.fx_by_query_index(fx_location.to_raw()) {
                            None => return None,
                            Some(fx) => fx,
                        };
                        Some(fx.parameter_by_index(param_index))
                    }
                    TakeFx { .. } => None, // TODO-low Implement,
                }
            })
    }

    pub fn resource_path(&self) -> Utf8PathBuf {
        self.medium_reaper.get_resource_path(|p| p.to_owned())
    }

    // Attention: Returns normal fx only, not input fx!
    // This is not reliable! After REAPER start no focused Fx can be found!
    #[allow(deprecated)]
    pub fn focused_fx(&self) -> Option<FocusedFxResult> {
        let reaper = self.medium_reaper();
        if reaper.low().pointers().GetFocusedFX2.is_some() {
            reaper.get_focused_fx_2().and_then(|res| {
                self.translate_focused_fx_result(res.fx, Some(res.is_still_focused))
            })
        } else {
            reaper
                .get_focused_fx()
                .and_then(|res| self.translate_focused_fx_result(res, None))
        }
    }

    fn translate_focused_fx_result(
        &self,
        fx: reaper_medium::GetFocusedFxResult,
        is_still_focused: Option<bool>,
    ) -> Option<FocusedFxResult> {
        use reaper_medium::GetFocusedFxResult::*;
        let fx = match fx {
            TakeFx { .. } => return None, // TODO-low implement
            TrackFx {
                track_location,
                fx_location,
            } => {
                // We don't know the project so we must check each project
                self.projects()
                    .filter_map(|p| {
                        let track = p.track_by_ref(track_location)?;
                        let fx = track.fx_by_query_index(fx_location.to_raw())?;
                        if fx.window_is_open() {
                            Some(fx)
                        } else {
                            None
                        }
                    })
                    .next()?
            }
            Unknown(_) => return None,
        };
        let res = FocusedFxResult {
            is_still_focused,
            fx,
        };
        Some(res)
    }

    pub fn current_project(&self) -> Project {
        Project::new(
            self.medium_reaper()
                .enum_projects(ProjectRef::Current, 0)
                .unwrap()
                .project,
        )
    }

    pub fn main_window(&self) -> Hwnd {
        self.medium_reaper().get_main_hwnd()
    }

    pub fn resample_modes(&self) -> impl Iterator<Item = &'static ReaperStr> + '_ {
        (0..)
            .map(move |i| {
                self.medium_reaper()
                    .resample_enum_modes(ResampleMode::new(i))
            })
            .take_while(|r| !r.is_none())
            .map(|r| r.unwrap())
    }

    pub fn pitch_shift_modes(
        &self,
    ) -> impl Iterator<Item = EnumPitchShiftModesResult<'static>> + '_ {
        (0..)
            .map(move |i| {
                self.medium_reaper()
                    .enum_pitch_shift_modes(PitchShiftMode::new(i))
            })
            .take_while(|r| !r.is_none())
            .map(|r| r.unwrap())
    }

    pub fn pitch_shift_sub_modes(
        &self,
        mode: PitchShiftMode,
    ) -> impl Iterator<Item = ReaperString> + '_ {
        (0..)
            .map(move |i| {
                self.medium_reaper().enum_pitch_shift_sub_modes(
                    mode,
                    PitchShiftSubMode::new(i),
                    |n| n.map(|n| n.to_reaper_string()),
                )
            })
            .take_while(|r| !r.is_none())
            .map(|r| r.unwrap())
    }

    pub fn projects(&self) -> impl Iterator<Item = Project> + '_ {
        (0..)
            .map(move |i| self.medium_reaper().enum_projects(ProjectRef::Tab(i), 0))
            .take_while(|r| !r.is_none())
            .map(|r| Project::new(r.unwrap().project))
    }

    pub fn project_count(&self) -> u32 {
        self.projects().count() as u32
    }

    pub fn version(&self) -> ReaperVersion {
        self.medium_reaper().get_app_version()
    }

    pub fn clear_console(&self) {
        self.medium_reaper().clear_console();
    }

    pub fn stuff_midi_message(&self, target: StuffMidiMessageTarget, message: impl ShortMessage) {
        self.medium_reaper().stuff_midi_message(target, message);
    }

    pub fn global_automation_override(&self) -> Option<GlobalAutomationModeOverride> {
        self.medium_reaper().get_global_automation_override()
    }

    pub fn set_global_automation_override(
        &self,
        mode_override: Option<GlobalAutomationModeOverride>,
    ) {
        self.medium_reaper()
            .set_global_automation_override(mode_override);
    }

    pub fn generate_guid(&self) -> Guid {
        Guid::new(Reaper::get().medium_reaper().gen_guid())
    }

    // It's correct that this method returns a non-optional. An id is supposed to uniquely identify
    // a device. A MidiInputDevice#isAvailable method returns if the device is actually existing
    // at runtime. That way we support (still) unloaded MidiInputDevices.

    pub fn midi_input_device_by_id(&self, id: MidiInputDeviceId) -> MidiInputDevice {
        MidiInputDevice::new(id)
    }

    // It's correct that this method returns a non-optional. An id is supposed to uniquely identify
    // a device. A MidiOutputDevice#isAvailable method returns if the device is actually
    // existing at runtime. That way we support (still) unloaded MidiOutputDevices.

    pub fn midi_output_device_by_id(&self, id: MidiOutputDeviceId) -> MidiOutputDevice {
        MidiOutputDevice::new(id)
    }

    pub fn midi_input_devices(&self) -> impl ExactSizeIterator<Item = MidiInputDevice> + '_ {
        (0..self.medium_reaper().get_max_midi_inputs())
            .map(move |i| self.midi_input_device_by_id(MidiInputDeviceId::new(i as u8)))
    }

    pub fn midi_output_devices(&self) -> impl ExactSizeIterator<Item = MidiOutputDevice> + '_ {
        (0..self.medium_reaper().get_max_midi_outputs())
            .map(move |i| self.midi_output_device_by_id(MidiOutputDeviceId::new(i as u8)))
    }

    pub fn currently_loading_or_saving_project(&self) -> Option<Project> {
        let ptr = self.medium_reaper().get_current_project_in_load_save()?;
        Some(Project::new(ptr))
    }

    // It's correct that this method returns a non-optional. A commandName is supposed to uniquely
    // identify the action, so it could be part of the resulting Action itself. An
    // Action#isAvailable method could return if the action is actually existing at runtime.
    // That way we would support (still) unloaded Actions. TODO-low Don't automatically
    // interpret command name as commandId

    pub fn action_by_command_name<'a>(
        &self,
        command_name: impl Into<ReaperStringArg<'a>>,
    ) -> Action {
        Action::command_name_based(command_name.into().into_inner().to_reaper_string())
    }

    /// # Examples
    ///
    /// ## Passing literal with zero runtime overhead
    /// ```no_compile
    /// reaper.show_console_msg(c_str!("Hello from Rust!"))
    /// ```
    /// - Uses macro `c_str!` to create new 0-terminated static literal embedded in binary
    ///
    /// ## Passing 0-terminated literal with borrowing
    /// ```no_compile
    /// let literal = "Hello from Rust!\0";
    /// reaper.show_console_msg(CStr::from_bytes_with_nul(literal.as_bytes()).unwrap())
    /// ```
    /// - You *must* make sure that the literal is 0-terminated, otherwise it will panic
    /// - Checks for existing 0 bytes
    /// - No copying involved
    ///
    /// ## Passing 0-terminated owned string with borrowing
    /// ```no_compile
    /// let owned = String::from("Hello from Rust!\0");
    /// reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap())
    /// ```
    /// - You *must* make sure that the String is 0-terminated, otherwise it will panic
    /// - Checks for existing 0 bytes
    /// - No copying involved
    ///
    /// ## Passing not 0-terminated owned string with moving
    /// ```no_compile
    /// let owned = String::from("Hello from Rust!");
    /// reaper.show_console_msg(&CString::new(owned).unwrap())
    /// ```
    /// - Moves owned string for appending 0 byte (maybe increasing String capacity)
    /// - Checks for existing 0 bytes
    /// - No copying involved
    ///
    /// ## Absolutely zero-overhead variations
    ///
    /// If you really need absolutely zero-overhead, you need to resort to unsafe functions. But
    /// this should be done only in situations when you are very constrained, e.g. in audio thread
    /// (which is forbidden to call most of the REAPER functions anyway).
    ///
    /// Look into [from_vec_unchecked](std::ffi::CString::from_vec_unchecked) or
    /// [from_bytes_with_nul_unchecked](std::ffi::CStr::from_bytes_with_nul_unchecked)
    /// respectively.
    pub fn show_console_msg<'a>(&self, msg: impl Into<ReaperStringArg<'a>>) {
        self.medium_reaper().show_console_msg(msg);
    }

    pub fn create_empty_project_in_new_tab(&self) -> Project {
        self.main_section()
            .action_by_command_id(CommandId::new(41929))
            .invoke_as_trigger(None, None)
            .expect("built-in action for creating empty project must exist");
        self.current_project()
    }

    pub fn enable_record_in_current_project(&self) {
        if self.current_project().is_recording() {
            return;
        }
        self.medium_reaper().csurf_on_record();
    }

    pub fn disable_record_in_current_project(&self) {
        if !self.current_project().is_recording() {
            return;
        }
        self.medium_reaper().csurf_on_record();
    }

    pub fn audio_is_running(&self) -> bool {
        self.medium_reaper().audio_is_running()
    }

    pub fn with_pref_pool_midi_when_duplicating<R>(&self, on: bool, f: impl FnOnce() -> R) -> R {
        // Bit 1 (2^1 = 2) of "trimmidionsplit" pref
        self.with_temporarily_modified_preference(
            "trimmidionsplit",
            |v: os::raw::c_int| if on { v | 2 } else { v & !2 },
            f,
        )
        .unwrap()
    }

    pub fn with_pref_import_as_mid_file_reference<R>(&self, on: bool, f: impl FnOnce() -> R) -> R {
        // Bit 3 (2^3 = 8) of "opencopyprompt" changes between "Import as MID file reference" (on)
        // and "Import as in-project MIDI" (off).
        self.with_temporarily_modified_preference(
            "opencopyprompt",
            |v: os::raw::c_int| if on { v | 8 } else { v & !8 },
            f,
        )
        .unwrap()
    }

    pub fn vst_scan_is_enabled(&self) -> bool {
        // 2 means that Preferences => Plug-ins => VST => "Scan new/updated plug-ins on startup" is disabled.
        // 0 means the default (scan normally in external process)
        // 1 means IIRC that the VSTs are scanned in the same process as REAPER (good for debugging)
        self.get_preference_ref("vst_scan") != Ok(&mut 2)
    }

    pub fn smooth_seek_is_enabled(&self) -> bool {
        self.get_preference_ref("smoothseek") == Ok(&mut 1)
    }

    pub fn with_smooth_seek<R>(&self, on: bool, f: impl FnOnce() -> R) -> R {
        self.with_temporarily_modified_preference("smoothseek", |_| i32::from(on), f)
            .unwrap()
    }

    pub fn with_solo_in_place<R>(&self, on: bool, f: impl FnOnce() -> R) -> R {
        self.with_temporarily_modified_preference("soloip", |_| i32::from(on), f)
            .unwrap()
    }

    /// Parses the "reaper-vstplugins*.ini" files to find the VST2 file name.
    ///
    /// Usually not necessary for REAPER >= 6.37 because one can use [`crate::FxInfo::id`] for
    /// querying and `<` + magic number for adding a FX.
    ///
    /// Attention: Just returns the first one found. Doesn't care of preferring the one in the
    /// current processor architecture.
    pub fn find_vst_file_name_by_vst2_magic_number(
        &self,
        vst_magic_number: u32,
    ) -> Option<PathBuf> {
        let mut ini_file_entries = fs::read_dir(self.resource_path())
            .ok()?
            .flatten()
            .filter_map(|entry| {
                let file_name = entry.file_name();
                let file_name = file_name.to_str()?;
                if file_name.starts_with("reaper-vstplugins") {
                    Some(entry)
                } else {
                    None
                }
            });
        let vst_magic_number = vst_magic_number.to_string();
        ini_file_entries.find_map(|e| {
            let mut reader = csv::ReaderBuilder::new()
                .flexible(true)
                .from_path(e.path())
                .ok()?;
            let record = reader
                .records()
                .flatten()
                .find(|r| r.get(1).map(|s| s.trim()) == Some(vst_magic_number.as_str()))?;
            let first_field = record.get(0)?;
            let file_name: String = first_field.chars().take_while(|ch| *ch != '=').collect();
            Some(file_name.replace('_', " ").into())
        })
    }

    fn with_temporarily_modified_preference<'a, T: Copy + Debug, R>(
        &self,
        name: impl Into<ReaperStringArg<'a>>,
        create_new_value: impl FnOnce(T) -> T,
        f: impl FnOnce() -> R,
    ) -> Result<R, &'static str> {
        let casted_value_ref = self.get_preference_ref(name)?;
        let old_value = *casted_value_ref;
        let new_value = create_new_value(old_value);
        *casted_value_ref = new_value;
        let result = f();
        *casted_value_ref = old_value;
        Ok(result)
    }

    pub fn get_preference_ref<'a, T>(
        &self,
        name: impl Into<ReaperStringArg<'a>>,
    ) -> ReaperResult<&mut T> {
        let config_var_result = Reaper::get()
            .medium_reaper
            .get_config_var(name)
            .ok_or(ReaperError::new("preference doesn't exist"))?;
        let size_matches = config_var_result.size as usize == mem::size_of::<T>();
        if !size_matches {
            return Err("size mismatch".into());
        }
        let mut casted_value_ptr = config_var_result.value.cast::<T>();
        let casted_value_ref = unsafe { casted_value_ptr.as_mut() };
        Ok(casted_value_ref)
    }
}

pub struct FocusedFxResult {
    /// If `None`, it's not clear whether the FX is still focused (older REAPER versions).
    pub is_still_focused: Option<bool>,
    pub fx: Fx,
}
