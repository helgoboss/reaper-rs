use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use std::ffi::{CStr, CString};

use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Weak};

use rxrust::prelude::*;

use crate::fx::Fx;
use crate::fx_parameter::FxParameter;
use crate::helper_control_surface::HelperControlSurface;
use crate::track_send::TrackSend;
use crate::undo_block::UndoBlock;
use crate::ActionKind::Toggleable;
use crate::{
    create_default_console_msg_formatter, create_reaper_panic_hook, create_std_logger,
    create_terminal_logger, Action, Guid, MidiInputDevice, MidiOutputDevice, Project, Reaper,
    Track,
};
use helgoboss_midi::{RawShortMessage, ShortMessage, ShortMessageType};
use once_cell::sync::Lazy;
use reaper_low::raw;

use reaper_low::ReaperPluginContext;

use reaper_medium::ProjectContext::Proj;
use reaper_medium::UndoScope::All;
use reaper_medium::{
    CommandId, GetFocusedFxResult, GetLastTouchedFxResult, GlobalAutomationModeOverride, Hwnd,
    MediumGaccelRegister, MediumHookCommand, MediumHookPostCommand, MediumOnAudioBuffer,
    MediumToggleAction, MidiInputDeviceId, MidiOutputDeviceId, OnAudioBufferArgs, ProjectRef,
    RealTimeAudioThreadScope, ReaperStringArg, ReaperVersion, StuffMidiMessageTarget,
    ToggleActionResult, TrackRef,
};
use std::fmt::{Debug, Formatter};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

// Must ONLY ever be accessed from main thread
/// Access to this variable is encapsulated in 3 functions:
/// - Reaper::setup()
/// - Reaper::get()
/// - Reaper::teardown()
///
/// We  make sure that ...
///
/// 1. Reaper::setup() is only ever called from the main thread
///    => Check when entering function
/// 2. Reaper::get() is only ever called from one thread (main thread). Even the exposed
///    Reaper reference is not mutable, the struct has members that have interior
///    mutability, and that is done via RefCells.
///    => Check when entering function
///    => Provide a separate RealtimeReaper::get()
/// 4. Reaper::teardown() is only ever called from the main thread
///    => Check when entering function
/// 5. Reaper::teardown() is not called while a get() reference is still active.
///    => Wrap Option in a RefCell.
///
/// We could also go the easy way of using one Reaper instance wrapped in a Mutex. Downside: This is
/// more guarantees than we need. Why should audio thread and main thread fight for access to one
/// Reaper instance if there can be two instances and each one has their own? That sounds a lot like
/// thread_local!, but this has the problem of being usable with a closure only.
///
/// This outer RefCell is just necessary for creating/destroying the instance safely, so most of the
/// time it will be borrowed immutably. That's why all the inner RefCells are still necessary!
// TODO-medium Main thread check is not accurate and sometimes happens a bit too late. Ideally we
//  would just save the the thread ID  of the main thread as soon as entering the main entry point
//  (can be done safely using  OnceCell). Then we could always access that main thread ID and
//  compare.
// TODO-medium In high level API, maybe make only function objects statically accessible. And
//  probably only the main thread functions object. When getting it, check if in main thread. Expose
//  a non static reference so it can only be used temporary. We shouldn't expose more than we need.
//  Protecting the root static RefCell from being accessed from multiple threads is one thing, but
//  all the inner RefCells ... that's horrible. And we can't prevent it because we want objects such
//  as Tracks and Co. to be easily cloneable. So they can easily end up in the audio thread ... but
//  wait. As long as nobody saves the reference to global Reaper instance, we should be safe. Maybe
//  a  good first step would be to expose it non static. Or even thread-local with closure? But
//  still, we shouldnt require more than necessary, so just exposing functions is good anyway.
static mut REAPER_INSTANCE: RefCell<Option<ReaperSession>> = RefCell::new(None);

// Here we don't mind having a boring mutex because this is not often accessed.
static REAPER_GUARD: Lazy<Mutex<Weak<ReaperGuard>>> = Lazy::new(|| Mutex::new(Weak::new()));

