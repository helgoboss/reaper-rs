use std::borrow::{Borrow, BorrowMut, Cow};
use std::cell::{Ref, RefCell, RefMut, Cell};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_ushort, c_void, c_int};
use std::ptr::{null, null_mut};
use std::sync::{Once, mpsc};
use num_enum::IntoPrimitive;

use c_str_macro::c_str;

use crate::high_level::ActionKind::Toggleable;
use crate::high_level::{Project, Section, Track, create_std_logger, create_terminal_logger, create_reaper_panic_hook, create_default_console_msg_formatter, Action, Guid, MidiInputDevice, MidiOutputDevice, UndoBlock, MessageBoxKind, MessageBoxResult};
use crate::low_level::{ACCEL, gaccel_register_t, MediaTrack, ReaProject, firewall, ReaperPluginContext, HWND, audio_hook_register_t, midi_Input_GetReadBuf, MIDI_eventlist_EnumItems, MIDI_event_t};
use crate::low_level;
use crate::medium_level;
use rxrust::subscriber::Subscriber;
use crate::high_level::helper_control_surface::HelperControlSurface;
use rxrust::subscription::SubscriptionLike;
use rxrust::prelude::*;
use rxrust::subject::{LocalSubjectObserver, SubjectValue};
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};
use slog::Level::Debug;
use std::thread;
use std::thread::ThreadId;
use crate::high_level::track_send::TrackSend;
use crate::high_level::fx::Fx;
use crate::high_level::automation_mode::AutomationMode;
use std::convert::{TryFrom, TryInto};
use crate::high_level::fx_parameter::FxParameter;

// See https://doc.rust-lang.org/std/sync/struct.Once.html why this is safe in combination with Once
static mut REAPER_INSTANCE: Option<Reaper> = None;
static INIT_REAPER_INSTANCE: Once = Once::new();

// Called by REAPER directly!
// Only for main section
extern "C" fn hook_command(command_index: i32, flag: i32) -> bool {
    firewall(|| {
        let mut operation = match Reaper::instance().command_by_index.borrow().get(&(command_index as u32)) {
            Some(command) => command.operation.clone(),
            None => return false
        };
        (*operation).borrow_mut().call_mut(());
        true
    }).unwrap_or(false)
}

// Called by REAPER directly!
// Only for main section
extern "C" fn hook_post_command(command_id: i32, flag: i32) {
    firewall(|| {
        let reaper = Reaper::instance();
        let action = reaper.get_main_section().get_action_by_command_id(command_id);
        reaper.subjects.action_invoked.borrow_mut().next(Rc::new(action));
    });
}

// Called by REAPER directly!
// Only for main section
extern "C" fn toggle_action(command_index: i32) -> i32 {
    firewall(|| {
        if let Some(command) = Reaper::instance().command_by_index.borrow().get(&(command_index as u32)) {
            match &command.kind {
                ActionKind::Toggleable(is_on) => if is_on() { 1 } else { 0 },
                ActionKind::NotToggleable => -1
            }
        } else {
            -1
        }
    }).unwrap_or(-1)
}

// Called by REAPER directly!
extern "C" fn process_audio_buffer(is_post: bool, len: i32, srate: f64, reg: *mut audio_hook_register_t) {
    // TODO Check performance implications for firewall call
    firewall(|| {
        if is_post {
            return;
        }
        // TODO Check performance implications for Reaper instance unwrapping
        let reaper = Reaper::instance();
        // TODO Should we use an unsafe cell here for better performance?
        let mut subject = reaper.subjects.midi_message_received.borrow_mut();
        // TODO IMPORTANT Use early return if nobody is subscribed to incoming MIDI events
        for i in 0..reaper.get_max_midi_input_devices() {
            let dev = reaper.medium.get_midi_input(i);
            if dev.is_null() {
                continue;
            }
            let dev = unsafe { &mut *dev };
            let midi_events = unsafe { midi_Input_GetReadBuf(dev as *mut _) };
            let mut bpos = 0;
            loop {
                let midi_event = unsafe { MIDI_eventlist_EnumItems(midi_events, &mut bpos as *mut c_int) };
                if midi_event.is_null() {
                    // No MIDI messages left
                    break;
                }
                let midi_event = unsafe { &*midi_event };
                if midi_event.midi_message[0] == 254 {
                    // Active sensing, we don't want to forward that TODO maybe yes?
                    break;
                }
                subject.next(midi_event);
            }
        }
    });
}

