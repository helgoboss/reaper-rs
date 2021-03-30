//! This file should contain all the top-level REAPER functions which can be implemented with
//! just access to `reaper_medium::Reaper` - without all the advanced stuff like subjects,
//! channels etc. Although they end up in the same struct, this gives a little bit of structure.
use crate::{
    Action, Fx, FxChain, FxParameter, Guid, MidiInputDevice, MidiOutputDevice, Project, Reaper,
    Section,
};
use helgoboss_midi::ShortMessage;
use reaper_medium::{
    CommandId, GetLastTouchedFxResult, GlobalAutomationModeOverride, Hwnd, MidiInputDeviceId,
    MidiOutputDeviceId, ProjectRef, ReaperStringArg, ReaperVersion, SectionId,
    StuffMidiMessageTarget, TrackLocation,
};
use std::path::PathBuf;

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

    pub fn main_section(&self) -> Section {
        Section::new(SectionId::new(0))
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
                            MasterTrack => self.current_project().master_track(),
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

    pub fn resource_path(&self) -> PathBuf {
        self.medium_reaper.get_resource_path(|p| p.to_owned())
    }

    // Attention: Returns normal fx only, not input fx!
    // This is not reliable! After REAPER start no focused Fx can be found!
    pub fn focused_fx(&self) -> Option<Fx> {
        self.medium_reaper().get_focused_fx().and_then(|res| {
            use reaper_medium::GetFocusedFxResult::*;
            match res {
                TakeFx { .. } => None, // TODO-low implement
                TrackFx {
                    track_location,
                    fx_location,
                } => {
                    // We don't know the project so we must check each project
                    self.projects()
                        .filter_map(|p| {
                            let track = p.track_by_ref(track_location)?;
                            let fx = track.fx_by_query_index(fx_location.to_raw())?;
                            if fx.window_is_open() { Some(fx) } else { None }
                        })
                        .next()
                }
                Unknown(_) => None,
            }
        })
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

    pub fn midi_input_devices(
        &self,
    ) -> impl Iterator<Item = MidiInputDevice> + ExactSizeIterator + '_ {
        (0..self.medium_reaper().get_max_midi_inputs())
            .map(move |i| self.midi_input_device_by_id(MidiInputDeviceId::new(i as u8)))
    }

    pub fn midi_output_devices(
        &self,
    ) -> impl Iterator<Item = MidiOutputDevice> + ExactSizeIterator + '_ {
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
            .invoke_as_trigger(None);
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
}