pub struct ReaperBuilder {
    medium: reaper_medium::ReaperSession,
    logger: Option<slog::Logger>,
}

impl ReaperBuilder {
    fn with_all_functions_loaded(context: ReaperPluginContext) -> ReaperBuilder {
        ReaperBuilder {
            medium: {
                let low = reaper_low::Reaper::load(context);
                reaper_medium::ReaperSession::new(low)
            },
            logger: Default::default(),
        }
    }

    pub fn logger(mut self, logger: slog::Logger) -> ReaperBuilder {
        self.logger = Some(logger);
        self
    }

    pub fn setup(self) {
        ReaperSession::setup(self.medium, self.logger.unwrap_or_else(create_std_logger));
    }
}

pub struct RealTimeReaper {
    medium: reaper_medium::Reaper<RealTimeAudioThreadScope>,
    receiver: mpsc::Receiver<AudioThreadTaskOp>,
    #[allow(dead_code)]
    sender_to_main_thread: mpsc::Sender<MainThreadTask>,
    subjects: RealTimeSubjects,
}

impl RealTimeReaper {
    pub fn midi_message_received(
        &self,
    ) -> impl LocalObservable<'static, Err = (), Item = MidiEvent<RawShortMessage>> {
        self.subjects.midi_message_received.borrow().clone()
    }
}

impl MediumOnAudioBuffer for RealTimeReaper {
    fn call(&mut self, args: OnAudioBufferArgs) {
        if args.is_post {
            return;
        }
        // TODO-medium call() must not be exposed!
        for task in self.receiver.try_iter().take(1) {
            (task)(self);
        }
        // Process MIDI
        let mut subject = self.subjects.midi_message_received.borrow_mut();
        if subject.subscribed_size() == 0 {
            return;
        }
        for i in 0..self.medium.get_max_midi_inputs() {
            self.medium
                .get_midi_input(MidiInputDeviceId::new(i as u8), |input| {
                    let evt_list = input.get_read_buf();
                    for evt in evt_list.enum_items(0) {
                        let msg = evt.message();
                        if msg.r#type() == ShortMessageType::ActiveSensing {
                            // TODO-low We should forward active sensing. Can be filtered out
                            // later.
                            continue;
                        }
                        let owned_msg: RawShortMessage = msg.to_other();
                        let owned_evt = MidiEvent::new(evt.frame_offset(), owned_msg);
                        subject.next(owned_evt);
                    }
                });
        }
    }
}

#[derive(Debug)]
pub struct ReaperSession {
    medium: RefCell<reaper_medium::ReaperSession>,
    logger: slog::Logger,
    // We take a mutable reference from this RefCell in order to add/remove commands.
    // TODO-low Adding an action in an action would panic because we have an immutable borrow of
    // the map  to obtain and execute the command, plus a mutable borrow of the map to add the
    // new command.  (the latter being unavoidable because we somehow need to modify the map!).
    //  That's not good. Is there a way to avoid this constellation? It's probably hard to avoid
    // the  immutable borrow because the `operation` is part of the map after all. And we can't
    // just  copy it before execution, at least not when it captures and mutates state, which
    // might not  be copyable (which we want to explicitly allow, that's why we accept FnMut!).
    // Or is it  possible to give up the map borrow after obtaining the command/operation
    // reference???  Look into that!!!
    command_by_id: RefCell<HashMap<CommandId, Command>>,
    pub(super) subjects: MainSubjects,
    undo_block_is_active: Cell<bool>,
    active_data: RefCell<Option<ActiveData>>,
}

impl Default for ReaperSession {
    fn default() -> Self {
        ReaperSession {
            medium: Default::default(),
            logger: create_std_logger(),
            command_by_id: Default::default(),
            subjects: Default::default(),
            undo_block_is_active: Default::default(),
            active_data: Default::default(),
        }
    }
}