//pub(super) type Task = Box<dyn FnOnce() + Send + 'static>;
pub(super) type Task = Box<dyn FnOnce() + 'static>;

pub struct ReaperBuilder {
    medium: medium_level::Reaper,
    logger: Option<slog::Logger>,
}

impl ReaperBuilder {
    fn with_all_functions_loaded(context: ReaperPluginContext) -> ReaperBuilder {
        ReaperBuilder {
            medium: {
                let low = low_level::Reaper::with_all_functions_loaded(context.function_provider);
                medium_level::Reaper::new(low)
            },
            logger: Default::default(),
        }
    }

    fn with_custom_medium(medium: medium_level::Reaper) -> ReaperBuilder {
        ReaperBuilder {
            medium,
            logger: Default::default(),
        }
    }

    pub fn logger(mut self, logger: slog::Logger) -> ReaperBuilder {
        self.logger = Some(logger);
        self
    }

    pub fn setup(self) {
        Reaper::setup(
            self.medium,
            self.logger.unwrap_or_else(create_std_logger),
        );
    }
}

pub fn setup_all_with_defaults(context: ReaperPluginContext, email_address: &'static str) {
    Reaper::with_all_functions_loaded(context)
        .logger(create_terminal_logger())
        .setup();
    std::panic::set_hook(create_reaper_panic_hook(
        create_terminal_logger(),
        Some(create_default_console_msg_formatter(email_address)),
    ));
}

pub struct Reaper {
    pub medium: medium_level::Reaper,
    pub logger: slog::Logger,
    // We take a mutable reference from this RefCell in order to add/remove commands.
    // TODO Adding an action in an action would panic because we have an immutable borrow of the map
    //  to obtain and execute the command, plus a mutable borrow of the map to add the new command.
    //  (the latter being unavoidable because we somehow need to modify the map!).
    //  That's not good. Is there a way to avoid this constellation? It's probably hard to avoid the
    //  immutable borrow because the `operation` is part of the map after all. And we can't just
    //  copy it before execution, at least not when it captures and mutates state, which might not
    //  be copyable (which we want to explicitly allow, that's why we accept FnMut!). Or is it
    //  possible to give up the map borrow after obtaining the command/operation reference???
    //  Look into that!!!
    command_by_index: RefCell<HashMap<u32, Command>>,
    pub(super) subjects: EventStreamSubjects,
    task_sender: Sender<Task>,
    main_thread_id: ThreadId,
    undo_block_is_active: Cell<bool>,
    audio_hook: audio_hook_register_t,
}

pub(super) struct EventStreamSubjects {
    // This is a RefCell. So calling next() while another next() is still running will panic.
    // I guess it's good that way because this is very generic code, panicking or not panicking
    // depending on the user's code. And getting a panic is good for becoming aware of the problem
    // instead of running into undefined behavior. The developer can always choose to defer to
    // the next `ControlSurface::run()` invocation (execute things in next main loop cycle).
    pub(super) project_switched: EventStreamSubject<Project>,
    pub(super) track_volume_changed: EventStreamSubject<Track>,
    pub(super) track_volume_touched: EventStreamSubject<Track>,
    pub(super) track_pan_changed: EventStreamSubject<Track>,
    pub(super) track_pan_touched: EventStreamSubject<Track>,
    pub(super) track_send_volume_changed: EventStreamSubject<TrackSend>,
    pub(super) track_send_volume_touched: EventStreamSubject<TrackSend>,
    pub(super) track_send_pan_changed: EventStreamSubject<TrackSend>,
    pub(super) track_send_pan_touched: EventStreamSubject<TrackSend>,
    pub(super) track_added: EventStreamSubject<Track>,
    pub(super) track_removed: EventStreamSubject<Track>,
    pub(super) tracks_reordered: EventStreamSubject<Project>,
    pub(super) track_name_changed: EventStreamSubject<Track>,
    pub(super) track_input_changed: EventStreamSubject<Track>,
    pub(super) track_input_monitoring_changed: EventStreamSubject<Track>,
    pub(super) track_arm_changed: EventStreamSubject<Track>,
    pub(super) track_mute_changed: EventStreamSubject<Track>,
    pub(super) track_mute_touched: EventStreamSubject<Track>,
    pub(super) track_solo_changed: EventStreamSubject<Track>,
    pub(super) track_selected_changed: EventStreamSubject<Track>,
    pub(super) fx_added: EventStreamSubject<Fx>,
    pub(super) fx_removed: EventStreamSubject<Fx>,
    pub(super) fx_enabled_changed: EventStreamSubject<Fx>,
    pub(super) fx_opened: EventStreamSubject<Fx>,
    pub(super) fx_closed: EventStreamSubject<Fx>,
    pub(super) fx_focused: EventStreamSubject<Option<Fx>>,
    pub(super) fx_reordered: EventStreamSubject<Track>,
    pub(super) fx_parameter_value_changed: EventStreamSubject<FxParameter>,
    pub(super) fx_parameter_touched: EventStreamSubject<FxParameter>,
    pub(super) master_tempo_changed: EventStreamSubject<bool>,
    pub(super) master_tempo_touched: EventStreamSubject<bool>,
    pub(super) master_playrate_changed: EventStreamSubject<bool>,
    pub(super) master_playrate_touched: EventStreamSubject<bool>,
    pub(super) main_thread_idle: EventStreamSubject<bool>,
    pub(super) project_closed: EventStreamSubject<Project>,
    pub(super) action_invoked: EventStreamSubject<Rc<Action>>,
    pub(super) midi_message_received: EventStreamSubject<*const MIDI_event_t>,
}


