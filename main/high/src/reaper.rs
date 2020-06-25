use crate::ReactiveEvent;
use std::cell::{Cell, RefCell, RefMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;



use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{Arc, Weak};

use rxrust::prelude::*;

use crate::fx::Fx;
use crate::fx_parameter::FxParameter;
use crate::helper_control_surface::HelperControlSurface;
use crate::track_send::TrackSend;
use crate::undo_block::UndoBlock;
use crate::ActionKind::Toggleable;
use crate::{
    create_default_console_msg_formatter, create_reaper_panic_hook, create_std_logger,
    create_terminal_logger, Action, Project, Spawner, Track,
};
use helgoboss_midi::{RawShortMessage, ShortMessage, ShortMessageType};
use once_cell::sync::Lazy;
use reaper_low::raw;

use reaper_low::PluginContext;

use crate::run_loop_executor::new_spawner_and_executor;
use crate::run_loop_scheduler::{RunLoopScheduler, RxTask};
use crossbeam_channel::{Receiver, Sender};
use reaper_medium::ProjectContext::Proj;
use reaper_medium::UndoScope::All;
use reaper_medium::{
    CommandId, HookCommand, HookPostCommand, MidiFrameOffset, MidiInputDeviceId, OnAudioBuffer,
    OnAudioBufferArgs, OwnedGaccelRegister, ProjectRef, RealTimeAudioThreadScope, ReaperStr,
    ReaperString, ReaperStringArg, RegistrationHandle, ToggleAction, ToggleActionResult,
};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

/// Capacity of the channel which is used to scheduled tasks for execution in the main thread.
///
/// Should probably be a bit more than MAX_AUDIO_THREAD_TASKS because the audio callback is
/// usually executed more often and therefore can produce faster. Plus, the main thread also
/// uses this very often to schedule tasks for a later execution in the main thread.
///
/// Shouldn't be too high because when `Reaper::deactivate()` is called, those tasks are
/// going to pile up - and they will be discarded on the next activate.
const MAIN_THREAD_TASK_CHANNEL_CAPACITY: usize = 1000;

/// How many tasks to process at a maximum in one main loop iteration.
pub(crate) const MAIN_THREAD_TASK_BULK_SIZE: usize = 100;

/// Capacity of the channel which is used to scheduled tasks for execution in the real-time audio
/// thread.
const AUDIO_THREAD_TASK_CHANNEL_CAPACITY: usize = 500;

/// How many tasks to process at a maximum in one real-time audio loop iteration.
const AUDIO_THREAD_TASK_BULK_SIZE: usize = 1;

/// We  make sure in each public function/method that it's called from the correct thread. Similar
/// with other methods. We basically make this struct thread-safe by panicking whenever we are in
/// the wrong thread.
///
/// We could also go the easy way of using one Reaper instance wrapped in a Mutex. Downside: This is
/// more guarantees than we need. Why should audio thread and main thread fight for access to one
/// Reaper instance. That results in performance loss.
//
// This is safe (see https://doc.rust-lang.org/std/sync/struct.Once.html#examples-1).
static mut INSTANCE: Option<Reaper> = None;
static INIT_INSTANCE: std::sync::Once = std::sync::Once::new();

// Here we don't mind having a heavy mutex because this is not often accessed.
static REAPER_GUARD: Lazy<Mutex<Weak<ReaperGuard>>> = Lazy::new(|| Mutex::new(Weak::new()));

pub struct ReaperBuilder {
    medium: reaper_medium::ReaperSession,
    logger: Option<slog::Logger>,
}

impl ReaperBuilder {
    fn new(context: PluginContext) -> ReaperBuilder {
        ReaperBuilder {
            medium: {
                let low = reaper_low::Reaper::load(context);
                reaper_medium::ReaperSession::new(low)
            },
            logger: Default::default(),
        }
    }

    pub fn logger(mut self, logger: slog::Logger) -> ReaperBuilder {
        self.require_main_thread();
        self.logger = Some(logger);
        self
    }

    /// This has an effect only if there isn't an instance already.
    pub fn setup(self) {
        self.require_main_thread();
        unsafe {
            INIT_INSTANCE.call_once(|| {
                let (mt_sender, mt_receiver) =
                    crossbeam_channel::bounded::<MainThreadTask>(MAIN_THREAD_TASK_CHANNEL_CAPACITY);
                let (mt_rx_sender, mt_rx_receiver) =
                    crossbeam_channel::bounded::<RxTask>(MAIN_THREAD_TASK_CHANNEL_CAPACITY);
                let (spawner, executor) = new_spawner_and_executor(
                    MAIN_THREAD_TASK_CHANNEL_CAPACITY,
                    MAIN_THREAD_TASK_BULK_SIZE,
                );
                let main_thread_scheduler = RunLoopScheduler::new(mt_rx_sender.clone());
                let (at_sender, at_receiver) = crossbeam_channel::bounded::<AudioThreadTaskOp>(
                    AUDIO_THREAD_TASK_CHANNEL_CAPACITY,
                );
                let logger = self.logger.unwrap_or_else(create_std_logger);
                let medium_reaper = self.medium.reaper().clone();
                let current_project = Project::new(
                    medium_reaper
                        .enum_projects(ProjectRef::Current, 0)
                        .unwrap()
                        .project,
                );
                let version = medium_reaper.get_app_version();
                let medium_real_time_reaper = self.medium.create_real_time_reaper();
                let reaper = Reaper {
                    medium_session: RefCell::new(self.medium),
                    medium_reaper,
                    medium_real_time_reaper: medium_real_time_reaper.clone(),
                    logger,
                    command_by_id: RefCell::new(HashMap::new()),
                    subjects: MainSubjects::new(),
                    undo_block_is_active: Cell::new(false),
                    main_thread_task_sender: mt_sender.clone(),
                    audio_thread_task_sender: at_sender,
                    main_thread_rx_task_sender: mt_rx_sender,
                    main_thread_scheduler,
                    main_thread_future_spawner: spawner,
                    session_status: RefCell::new(SessionStatus::Sleeping(Some(SleepingState {
                        csurf_inst: Box::new(HelperControlSurface::new(
                            version,
                            current_project,
                            mt_sender.clone(),
                            mt_receiver,
                            mt_rx_receiver,
                            executor,
                        )),
                        audio_hook: Box::new(HighOnAudioBuffer {
                            task_receiver: at_receiver,
                            reaper: RealTimeReaper {
                                medium_reaper: medium_real_time_reaper,
                                midi_message_received: LocalSubject::new(),
                                main_thread_task_sender: mt_sender,
                            },
                        }),
                    }))),
                };
                INSTANCE = Some(reaper)
            });
        }
    }

    fn require_main_thread(&self) {
        require_main_thread(self.medium.reaper().low().plugin_context());
    }
}

pub struct RealTimeReaper {
    medium_reaper: reaper_medium::Reaper<RealTimeAudioThreadScope>,
    midi_message_received: LocalSubject<'static, MidiEvent<RawShortMessage>, ()>,
    #[allow(unused)]
    main_thread_task_sender: Sender<MainThreadTask>,
}

impl RealTimeReaper {
    pub fn midi_message_received(&self) -> impl ReactiveEvent<MidiEvent<RawShortMessage>> {
        self.midi_message_received.clone()
    }
}

struct HighOnAudioBuffer {
    task_receiver: Receiver<AudioThreadTaskOp>,
    reaper: RealTimeReaper,
}

impl HighOnAudioBuffer {
    pub fn discard_tasks(&self) {
        let task_count = self.task_receiver.try_iter().count();
        if task_count > 0 {
            slog::warn!(Reaper::get().logger(), "Discarded audio thread tasks on reactivation";
                "task_count" => task_count,
            );
        }
    }
}

impl OnAudioBuffer for HighOnAudioBuffer {
    fn call(&mut self, args: OnAudioBufferArgs) {
        if args.is_post {
            return;
        }
        // Take only one task each time because we don't want to do to much in one go in the
        // real-time thread.
        for task in self
            .task_receiver
            .try_iter()
            .take(AUDIO_THREAD_TASK_BULK_SIZE)
        {
            (task)(&self.reaper);
        }
        // Process MIDI
        let subject = &mut self.reaper.midi_message_received;
        if subject.subscribed_size() == 0 {
            return;
        }
        for i in 0..self.reaper.medium_reaper.get_max_midi_inputs() {
            self.reaper
                .medium_reaper
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
pub struct Reaper {
    medium_session: RefCell<reaper_medium::ReaperSession>,
    pub(crate) medium_reaper: reaper_medium::Reaper,
    pub(crate) medium_real_time_reaper: reaper_medium::Reaper<RealTimeAudioThreadScope>,
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
    pub(crate) subjects: MainSubjects,
    undo_block_is_active: Cell<bool>,
    main_thread_task_sender: Sender<MainThreadTask>,
    audio_thread_task_sender: Sender<AudioThreadTaskOp>,
    main_thread_rx_task_sender: Sender<RxTask>,
    main_thread_scheduler: RunLoopScheduler,
    main_thread_future_spawner: Spawner,
    session_status: RefCell<SessionStatus>,
}

#[derive(Debug)]
enum SessionStatus {
    Sleeping(Option<SleepingState>),
    Awake(AwakeState),
}

struct SleepingState {
    csurf_inst: Box<HelperControlSurface>,
    audio_hook: Box<HighOnAudioBuffer>,
}

impl Debug for SleepingState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("SleepingState")
            .field("csurf_inst", &self.csurf_inst)
            .field("audio_hook", &"<omitted>")
            .finish()
    }
}

#[derive(Debug)]
struct AwakeState {
    csurf_inst_handle: RegistrationHandle<HelperControlSurface>,
    audio_hook_register_handle: RegistrationHandle<HighOnAudioBuffer>,
    gaccel_registers: HashMap<CommandId, NonNull<raw::gaccel_register_t>>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct MidiEvent<M> {
    frame_offset: MidiFrameOffset,
    msg: M,
}

impl<M> MidiEvent<M> {
    pub fn new(frame_offset: MidiFrameOffset, msg: M) -> MidiEvent<M> {
        MidiEvent { frame_offset, msg }
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
    pub(super) fx_focused: EventStreamSubject<Option<Fx>>,
    pub(super) fx_reordered: EventStreamSubject<Track>,
    pub(super) fx_parameter_value_changed: EventStreamSubject<FxParameter>,
    pub(super) fx_parameter_touched: EventStreamSubject<FxParameter>,
    pub(super) fx_preset_changed: EventStreamSubject<Fx>,
    pub(super) master_tempo_changed: EventStreamSubject<()>,
    pub(super) master_tempo_touched: EventStreamSubject<()>,
    pub(super) master_playrate_changed: EventStreamSubject<()>,
    pub(super) master_playrate_touched: EventStreamSubject<()>,
    pub(super) main_thread_idle: EventStreamSubject<()>,
    pub(super) project_closed: EventStreamSubject<Project>,
    pub(super) action_invoked: EventStreamSubject<Rc<Action>>,
}

impl Debug for MainSubjects {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MainSubjects").finish()
    }
}

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
            fx_preset_changed: default(),
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

pub struct ReaperGuard;

impl Drop for ReaperGuard {
    fn drop(&mut self) {
        let _ = Reaper::get().go_to_sleep();
    }
}

static GUARD_INITIALIZER: std::sync::Once = std::sync::Once::new();

impl Reaper {
    /// The given initializer is executed only the first time this is called.
    ///
    /// `activate()` is called whenever first first instance pops up. `deactivate()` is called
    /// whenver the last instance goes away.
    pub fn guarded(initializer: impl FnOnce()) -> Arc<ReaperGuard> {
        // This is supposed to be called in the main thread. A check is not necessary, because this
        // is protected by a mutex and it will fail in the initializer and getter if called from
        // wrong thread.
        let mut result = REAPER_GUARD.lock().unwrap();
        if let Some(rc) = result.upgrade() {
            // There's at least one active instance. No need to reactivate.
            return rc;
        }
        // There's no active instance.
        GUARD_INITIALIZER.call_once(|| {
            // If this is called, there was never an active instance in this session. Initialize!
            initializer();
        });
        let _ = Reaper::get().wake_up();
        let arc = Arc::new(ReaperGuard);
        *result = Arc::downgrade(&arc);
        arc
    }

    /// Returns the builder for further configuration.
    pub fn load(context: PluginContext) -> ReaperBuilder {
        require_main_thread(&context);
        ReaperBuilder::new(context)
    }

    /// This has an effect only if there isn't an instance already.
    pub fn setup_with_defaults(context: PluginContext, email_address: &'static str) {
        require_main_thread(&context);
        Reaper::load(context)
            .logger(create_terminal_logger())
            .setup();
        std::panic::set_hook(create_reaper_panic_hook(
            create_terminal_logger(),
            Some(create_default_console_msg_formatter(email_address)),
        ));
    }

    /// May be called from any thread.
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
    pub fn get() -> &'static Reaper {
        unsafe {
            INSTANCE
                .as_ref()
                .expect("Reaper::load().setup() must be called before Reaper::get()")
        }
    }

    pub fn logger(&self) -> &slog::Logger {
        &self.logger
    }

    pub fn wake_up(&self) -> Result<(), &'static str> {
        self.require_main_thread();
        let mut session_status = self.session_status.borrow_mut();
        let sleeping_state = match session_status.deref_mut() {
            SessionStatus::Awake(_) => return Err("Session is already awake"),
            SessionStatus::Sleeping(state) => state.take(),
        };
        let sleeping_state = match sleeping_state {
            None => return Err("Previous wake-up left session in invalid state"),
            Some(s) => s,
        };
        // We don't want to execute tasks which accumulated during the "downtime" of Reaper.
        // So we just consume all without executing them.
        sleeping_state.csurf_inst.init();
        sleeping_state.csurf_inst.discard_tasks();
        sleeping_state.audio_hook.discard_tasks();
        // Functions
        let mut medium = self.medium_session();
        medium
            .plugin_register_add_hook_command::<HighLevelHookCommand>()
            .map_err(|_| "couldn't register hook command")?;
        medium
            .plugin_register_add_toggle_action::<HighLevelToggleAction>()
            .map_err(|_| "couldn't register toggle command")?;
        medium
            .plugin_register_add_hook_post_command::<HighLevelHookPostCommand>()
            .map_err(|_| "couldn't register hook post command")?;
        *session_status = SessionStatus::Awake(AwakeState {
            gaccel_registers: self
                .command_by_id
                .borrow()
                .iter()
                .map(|(id, command)| {
                    let handle = medium
                        .plugin_register_add_gaccel(OwnedGaccelRegister::without_key_binding(
                            *id,
                            command.description.clone(),
                        ))
                        .unwrap();
                    (*id, handle)
                })
                .collect(),
            csurf_inst_handle: {
                medium
                    .plugin_register_add_csurf_inst(sleeping_state.csurf_inst)
                    .map_err(|_| "Control surface registration failed")?
            },
            audio_hook_register_handle: {
                medium
                    .audio_reg_hardware_hook_add(sleeping_state.audio_hook)
                    .map_err(|_| "Audio hook registration failed")?
            },
        });
        Ok(())
    }

    pub fn go_to_sleep(&self) -> Result<(), &'static str> {
        self.require_main_thread();
        let mut session_status = self.session_status.borrow_mut();
        let awake_state = match session_status.deref() {
            SessionStatus::Sleeping(_) => return Err("Session is already sleeping"),
            SessionStatus::Awake(s) => s,
        };
        let mut medium = self.medium_session();
        // Remove audio hook
        let audio_hook = medium
            .audio_reg_hardware_hook_remove(awake_state.audio_hook_register_handle)
            .ok_or("audio hook was not registered")?;
        // Remove control surface
        let csurf_inst = unsafe {
            medium
                .plugin_register_remove_csurf_inst(awake_state.csurf_inst_handle)
                .ok_or("control surface was not registered")?
        };
        // Unregister actions
        for gaccel_handle in awake_state.gaccel_registers.values() {
            medium.plugin_register_remove_gaccel(*gaccel_handle);
        }
        // Remove functions
        medium.plugin_register_remove_hook_post_command::<HighLevelHookPostCommand>();
        medium.plugin_register_remove_toggle_action::<HighLevelToggleAction>();
        medium.plugin_register_remove_hook_command::<HighLevelHookCommand>();
        *session_status = SessionStatus::Sleeping(Some(SleepingState {
            csurf_inst,
            audio_hook,
        }));
        Ok(())
    }

    pub fn medium_session(&self) -> RefMut<reaper_medium::ReaperSession> {
        self.require_main_thread();
        self.medium_session.borrow_mut()
    }

    pub fn register_action(
        &self,
        command_name: impl Into<ReaperStringArg<'static>>,
        description: impl Into<ReaperStringArg<'static>>,
        operation: impl FnMut() + 'static,
        kind: ActionKind,
    ) -> RegisteredAction {
        self.require_main_thread();
        let mut medium = self.medium_session();
        let command_id = medium.plugin_register_add_command_id(command_name).unwrap();
        let description = description.into().into_inner();
        let command = Command::new(
            Rc::new(RefCell::new(operation)),
            kind,
            description.to_reaper_string(),
        );
        if let Entry::Vacant(p) = self.command_by_id.borrow_mut().entry(command_id) {
            p.insert(command);
        }
        let registered_action = RegisteredAction::new(command_id);
        // Immediately register if active
        let mut session_status = self.session_status.borrow_mut();
        let awake_state = match session_status.deref_mut() {
            SessionStatus::Sleeping(_) => return registered_action,
            SessionStatus::Awake(s) => s,
        };
        let address = medium
            .plugin_register_add_gaccel(OwnedGaccelRegister::without_key_binding(
                command_id,
                description.into_owned(),
            ))
            .unwrap();
        awake_state.gaccel_registers.insert(command_id, address);
        registered_action
    }

    fn unregister_action(&self, command_id: CommandId) {
        // Unregistering command when it's destroyed via RAII (implementing Drop)? Bad idea, because
        // this is the wrong point in time. The right point in time for unregistering is when it's
        // removed from the command hash map. Because even if the command still exists in memory,
        // if it's not in the map anymore, REAPER won't be able to find it.
        self.command_by_id.borrow_mut().remove(&command_id);
        // Unregister if active
        let mut session_status = self.session_status.borrow_mut();
        let awake_state = match session_status.deref_mut() {
            SessionStatus::Sleeping(_) => return,
            SessionStatus::Awake(s) => s,
        };
        if let Some(gaccel_handle) = awake_state.gaccel_registers.get(&command_id) {
            self.medium_session()
                .plugin_register_remove_gaccel(*gaccel_handle);
        }
    }

    pub fn main_thread_idle(&self) -> impl ReactiveEvent<()> {
        self.require_main_thread();
        self.subjects.main_thread_idle.borrow().clone()
    }

    pub fn project_switched(&self) -> impl ReactiveEvent<Project> {
        self.require_main_thread();
        self.subjects.project_switched.borrow().clone()
    }

    pub fn fx_opened(&self) -> impl ReactiveEvent<Fx> {
        self.require_main_thread();
        self.subjects.fx_opened.borrow().clone()
    }

    pub fn fx_focused(&self) -> impl ReactiveEvent<Option<Fx>> {
        self.require_main_thread();
        self.subjects.fx_focused.borrow().clone()
    }

    pub fn track_added(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_added.borrow().clone()
    }

    // Delivers a GUID-based track (to still be able to identify it even it is deleted)
    pub fn track_removed(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_removed.borrow().clone()
    }

    pub fn track_name_changed(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_name_changed.borrow().clone()
    }

    pub fn master_tempo_changed(&self) -> impl ReactiveEvent<()> {
        self.require_main_thread();
        self.subjects.master_tempo_changed.borrow().clone()
    }

    pub fn master_tempo_touched(&self) -> impl ReactiveEvent<()> {
        self.require_main_thread();
        self.subjects.master_tempo_touched.borrow().clone()
    }

    pub fn master_playrate_changed(&self) -> impl ReactiveEvent<()> {
        self.require_main_thread();
        self.subjects.master_playrate_changed.borrow().clone()
    }

    pub fn master_playrate_touched(&self) -> impl ReactiveEvent<()> {
        self.require_main_thread();
        self.subjects.master_playrate_touched.borrow().clone()
    }

    pub fn fx_added(&self) -> impl ReactiveEvent<Fx> {
        self.require_main_thread();
        self.subjects.fx_added.borrow().clone()
    }

    pub fn fx_enabled_changed(&self) -> impl ReactiveEvent<Fx> {
        self.require_main_thread();
        self.subjects.fx_enabled_changed.borrow().clone()
    }

    pub fn fx_reordered(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.fx_reordered.borrow().clone()
    }

    pub fn fx_removed(&self) -> impl ReactiveEvent<Fx> {
        self.require_main_thread();
        self.subjects.fx_removed.borrow().clone()
    }

    pub fn fx_parameter_value_changed(&self) -> impl ReactiveEvent<FxParameter> {
        self.require_main_thread();
        self.subjects.fx_parameter_value_changed.borrow().clone()
    }

    pub fn fx_parameter_touched(&self) -> impl ReactiveEvent<FxParameter> {
        self.require_main_thread();
        self.subjects.fx_parameter_touched.borrow().clone()
    }

    pub fn fx_preset_changed(&self) -> impl ReactiveEvent<Fx> {
        self.require_main_thread();
        self.subjects.fx_preset_changed.borrow().clone()
    }

    pub fn track_input_monitoring_changed(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects
            .track_input_monitoring_changed
            .borrow()
            .clone()
    }

    pub fn track_input_changed(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_input_changed.borrow().clone()
    }

    pub fn track_volume_changed(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_volume_changed.borrow().clone()
    }

    pub fn track_volume_touched(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_volume_touched.borrow().clone()
    }

    pub fn track_pan_changed(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_pan_changed.borrow().clone()
    }

    pub fn track_pan_touched(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_pan_touched.borrow().clone()
    }

    pub fn track_selected_changed(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_selected_changed.borrow().clone()
    }

    pub fn track_mute_changed(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        // TODO-medium Use try_borrow() and emit a helpful error message, e.g.
        //  "Don't subscribe to an event x while this event is raised! Defer the subscription."
        self.subjects.track_mute_changed.borrow().clone()
    }

    pub fn track_mute_touched(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_mute_touched.borrow().clone()
    }

    pub fn track_solo_changed(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_solo_changed.borrow().clone()
    }

    pub fn track_arm_changed(&self) -> impl ReactiveEvent<Track> {
        self.require_main_thread();
        self.subjects.track_arm_changed.borrow().clone()
    }

    pub fn track_send_volume_changed(&self) -> impl ReactiveEvent<TrackSend> {
        self.require_main_thread();
        self.subjects.track_send_volume_changed.borrow().clone()
    }

    pub fn track_send_volume_touched(&self) -> impl ReactiveEvent<TrackSend> {
        self.require_main_thread();
        self.subjects.track_send_volume_touched.borrow().clone()
    }

    pub fn track_send_pan_changed(&self) -> impl ReactiveEvent<TrackSend> {
        self.require_main_thread();
        self.subjects.track_send_pan_changed.borrow().clone()
    }

    pub fn track_send_pan_touched(&self) -> impl ReactiveEvent<TrackSend> {
        self.require_main_thread();
        self.subjects.track_send_pan_touched.borrow().clone()
    }

    pub fn action_invoked(&self) -> impl ReactiveEvent<Rc<Action>> {
        self.require_main_thread();
        self.subjects.action_invoked.borrow().clone()
    }

    /// Returns an rxRust scheduler for scheduling observables.
    pub fn main_thread_scheduler(&self) -> &RunLoopScheduler {
        &self.main_thread_scheduler
    }

    /// Spawns a future for execution in main thread.
    pub fn spawn_in_main_thread(
        &self,
        future: impl std::future::Future<Output = ()> + 'static + Send,
    ) {
        let spawner = &self.main_thread_future_spawner;
        spawner.spawn(future);
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread(
        &self,
        waiting_time: Duration,
        op: impl FnOnce() + 'static,
    ) -> Result<(), ()> {
        let sender = &self.main_thread_task_sender;
        sender
            .send(MainThreadTask::new(
                Box::new(op),
                Some(SystemTime::now() + waiting_time),
            ))
            .map_err(|_| ())
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_in_main_thread_asap(&self, op: impl FnOnce() + 'static) -> Result<(), ()> {
        if Reaper::get().is_in_main_thread() {
            op();
            Ok(())
        } else {
            self.do_later_in_main_thread_asap(op)
        }
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread_asap(&self, op: impl FnOnce() + 'static) -> Result<(), ()> {
        let sender = &self.main_thread_task_sender;
        sender
            .send(MainThreadTask::new(Box::new(op), None))
            .map_err(|_| ())
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_real_time_audio_thread_asap(
        &self,
        op: impl FnOnce(&RealTimeReaper) + 'static,
    ) -> Result<(), ()> {
        let sender = &self.audio_thread_task_sender;
        sender.send(Box::new(op)).map_err(|_| ())
    }

    pub fn undoable_action_is_running(&self) -> bool {
        self.require_main_thread();
        self.undo_block_is_active.get()
    }

    // Doesn't start a new block if we already are in an undo block.
    #[must_use = "Return value determines the scope of the undo block (RAII)"]
    pub(super) fn enter_undo_block_internal<'a>(
        &self,
        project: Project,
        label: &'a ReaperStr,
    ) -> Option<UndoBlock<'a>> {
        self.require_main_thread();
        if self.undo_block_is_active.get() {
            return None;
        }
        self.undo_block_is_active.replace(true);
        self.medium_reaper().undo_begin_block_2(Proj(project.raw()));
        Some(UndoBlock::new(project, label))
    }

    // Doesn't attempt to end a block if we are not in an undo block.
    pub(super) fn leave_undo_block_internal(&self, project: Project, label: &ReaperStr) {
        self.require_main_thread();
        if !self.undo_block_is_active.get() {
            return;
        }
        self.medium_reaper()
            .undo_end_block_2(Proj(project.raw()), label, All);
        self.undo_block_is_active.replace(false);
    }

    pub fn require_main_thread(&self) {
        require_main_thread(Reaper::get().medium_reaper().low().plugin_context());
    }
}

unsafe impl Sync for Reaper {}

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
    description: ReaperString,
}

impl Debug for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").finish()
    }
}