#[derive(Debug)]
struct ActiveData {
    sender_to_main_thread: Sender<MainThreadTask>,
    sender_to_audio_thread: Sender<AudioThreadTaskOp>,
    csurf_inst_handle: NonNull<raw::IReaperControlSurface>,
    audio_hook_register_handle: NonNull<raw::audio_hook_register_t>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct MidiEvent<M> {
    frame_offset: u32,
    msg: M,
}

impl<M> MidiEvent<M> {
    pub fn new(frame_offset: u32, msg: M) -> MidiEvent<M> {
        MidiEvent { frame_offset, msg }
    }
}

struct RealTimeSubjects {
    midi_message_received: EventStreamSubject<MidiEvent<RawShortMessage>>,
}

impl RealTimeSubjects {
    fn new() -> RealTimeSubjects {
        fn default<T>() -> EventStreamSubject<T> {
            RefCell::new(LocalSubject::new())
        }
        RealTimeSubjects {
            midi_message_received: default(),
        }
    }
}

#[derive(Default)]
pub(super) struct MainSubjects {
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
    pub(super) fx_focused: EventStreamSubject<Payload<Option<Fx>>>,
    pub(super) fx_reordered: EventStreamSubject<Track>,
    pub(super) fx_parameter_value_changed: EventStreamSubject<FxParameter>,
    pub(super) fx_parameter_touched: EventStreamSubject<FxParameter>,
    pub(super) master_tempo_changed: EventStreamSubject<()>,
    pub(super) master_tempo_touched: EventStreamSubject<()>,
    pub(super) master_playrate_changed: EventStreamSubject<bool>,
    pub(super) master_playrate_touched: EventStreamSubject<bool>,
    pub(super) main_thread_idle: EventStreamSubject<bool>,
    pub(super) project_closed: EventStreamSubject<Project>,
    pub(super) action_invoked: EventStreamSubject<Payload<Rc<Action>>>,
}

impl Debug for MainSubjects {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MainSubjects").finish()
    }
}

#[derive(Clone)]
pub struct Payload<T>(pub T);

impl<T: Clone> PayloadCopy for Payload<T> {}