impl EventStreamSubjects {
    fn new() -> EventStreamSubjects {
        fn default<T>() -> EventStreamSubject<T> {
            RefCell::new(Subject::local())
        }
        EventStreamSubjects {
            project_switched: default(),
            track_volume_changed: default(),
            track_volume_touched: default(),
            track_pan_changed: default(),
            track_pan_touched: default(),
            track_send_volume_changed: default(),
            track_send_volume_touched: default(),
            track_send_pan_changed: default(),
            track_send_pan_touched: default(),
            track_added: default(),
            track_removed: default(),
            tracks_reordered: default(),
            track_name_changed: default(),
            track_input_changed: default(),
            track_input_monitoring_changed: default(),
            track_arm_changed: default(),
            track_mute_changed: default(),
            track_mute_touched: default(),
            track_solo_changed: default(),
            track_selected_changed: default(),
            fx_added: default(),
            fx_removed: default(),
            fx_enabled_changed: default(),
            fx_opened: default(),
            fx_closed: default(),
            fx_focused: default(),
            fx_reordered: default(),
            fx_parameter_value_changed: default(),
            fx_parameter_touched: default(),
            master_tempo_changed: default(),
            master_tempo_touched: default(),
            master_playrate_changed: default(),
            master_playrate_touched: default(),
            main_thread_idle: default(),
            project_closed: default(),
            action_invoked: default(),
            midi_message_received: default(),
        }
    }
}

pub enum ActionKind {
    NotToggleable,
    Toggleable(Box<dyn Fn() -> bool>),
}

pub fn toggleable(is_on: impl Fn() -> bool + 'static) -> ActionKind {
    Toggleable(Box::new(is_on))
}

type EventStreamSubject<T> = RefCell<EventStream<T>>;
type EventStream<T> = LocalSubject<'static, SubjectValue<T>, SubjectValue<()>>;

impl Drop for Reaper {
    fn drop(&mut self) {
        self.deactivate();
    }
}

// TODO Maybe don't rely on static reference (We don't even know for sure if REAPER guarantees that)
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReaperVersion {
    internal: &'static CStr,
}

// TODO Working with C strings is a bit exaggerated in case of versions where, we don't have special
//  characters which could cause problems
impl From<&'static CStr> for ReaperVersion {
    fn from(internal: &'static CStr) -> Self {
        ReaperVersion {
            internal
        }
    }
}

impl Reaper {
    pub fn with_all_functions_loaded(context: ReaperPluginContext) -> ReaperBuilder {
        ReaperBuilder::with_all_functions_loaded(context)
    }

    // TODO Make pub when the time has come
    fn with_custom_medium(medium: medium_level::Reaper) -> ReaperBuilder {
        ReaperBuilder::with_custom_medium(medium)
    }