impl Command {
    fn new(
        operation: Rc<RefCell<dyn FnMut()>>,
        kind: ActionKind,
        description: ReaperString,
    ) -> Command {
        Command {
            operation,
            kind,
            description,
        }
    }
}

pub struct RegisteredAction {
    // For identifying the registered command (= the functions to be executed)
    command_id: CommandId,
}

impl RegisteredAction {
    fn new(command_id: CommandId) -> RegisteredAction {
        RegisteredAction { command_id }
    }

    pub fn unregister(&self) {
        require_main_thread(Reaper::get().medium_reaper().low().plugin_context());
        Reaper::get().unregister_action(self.command_id);
    }
}

// Called by REAPER (using a delegate function)!
// Only for main section
struct HighLevelHookCommand {}

impl HookCommand for HighLevelHookCommand {
    fn call(command_id: CommandId, _flag: i32) -> bool {
        // TODO-low Pass on flag
        let operation = match Reaper::get().command_by_id.borrow().get(&command_id) {
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

impl HookPostCommand for HighLevelHookPostCommand {
    fn call(command_id: CommandId, _flag: i32) {
        let action = Reaper::get()
            .main_section()
            .action_by_command_id(command_id);
        Reaper::get()
            .subjects
            .action_invoked
            .borrow_mut()
            .next(Rc::new(action));
    }
}

// Called by REAPER directly!
// Only for main section
struct HighLevelToggleAction {}

impl ToggleAction for HighLevelToggleAction {
    fn call(command_id: CommandId) -> ToggleActionResult {
        if let Some(command) = Reaper::get().command_by_id.borrow().get(&(command_id)) {
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

fn require_main_thread(context: &PluginContext) {
    assert!(
        context.is_in_main_thread(),
        "this function must be called in the main thread"
    );
}