impl MainSubjects {
    fn new() -> MainSubjects {
        fn default<T>() -> EventStreamSubject<T> {
            RefCell::new(LocalSubject::new())
        }
        MainSubjects {
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

type EventStreamSubject<T> = RefCell<LocalSubject<'static, T, ()>>;

impl Drop for ReaperSession {
    fn drop(&mut self) {
        self.deactivate();
    }
}

pub struct ReaperGuard;

impl Drop for ReaperGuard {
    fn drop(&mut self) {
        ReaperSession::teardown()
    }
}

impl ReaperSession {
    pub fn guarded(initializer: impl FnOnce()) -> Arc<ReaperGuard> {
        // This is supposed to be called in the main thread. A check is not necessary, because this
        // is protected by a mutex and it will fail in the initializer if called from wrong thread.
        let mut result = REAPER_GUARD.lock().unwrap();
        if let Some(rc) = result.upgrade() {
            return rc;
        }
        initializer();
        let arc = Arc::new(ReaperGuard);
        *result = Arc::downgrade(&arc);
        arc
    }

    pub fn load(context: ReaperPluginContext) -> ReaperBuilder {
        ReaperBuilder::with_all_functions_loaded(context)
    }

    pub fn setup_with_defaults(context: ReaperPluginContext, email_address: &'static str) {
        ReaperSession::load(context)
            .logger(create_terminal_logger())
            .setup();
        std::panic::set_hook(create_reaper_panic_hook(
            create_terminal_logger(),
            Some(create_default_console_msg_formatter(email_address)),
        ));
    }

    fn setup(medium: reaper_medium::ReaperSession, logger: slog::Logger) {
        assert!(
            medium.reaper().low().plugin_context().is_in_main_thread(),
            "Reaper::setup() must be called from main thread"
        );
        assert!(
            unsafe { REAPER_INSTANCE.borrow().is_none() },
            "There's a Reaper instance already"
        );
        // We set up an (easily copyable) high-level Reaper instance and use this wherever possible.
        // ReaperSession is more complicated and should only be accessed if its functionality is
        // needed.
        Reaper::make_available_globally(Reaper::new(medium.reaper().clone()));
        let reaper = ReaperSession {
            medium: RefCell::new(medium),
            logger,
            command_by_id: RefCell::new(HashMap::new()),
            subjects: MainSubjects::new(),
            undo_block_is_active: Cell::new(false),
            active_data: RefCell::new(None),
        };
        unsafe {
            REAPER_INSTANCE.replace(Some(reaper));
        }
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
    pub fn get() -> Ref<'static, ReaperSession> {
        let reaper =
            ReaperSession::obtain_reaper_ref("Reaper::setup() must be called before Reaper::get()");
        assert!(
            !reaper.medium().reaper().is_in_real_time_audio(),
            "Reaper::get() must be called from main thread"
        );
        reaper
    }

    pub fn teardown() {
        {
            let reaper = ReaperSession::obtain_reaper_ref("There's no Reaper instance to teardown");
            assert!(
                !reaper.medium().reaper().is_in_real_time_audio(),
                "Reaper::teardown() must be called from main thread"
            );
        }
        unsafe {
            REAPER_INSTANCE.replace(None);
        }
    }

    fn obtain_reaper_ref(error_msg_if_none: &'static str) -> Ref<'static, ReaperSession> {
        Ref::map(unsafe { REAPER_INSTANCE.borrow() }, |r| match r {
            None => panic!(error_msg_if_none),
            Some(r) => r,
        })
    }

    pub fn activate(&self) {
        let mut active_data = self.active_data.borrow_mut();
        assert!(active_data.is_none(), "Reaper is already active");
        let real_time_reaper = self.medium().create_real_time_reaper();
        let (sender_to_main_thread, main_thread_receiver) = mpsc::channel::<MainThreadTask>();
        let control_surface =
            HelperControlSurface::new(sender_to_main_thread.clone(), main_thread_receiver);
        let mut medium = self.medium_mut();
        // Functions
        medium
            .plugin_register_add_hook_command::<HighLevelHookCommand>()
            .expect("couldn't register hook command");
        medium
            .plugin_register_add_toggle_action::<HighLevelToggleAction>()
            .expect("couldn't register toggle command");
        medium
            .plugin_register_add_hook_post_command::<HighLevelHookPostCommand>()
            .expect("couldn't register hook post command");
        // Audio hook
        let (sender_to_audio_thread, audio_thread_receiver) = mpsc::channel::<AudioThreadTaskOp>();
        let rt_reaper = RealTimeReaper {
            medium: real_time_reaper,
            receiver: audio_thread_receiver,
            sender_to_main_thread: sender_to_main_thread.clone(),
            subjects: RealTimeSubjects::new(),
        };
        *active_data = Some(ActiveData {
            sender_to_main_thread,
            sender_to_audio_thread,
            csurf_inst_handle: {
                medium
                    .plugin_register_add_csurf_inst(control_surface)
                    .unwrap()
            },
            audio_hook_register_handle: { medium.audio_reg_hardware_hook_add(rt_reaper).unwrap() },
        });
    }

    pub fn deactivate(&self) {
        let mut active_data = self.active_data.borrow_mut();
        let ad = match active_data.as_ref() {
            None => panic!("Reaper is not active"),
            Some(ad) => ad,
        };
        let mut medium = self.medium_mut();
        // Remove audio hook
        medium.audio_reg_hardware_hook_remove(ad.audio_hook_register_handle);
        // Remove control surface
        medium.plugin_register_remove_csurf_inst(ad.csurf_inst_handle);
        // Remove functions
        medium.plugin_register_remove_hook_post_command::<HighLevelHookPostCommand>();
        medium.plugin_register_remove_toggle_action::<HighLevelToggleAction>();
        medium.plugin_register_remove_hook_command::<HighLevelHookCommand>();
        *active_data = None;
    }

    pub fn medium(&self) -> Ref<reaper_medium::ReaperSession> {
        self.medium.borrow()
    }

    pub fn medium_mut(&self) -> RefMut<reaper_medium::ReaperSession> {
        self.medium.borrow_mut()
    }

    pub fn register_action(
        &self,
        command_name: &CStr,
        description: impl Into<ReaperStringArg<'static>>,
        operation: impl FnMut() + 'static,
        kind: ActionKind,
    ) -> RegisteredAction {
        let mut medium = self.medium_mut();
        let command_id = medium.plugin_register_add_command_id(command_name).unwrap();
        let command = Command::new(Rc::new(RefCell::new(operation)), kind);
        if let Entry::Vacant(p) = self.command_by_id.borrow_mut().entry(command_id) {
            p.insert(command);
        }
        let address = medium
            .plugin_register_add_gaccel(MediumGaccelRegister::without_key_binding(
                command_id,
                description,
            ))
            .unwrap();
        RegisteredAction::new(command_id, address)
    }

    fn unregister_command(
        &self,
        command_id: CommandId,
        gaccel_handle: NonNull<raw::gaccel_register_t>,
    ) {
        // Unregistering command when it's destroyed via RAII (implementing Drop)? Bad idea, because
        // this is the wrong point in time. The right point in time for unregistering is when it's
        // removed from the command hash map. Because even if the command still exists in memory,
        // if it's not in the map anymore, REAPER won't be able to find it.
        let mut command_by_id = self.command_by_id.borrow_mut();
        if let Some(_command) = command_by_id.get_mut(&command_id) {
            self.medium_mut()
                .plugin_register_remove_gaccel(gaccel_handle);
            command_by_id.remove(&command_id);
        }
    }

    pub fn project_switched(&self) -> impl LocalObservable<'static, Err = (), Item = Project> {
        self.subjects.project_switched.borrow().clone()
    }

    pub fn fx_opened(&self) -> impl LocalObservable<'static, Err = (), Item = Fx> {
        self.subjects.fx_opened.borrow().clone()
    }

    pub fn fx_focused(
        &self,
    ) -> impl LocalObservable<'static, Err = (), Item = Payload<Option<Fx>>> {
        self.subjects.fx_focused.borrow().clone()
    }

    pub fn track_added(&self) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects.track_added.borrow().clone()
    }