    fn setup(medium: medium_level::Reaper, logger: slog::Logger) {
        let (task_sender, task_receiver) = mpsc::channel::<Task>();
        let reaper = Reaper {
            medium,
            logger,
            command_by_index: RefCell::new(HashMap::new()),
            subjects: EventStreamSubjects::new(),
            task_sender,
            main_thread_id: thread::current().id(),
            undo_block_is_active: Cell::new(false),
            audio_hook: audio_hook_register_t {
                OnAudioBuffer: Some(process_audio_buffer),
                userdata1: null_mut(),
                userdata2: null_mut(),
                input_nch: 0,
                output_nch: 0,
                GetBuffer: None,
            },
        };
        unsafe {
            INIT_REAPER_INSTANCE.call_once(|| {
                REAPER_INSTANCE = Some(reaper);
            });
        }
        Reaper::instance().init(task_receiver);
    }

    fn init(&self, task_receiver: Receiver<Task>) {
        self.medium.install_control_surface(HelperControlSurface::new(task_receiver));
    }

    // Must be idempotent
    pub fn activate(&self) {
        self.medium.plugin_register(c_str!("hookcommand"), hook_command as *mut c_void);
        self.medium.plugin_register(c_str!("toggleaction"), toggle_action as *mut c_void);
        self.medium.plugin_register(c_str!("hookpostcommand"), hook_post_command as *mut c_void);
        self.medium.register_control_surface();
        self.medium.audio_reg_hardware_hook(true, &self.audio_hook as *const _);
    }

    // Must be idempotent
    pub fn deactivate(&self) {
        self.medium.audio_reg_hardware_hook(false, &self.audio_hook as *const _);
        self.medium.unregister_control_surface();
        self.medium.plugin_register(c_str!("-hookpostcommand"), hook_post_command as *mut c_void);
        self.medium.plugin_register(c_str!("-toggleaction"), toggle_action as *mut c_void);
        self.medium.plugin_register(c_str!("-hookcommand"), hook_command as *mut c_void);
    }

    pub fn get_version(&self) -> ReaperVersion {
        ReaperVersion {
            internal: self.medium.get_app_version()
        }
    }

    pub fn generate_guid(&self) -> Guid {
        Guid::new(Reaper::instance().medium.gen_guid())
    }