    // Delivers a GUID-based track (to still be able to identify it even it is deleted)
    pub fn track_removed(&self) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects.track_removed.borrow().clone()
    }

    pub fn track_name_changed(&self) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects.track_name_changed.borrow().clone()
    }

    pub fn master_tempo_changed(&self) -> impl LocalObservable<'static, Err = (), Item = ()> {
        self.subjects.master_tempo_changed.borrow().clone()
    }

    pub fn fx_added(&self) -> impl LocalObservable<'static, Err = (), Item = Fx> {
        self.subjects.fx_added.borrow().clone()
    }

    pub fn fx_enabled_changed(&self) -> impl LocalObservable<'static, Err = (), Item = Fx> {
        self.subjects.fx_enabled_changed.borrow().clone()
    }

    pub fn fx_reordered(&self) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects.fx_reordered.borrow().clone()
    }

    pub fn fx_removed(&self) -> impl LocalObservable<'static, Err = (), Item = Fx> {
        self.subjects.fx_removed.borrow().clone()
    }

    pub fn fx_parameter_value_changed(
        &self,
    ) -> impl LocalObservable<'static, Err = (), Item = FxParameter> {
        self.subjects.fx_parameter_value_changed.borrow().clone()
    }

    pub fn track_input_monitoring_changed(
        &self,
    ) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects
            .track_input_monitoring_changed
            .borrow()
            .clone()
    }

    pub fn track_input_changed(&self) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects.track_input_changed.borrow().clone()
    }

    pub fn track_volume_changed(&self) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects.track_volume_changed.borrow().clone()
    }

    pub fn track_pan_changed(&self) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects.track_pan_changed.borrow().clone()
    }

    pub fn track_selected_changed(&self) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects.track_selected_changed.borrow().clone()
    }

    pub fn track_mute_changed(&self) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects.track_mute_changed.borrow().clone()
    }

    pub fn track_solo_changed(&self) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects.track_solo_changed.borrow().clone()
    }

    pub fn track_arm_changed(&self) -> impl LocalObservable<'static, Err = (), Item = Track> {
        self.subjects.track_arm_changed.borrow().clone()
    }

    pub fn track_send_volume_changed(
        &self,
    ) -> impl LocalObservable<'static, Err = (), Item = TrackSend> {
        self.subjects.track_send_volume_changed.borrow().clone()
    }

    pub fn track_send_pan_changed(
        &self,
    ) -> impl LocalObservable<'static, Err = (), Item = TrackSend> {
        self.subjects.track_send_pan_changed.borrow().clone()
    }

    pub fn action_invoked(
        &self,
    ) -> impl LocalObservable<'static, Err = (), Item = Payload<Rc<Action>>> {
        self.subjects.action_invoked.borrow().clone()
    }

    // TODO-move-later
    pub fn execute_later_in_main_thread(
        &self,
        waiting_time: Duration,
        op: impl FnOnce() + 'static,
    ) -> Result<(), ()> {
        let active_data = self.active_data.borrow();
        let sender = &active_data.as_ref().ok_or(())?.sender_to_main_thread;
        sender
            .send(MainThreadTask::new(
                Box::new(op),
                Some(SystemTime::now() + waiting_time),
            ))
            .map_err(|_| ())
    }

    // TODO-move-later
    pub fn execute_later_in_main_thread_asap(&self, op: impl FnOnce() + 'static) -> Result<(), ()> {
        let active_data = self.active_data.borrow();
        let sender = &active_data.as_ref().ok_or(())?.sender_to_main_thread;
        sender
            .send(MainThreadTask::new(Box::new(op), None))
            .map_err(|_| ())
    }

    // TODO-move-later
    pub fn execute_asap_in_audio_thread(
        &self,
        op: impl FnOnce(&RealTimeReaper) + 'static,
    ) -> Result<(), ()> {
        let active_data = self.active_data.borrow();
        let sender = &active_data.as_ref().ok_or(())?.sender_to_audio_thread;
        sender.send(Box::new(op)).map_err(|_| ())
    }

    pub fn undoable_action_is_running(&self) -> bool {
        self.undo_block_is_active.get()
    }

    // Doesn't start a new block if we already are in an undo block.
    #[must_use = "Return value determines the scope of the undo block (RAII)"]
    pub(super) fn enter_undo_block_internal<'a>(
        &self,
        project: Project,
        label: &'a CStr,
    ) -> Option<UndoBlock<'a>> {
        if self.undo_block_is_active.get() {
            return None;
        }
        self.undo_block_is_active.replace(true);
        self.medium()
            .reaper()
            .undo_begin_block_2(Proj(project.get_raw()));
        Some(UndoBlock::new(project, label))
    }

    // Doesn't attempt to end a block if we are not in an undo block.
    pub(super) fn leave_undo_block_internal(&self, project: Project, label: &CStr) {
        if !self.undo_block_is_active.get() {
            return;
        }
        self.medium()
            .reaper()
            .undo_end_block_2(Proj(project.get_raw()), label, All);
        self.undo_block_is_active.replace(false);
    }
}

struct Command {
    /// Reasoning for that type (from inner to outer):
    /// - `FnMut`: We don't use just `fn` because we want to support closures. We don't use just
    ///   `Fn` because we want to support closures that keep mutable references to their captures.
    ///   We can't accept `FnOnce` because that would mean that the closure value itself is
    ///   consumed when it's called. That means we would have to remove the action from the action
    ///   list just to call it and we couldn't again it again.
    /// - `Box`: Of course we want to support very different closures with very different captures.
    ///   We don't use generic type parameters to achieve that because we need to put Commands into
    ///   a HashMap as values - so we need each Command to have the same size in memory and the
    ///   same type. Generics lead to the generation of different types and most likely also
    ///   different sizes. We don't use references because we want ownership. Yes, Box is (like
    ///   reference) a so-called trait object and therefore uses dynamic dispatch. It also needs
    ///   heap allocation (unlike general references). However, this is exactly what we want and
    ///   need here.
    /// - `RefCell`: We need this in order to make the FnMut callable in immutable context (for
    ///   safety reasons we are mostly in immutable context, see ControlSurface documentation).
    ///   It's good to use `RefCell` in a very fine-grained way like that and not for example on
    ///   the whole `Command`. That allows for very localized mutation and therefore a lower
    ///   likelihood that borrowing rules are violated (or if we wouldn't have the runtime borrow
    ///   checking of `RefCell`, the likeliness to get undefined behavior).
    /// - `Rc`: We don't want to keep an immutable reference to the surrounding `Command` around
    ///   just in order to execute this operation! Why? Because we want to support operations which
    ///   add a REAPER action when executed. And when doing that, we of course have to borrow the
    ///   command HashMap mutably. However, at that point we already have an immutable borrow to
    ///   the complete HashMap (via a `RefCell`) ... boom. Panic! With the `Rc` we can release the
    ///   borrow by cloning the first `Rc` instance and therefore gaining a short-term second
    ///   ownership of that operation.
    /// - Wait ... actually there's no `Box` anymore! Turned out that `Rc` makes all things
    ///   possible that also `Box` makes possible, in particular taking dynamically-sized types. If
    ///   we wouldn't need `Rc` (for shared references), we would have to take `Box` instead.
    operation: Rc<RefCell<dyn FnMut()>>,
    kind: ActionKind,
}