    // Allowing global access to native REAPER functions at all times is valid in my opinion.
    // Because REAPER itself is not written in Rust and therefore cannot take part in Rust's compile
    // time guarantees anyway. We need to rely on REAPER at that point and also take care not to do
    // something which is not allowed in Reaper (by reading documentation and learning from
    // mistakes ... no compiler is going to save us from them). REAPER as a whole is always mutable
    // from the perspective of extensions.
    //
    // We express that in Rust by making `Reaper` class an immutable (in the sense of non-`&mut`)
    // singleton and allowing all REAPER functions to be called from an immutable context ...
    // although they can and often will lead to mutations within REAPER!
    // TODO Consider naming this get()
    pub fn instance() -> &'static Reaper {
        unsafe {
            REAPER_INSTANCE.as_ref().unwrap()
        }
    }

    pub fn register_action(
        &self,
        command_id: &CStr,
        description: impl Into<Cow<'static, CStr>>,
        operation: impl FnMut() + 'static,
        kind: ActionKind,
    ) -> RegisteredAction
    {
        let command_index = self.medium.plugin_register(c_str!("command_id"), command_id.as_ptr() as *mut c_void) as u32;
        let command = Command::new(command_index, description.into(), Rc::new(RefCell::new(operation)), kind);
        self.register_command(command_index, command);
        RegisteredAction::new(command_index)
    }

    fn register_command(&self, command_index: u32, command: Command) {
        if let Entry::Vacant(p) = self.command_by_index.borrow_mut().entry(command_index) {
            let command = p.insert(command);
            let acc = &mut command.accelerator_register;
            self.medium.plugin_register(c_str!("gaccel"), acc as *mut _ as *mut c_void);
        }
    }

    fn unregister_command(&self, command_index: u32) {
        // TODO Use RAII
        let mut command_by_index = self.command_by_index.borrow_mut();
        if let Some(command) = command_by_index.get_mut(&command_index) {
            let acc = &mut command.accelerator_register;
            self.medium.plugin_register(c_str!("-gaccel"), acc as *mut _ as *mut c_void);
            command_by_index.remove(&command_index);
        }
    }

    pub fn get_max_midi_input_devices(&self) -> u32 {
        self.medium.get_max_midi_inputs()
    }

    pub fn get_max_midi_output_devices(&self) -> u32 {
        self.medium.get_max_midi_outputs()
    }

    // It's correct that this method returns a non-optional. An id is supposed to uniquely identify a device.
    // A MidiInputDevice#isAvailable method returns if the device is actually existing at runtime. That way we
    // support (still) unloaded MidiInputDevices.
    pub fn get_midi_input_device_by_id(&self, id: u32) -> MidiInputDevice {
        MidiInputDevice::new(id)
    }

    // It's correct that this method returns a non-optional. An id is supposed to uniquely identify a device.
    // A MidiOutputDevice#isAvailable method returns if the device is actually existing at runtime. That way we
    // support (still) unloaded MidiOutputDevices.
    pub fn get_midi_output_device_by_id(&self, id: u32) -> MidiOutputDevice {
        MidiOutputDevice::new(id)
    }

    pub fn get_midi_input_devices(&self) -> impl Iterator<Item=MidiInputDevice> + '_ {
        (0..self.get_max_midi_input_devices())
            .map(move |i| self.get_midi_input_device_by_id(i))
            // TODO I think we should also return unavailable devices. Client can filter easily.
            .filter(|d| d.is_available())
    }

    pub fn get_midi_output_devices(&self) -> impl Iterator<Item=MidiOutputDevice> + '_ {
        (0..self.get_max_midi_output_devices())
            .map(move |i| self.get_midi_output_device_by_id(i))
            // TODO I think we should also return unavailable devices. Client can filter easily.
            .filter(|d| d.is_available())
    }

    pub fn get_currently_loading_or_saving_project(&self) -> Option<Project> {
        let ptr = self.medium.get_current_project_in_load_save();
        if ptr.is_null() {
            return None;
        }
        Some(Project::new(ptr))
    }

    // It's correct that this method returns a non-optional. A commandName is supposed to uniquely identify the action,
    // so it could be part of the resulting Action itself. An Action#isAvailable method could return if the action is
    // actually existing at runtime. That way we would support (still) unloaded Actions.
    // TODO Don't automatically interpret command name as commandId
    pub fn get_action_by_command_name(&self, command_name: CString) -> Action {
        Action::command_name_based(command_name)
    }

    /// # Examples
    ///
    /// ## Passing literal with zero runtime overhead
    /// ```
    /// reaper.show_console_msg(c_str!("Hello from Rust!"))
    /// ```
    /// - Uses macro `c_str!` to create new 0-terminated static literal embedded in binary
    ///
    /// ## Passing 0-terminated literal with borrowing
    /// ```
    /// let literal = "Hello from Rust!\0";
    /// reaper.show_console_msg(CStr::from_bytes_with_nul(literal.as_bytes()).unwrap())
    /// ```
    /// - You *must* make sure that the literal is 0-terminated, otherwise it will panic
    /// - Checks for existing 0 bytes
    /// - No copying involved
    ///
    /// ## Passing 0-terminated owned string with borrowing
    /// ```
    /// let owned = String::from("Hello from Rust!\0");
    /// reaper.show_console_msg(CStr::from_bytes_with_nul(owned.as_bytes()).unwrap())
    /// ```
    /// - You *must* make sure that the String is 0-terminated, otherwise it will panic
    /// - Checks for existing 0 bytes
    /// - No copying involved
    ///
    /// ## Passing not 0-terminated owned string with moving
    /// ```
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
    /// (which is forbidden to call most of the REAPER SDK functions anyway).
    ///
    /// Look into [from_vec_unchecked](CString::from_vec_unchecked) or
    /// [from_bytes_with_nul_unchecked](CStr::from_bytes_with_nul_unchecked) respectively.
    pub fn show_console_msg(&self, msg: &CStr) {
        self.medium.show_console_msg(msg);
    }

    // type 0=OK,1=OKCANCEL,2=ABORTRETRYIGNORE,3=YESNOCANCEL,4=YESNO,5=RETRYCANCEL : ret 1=OK,2=CANCEL,3=ABORT,4=RETRY,5=IGNORE,6=YES,7=NO
    pub fn show_message_box(&self, msg: &CStr, title: &CStr, kind: MessageBoxKind) -> MessageBoxResult {
        self.medium.show_message_box(msg, title, kind.into()).try_into().expect("Unknown message box result")
    }

    pub fn get_main_section(&self) -> Section {
        Section::new(self.medium.section_from_unique_id(0))
    }

    pub fn create_empty_project_in_new_tab(&self) -> Project {
        self.get_main_section().get_action_by_command_id(41929).invoke_as_trigger(None);
        self.get_current_project()
    }

    pub fn project_switched(&self) -> EventStream<Project> {
        self.subjects.project_switched.borrow().fork()
    }

    pub fn track_added(&self) -> EventStream<Track> {
        self.subjects.track_added.borrow().fork()
    }

    pub fn midi_message_received(&self) -> EventStream<*const MIDI_event_t> {
        self.subjects.midi_message_received.borrow().fork()
    }

    pub fn track_removed(&self) -> EventStream<Track> {
        self.subjects.track_removed.borrow().fork()
    }

    pub fn track_name_changed(&self) -> EventStream<Track> {
        self.subjects.track_name_changed.borrow().fork()
    }

    // TODO bool is not useful here
    pub fn master_tempo_changed(&self) -> EventStream<bool> {
        self.subjects.master_tempo_changed.borrow().fork()
    }

    pub fn track_input_monitoring_changed(&self) -> EventStream<Track> {
        self.subjects.track_input_monitoring_changed.borrow().fork()
    }

    pub fn track_input_changed(&self) -> EventStream<Track> {
        self.subjects.track_input_changed.borrow().fork()
    }

    pub fn track_volume_changed(&self) -> EventStream<Track> {
        self.subjects.track_volume_changed.borrow().fork()
    }

    pub fn track_pan_changed(&self) -> EventStream<Track> {
        self.subjects.track_pan_changed.borrow().fork()
    }

    pub fn track_selected_changed(&self) -> EventStream<Track> {
        self.subjects.track_selected_changed.borrow().fork()
    }

    pub fn track_mute_changed(&self) -> EventStream<Track> {
        self.subjects.track_mute_changed.borrow().fork()
    }

    pub fn track_solo_changed(&self) -> EventStream<Track> {
        self.subjects.track_solo_changed.borrow().fork()
    }

    pub fn track_arm_changed(&self) -> EventStream<Track> {
        self.subjects.track_arm_changed.borrow().fork()
    }

    pub fn track_send_volume_changed(&self) -> EventStream<TrackSend> {
        self.subjects.track_send_volume_changed.borrow().fork()
    }

    pub fn track_send_pan_changed(&self) -> EventStream<TrackSend> {
        self.subjects.track_send_pan_changed.borrow().fork()
    }

    pub fn action_invoked(&self) -> EventStream<Rc<Action>> {
        self.subjects.action_invoked.borrow().fork()
    }

    pub fn get_current_project(&self) -> Project {
        let (rp, _) = self.medium.enum_projects(-1, 0);
        Project::new(rp)
    }

    pub fn get_main_window(&self) -> HWND {
        self.medium.get_main_hwnd()
    }

    pub fn get_projects(&self) -> impl Iterator<Item=Project> + '_ {
        (0..)
            .map(move |i| self.medium.enum_projects(i, 0).0)
            .take_while(|p| !p.is_null())
            .map(|p| { Project::new(p) })
    }

    pub fn get_project_count(&self) -> u32 {
        self.get_projects().count() as u32
    }

    pub fn clear_console(&self) {
        self.medium.clear_console();
    }

    // TODO Require Send?
    pub fn execute_later_in_main_thread(&self, task: impl FnOnce() + 'static) {
        self.task_sender.send(Box::new(task));
    }

    // TODO Require Send?
    pub fn execute_when_in_main_thread(&self, task: impl FnOnce() + 'static) {
        if self.current_thread_is_main_thread() {
            task();
        } else {
            self.execute_later_in_main_thread(task);
        }
    }

    pub fn stuff_midi_message(&self, target: StuffMidiMessageTarget, message: (u8, u8, u8)) {
        self.medium.stuff_midimessage(target.into(), message.0 as i32, message.1 as i32, message.2 as i32);
    }

    pub fn current_thread_is_main_thread(&self) -> bool {
        thread::current().id() == self.main_thread_id
    }

    pub fn get_main_thread_id(&self) -> ThreadId {
        self.main_thread_id
    }

    pub fn get_global_automation_override(&self) -> AutomationMode {
        let am = self.medium.get_global_automation_override();
        AutomationMode::try_from(am).expect("Unknown automation mode")
    }

    pub fn undoable_action_is_running(&self) -> bool {
        self.undo_block_is_active.get()
    }

    // Doesn't start a new block if we already are in an undo block.
    pub(super) fn enter_undo_block_internal<'a>(&self, project: Project, label: &'a CStr) -> Option<UndoBlock<'a>> {
        if self.undo_block_is_active.get() {
            return None;
        }
        self.undo_block_is_active.replace(true);
        self.medium.undo_begin_block_2(project.get_rea_project());
        Some(UndoBlock::new(project, label))
    }

    // Doesn't attempt to end a block if we are not in an undo block.
    pub(super) fn leave_undo_block_internal(&self, project: &Project, label: &CStr) {
        if !self.undo_block_is_active.get() {
            return;
        }
        self.medium.undo_end_block_2(project.get_rea_project(), label, -1);
        self.undo_block_is_active.replace(false);
    }
}

struct Command {
    description: Cow<'static, CStr>,
    /// Reasoning for that type (from inner to outer):
    /// - `FnMut`: We don't use just `fn` because we want to support closures. We don't use just
    ///   `Fn` because we want to support closures that keep mutable references to their captures.
    ///   TODO What about supporting also FnOnce?
    /// - `Box`: Of course we want to support very different closures with very different captures.
    ///   We don't use generic type parameters to achieve that because we need to put Commands into
    ///   a HashMap as values - so we need each Command to have the same size in memory and the same
    ///   type. Generics lead to the generation of different types and most likely also different
    ///   sizes. We don't use references because we want ownership. Yes, Box is (like reference) a
    ///   so-called trait object and therefore uses dynamic dispatch. It also needs heap allocation
    ///   (unlike general references). However, this is exactly what we want and need here.
    /// - `RefCell`: We need this in order to make the FnMut callable in immutable context (for
    ///   safety reasons we are mostly in immutable context, see ControlSurface documentation). It's
    ///   good to use `RefCell` in a very fine-grained way like that and not for example on the whole
    ///   `Command`. That allows for very localized mutation and therefore a lower likelihood that
    ///   borrowing rules are violated (or if we wouldn't have the runtime borrow checking of
    ///   `RefCell`, the likeliness to get undefined behavior).
    /// - `Rc`: We don't want to keep an immutable reference to the surrounding `Command` around
    ///   just in order to execute this operation! Why? Because we want to support operations which
    ///   add a REAPER action when executed. And when doing that, we of course have to borrow
    ///   the command HashMap mutably. However, at that point we already have an immutable borrow
    ///   to the complete HashMap (via a `RefCell`) ... boom. Panic! With the `Rc` we can release
    ///   the borrow by cloning the first `Rc` instance and therefore gaining a short-term
    ///   second ownership of that operation.
    /// - Wait ... actually there's no `Box` anymore! Turned out that `Rc` makes all things
    ///   possible that also `Box` makes possible, in particular taking dynamically-sized types.
    ///   If we wouldn't need `Rc` (for shared references), we would have to take `Box` instead.
    operation: Rc<RefCell<dyn FnMut()>>,
    kind: ActionKind,
    accelerator_register: gaccel_register_t,
}

impl Command {
    fn new(command_index: u32, description: Cow<'static, CStr>, operation: Rc<RefCell<dyn FnMut()>>, kind: ActionKind) -> Command {
        let mut c = Command {
            description,
            operation,
            kind,
            accelerator_register: gaccel_register_t {
                accel: ACCEL {
                    fVirt: 0,
                    key: 0,
                    cmd: command_index as c_ushort,
                },
                desc: null(),
            },
        };
        c.accelerator_register.desc = c.description.as_ptr();
        c
    }
}

pub struct RegisteredAction {
    command_index: u32,
}

impl RegisteredAction {
    fn new(command_index: u32) -> RegisteredAction {
        RegisteredAction {
            command_index,
        }
    }

    pub fn unregister(&self) {
        Reaper::instance().unregister_command(self.command_index);
    }
}

#[derive(Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum StuffMidiMessageTarget {
    VirtualMidiKeyboard,
    MidiAsControlInputQueue,
    VirtualMidiKeyboardOnCurrentChannel
}