impl Debug for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").finish()
    }
}

impl Command {
    fn new(operation: Rc<RefCell<dyn FnMut()>>, kind: ActionKind) -> Command {
        Command { operation, kind }
    }
}

pub struct RegisteredAction {
    // For identifying the registered command (= the functions to be executed)
    command_id: CommandId,
    // For identifying the registered action (= description, related keyboard shortcuts etc.)
    gaccel_handle: NonNull<raw::gaccel_register_t>,
}

impl RegisteredAction {
    fn new(
        command_id: CommandId,
        gaccel_handle: NonNull<raw::gaccel_register_t>,
    ) -> RegisteredAction {
        RegisteredAction {
            command_id,
            gaccel_handle,
        }
    }

    pub fn unregister(&self) {
        ReaperSession::get().unregister_command(self.command_id, self.gaccel_handle);
    }
}

// Called by REAPER (using a delegate function)!
// Only for main section
struct HighLevelHookCommand {}

impl MediumHookCommand for HighLevelHookCommand {
    fn call(command_id: CommandId, _flag: i32) -> bool {
        // TODO-low Pass on flag
        let operation = match ReaperSession::get().command_by_id.borrow().get(&command_id) {
            Some(command) => command.operation.clone(),
            None => return false,
        };
        (*operation).borrow_mut().call_mut(());
        true
    }
}

// Called by REAPER directly (using a delegate function)!
// Only for main section
struct HighLevelHookPostCommand {}

impl MediumHookPostCommand for HighLevelHookPostCommand {
    fn call(command_id: CommandId, _flag: i32) {
        let action = Reaper::get()
            .get_main_section()
            .get_action_by_command_id(command_id);
        ReaperSession::get()
            .subjects
            .action_invoked
            .borrow_mut()
            .next(Payload(Rc::new(action)));
    }
}

// Called by REAPER directly!
// Only for main section
struct HighLevelToggleAction {}

impl MediumToggleAction for HighLevelToggleAction {
    fn call(command_id: CommandId) -> ToggleActionResult {
        if let Some(command) = ReaperSession::get()
            .command_by_id
            .borrow()
            .get(&(command_id))
        {
            match &command.kind {
                ActionKind::Toggleable(is_on) => {
                    if is_on() {
                        ToggleActionResult::On
                    } else {
                        ToggleActionResult::Off
                    }
                }
                ActionKind::NotToggleable => ToggleActionResult::NotRelevant,
            }
        } else {
            ToggleActionResult::NotRelevant
        }
    }
}

type AudioThreadTaskOp = Box<dyn FnOnce(&RealTimeReaper) + 'static>;

type MainThreadTaskOp = Box<dyn FnOnce() + 'static>;

pub(super) struct MainThreadTask {
    pub desired_execution_time: Option<std::time::SystemTime>,
    pub op: MainThreadTaskOp,
}

impl MainThreadTask {
    pub fn new(
        op: MainThreadTaskOp,
        desired_execution_time: Option<std::time::SystemTime>,
    ) -> MainThreadTask {
        MainThreadTask {
            desired_execution_time,
            op,
        }
    }
}
