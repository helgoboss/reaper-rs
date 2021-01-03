use crate::{local_run_loop_executor, run_loop_executor, CrashInfo, MiddlewareControlSurface};
use std::cell::{Cell, RefCell, RefMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{Arc, Weak};

use crate::helper_middleware::{HelperMiddleware, HelperTask};
use crate::undo_block::UndoBlock;
use crate::ActionKind::Toggleable;
use crate::{
    create_default_console_msg_formatter, create_reaper_panic_hook, create_std_logger, Project,
};
use once_cell::sync::Lazy;
use reaper_low::{raw, register_plugin_destroy_hook};

use reaper_low::PluginContext;

use crossbeam_channel::{Receiver, Sender};
use futures::channel::oneshot;
use reaper_medium::ProjectContext::Proj;
use reaper_medium::UndoScope::All;
use reaper_medium::{
    ActionValueChange, CommandId, HookCommand, HookPostCommand2, OnAudioBuffer, OnAudioBufferArgs,
    OwnedGaccelRegister, ReaProject, RealTimeAudioThreadScope, ReaperStr, ReaperString,
    ReaperStringArg, RegistrationHandle, SectionContext, ToggleAction, ToggleActionResult,
    WindowContext,
};
use slog::{debug, o, Logger};
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

/// We  make sure in **each** public function/method that it's called from the correct thread.
/// Similar with other methods. We basically make this struct thread-safe by panicking whenever we
/// are in the wrong thread.
///
/// We could also go the easy way of using one Reaper instance wrapped in a Mutex. Downside: This is
/// more guarantees than we need. Why should audio thread and main thread fight for access to one
/// Reaper instance. That results in performance loss and possible deadlocks.
//
// This is safe (see https://doc.rust-lang.org/std/sync/struct.Once.html#examples-1).
static mut INSTANCE: Option<Reaper> = None;

/// This value can be set more than once and we don't necessarily have REAPER API access at our
/// disposal when accessing it, that's why we can't use `call_once` in combination with thread check
/// in order to get safe access. Let's use a Mutex instead.
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
        static INIT_INSTANCE: std::sync::Once = std::sync::Once::new();
        self.require_main_thread();
        unsafe {
            INIT_INSTANCE.call_once(|| {
                let (mt_sender, mt_receiver) =
                    crossbeam_channel::bounded::<MainThreadTask>(MAIN_THREAD_TASK_CHANNEL_CAPACITY);
                let (spawner, executor) = run_loop_executor::new_spawner_and_executor(
                    MAIN_THREAD_TASK_CHANNEL_CAPACITY,
                    MAIN_THREAD_TASK_BULK_SIZE,
                );
                let (local_spawner, local_executor) =
                    local_run_loop_executor::new_spawner_and_executor(
                        MAIN_THREAD_TASK_CHANNEL_CAPACITY,
                        MAIN_THREAD_TASK_BULK_SIZE,
                    );
                let (at_sender, at_receiver) = crossbeam_channel::bounded::<AudioThreadTaskOp>(
                    AUDIO_THREAD_TASK_CHANNEL_CAPACITY,
                );
                let (helper_task_sender, helper_task_receiver) = crossbeam_channel::unbounded();
                let logger = self.logger.unwrap_or_else(create_std_logger);
                let medium_reaper = self.medium.reaper().clone();
                let medium_real_time_reaper = self.medium.create_real_time_reaper();
                let reaper = Reaper {
                    medium_session: RefCell::new(self.medium),
                    medium_reaper,
                    medium_real_time_reaper,
                    logger: logger.clone(),
                    command_by_id: RefCell::new(HashMap::new()),
                    action_value_change_history: RefCell::new(Default::default()),
                    undo_block_is_active: Cell::new(false),
                    main_thread_task_sender: mt_sender.clone(),
                    audio_thread_task_sender: at_sender,
                    helper_task_sender,
                    main_thread_future_spawner: spawner,
                    local_main_thread_future_spawner: local_spawner,
                    session_status: RefCell::new(SessionStatus::Sleeping(Some(SleepingState {
                        csurf_inst: Box::new(MiddlewareControlSurface::new(HelperMiddleware::new(
                            logger.new(o!("struct" => "HelperMiddleware")),
                            mt_sender.clone(),
                            mt_receiver,
                            helper_task_receiver,
                            executor,
                            local_executor,
                        ))),
                        audio_hook: Box::new(HighOnAudioBuffer {
                            task_receiver: at_receiver,
                            reaper: RealTimeReaper {
                                main_thread_task_sender: mt_sender,
                            },
                        }),
                    }))),
                };
                INSTANCE = Some(reaper);
                register_plugin_destroy_hook(|| INSTANCE = None);
            });
        }
    }

    fn require_main_thread(&self) {
        require_main_thread(self.medium.reaper().low().plugin_context());
    }
}

pub struct RealTimeReaper {
    #[allow(unused)]
    main_thread_task_sender: Sender<MainThreadTask>,
}

struct HighOnAudioBuffer {
    task_receiver: Receiver<AudioThreadTaskOp>,
    reaper: RealTimeReaper,
}

impl HighOnAudioBuffer {
    pub fn reset(&self) {
        self.discard_tasks();
    }

    fn discard_tasks(&self) {
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
    action_value_change_history: RefCell<HashMap<CommandId, ActionValueChange>>,
    undo_block_is_active: Cell<bool>,
    main_thread_task_sender: Sender<MainThreadTask>,
    audio_thread_task_sender: Sender<AudioThreadTaskOp>,
    helper_task_sender: Sender<HelperTask>,
    main_thread_future_spawner: crate::run_loop_executor::Spawner,
    local_main_thread_future_spawner: crate::local_run_loop_executor::Spawner,
    session_status: RefCell<SessionStatus>,
}

#[derive(Debug)]
enum SessionStatus {
    Sleeping(Option<SleepingState>),
    Awake(AwakeState),
}

struct SleepingState {
    csurf_inst: Box<MiddlewareControlSurface<HelperMiddleware>>,
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
    csurf_inst_handle: RegistrationHandle<MiddlewareControlSurface<HelperMiddleware>>,
    audio_hook_register_handle: RegistrationHandle<HighOnAudioBuffer>,
    gaccel_registers: HashMap<CommandId, NonNull<raw::gaccel_register_t>>,
}

pub enum ActionKind {
    NotToggleable,
    Toggleable(Box<dyn Fn() -> bool>),
}

pub fn toggleable(is_on: impl Fn() -> bool + 'static) -> ActionKind {
    Toggleable(Box::new(is_on))
}

pub struct ReaperGuard {
    go_to_sleep: Option<Box<dyn FnOnce() + Sync + Send>>,
}

impl Drop for ReaperGuard {
    fn drop(&mut self) {
        debug!(
            Reaper::get().logger(),
            "REAPER guard dropped. Making _reaper-rs_ sleep..."
        );
        (self.go_to_sleep.take().unwrap())();
        let _ = Reaper::get().go_to_sleep();
    }
}

static GUARD_INITIALIZER: std::sync::Once = std::sync::Once::new();

impl Reaper {
    /// The given initializer is executed only the first time this is called.
    ///
    /// `wake_up()` is called whenever first first instance pops up. `go_to_sleep()` is called
    /// whenever the last instance goes away.
    pub fn guarded<S: FnOnce() + Sync + Send + 'static>(
        initializer: impl FnOnce(),
        wake_up: impl FnOnce() -> S,
    ) -> Arc<ReaperGuard> {
        // This is supposed to be called in the main thread. A check is not necessary, because this
        // is protected by a mutex and it will fail in the initializer and getter if called from
        // wrong thread.
        let mut guard = REAPER_GUARD.lock().unwrap();
        if let Some(arc) = guard.upgrade() {
            // There's at least one active instance. No need to reactivate.
            return arc;
        }
        // There's no active instance.
        GUARD_INITIALIZER.call_once(|| {
            // If this is called, there was never an active instance in this session. Initialize!
            initializer();
        });
        let _ = Reaper::get().wake_up();
        let go_to_sleep = wake_up();
        let arc = Arc::new(ReaperGuard {
            go_to_sleep: Some(Box::new(go_to_sleep)),
        });
        *guard = Arc::downgrade(&arc);
        arc
    }

    /// Returns the builder for further configuration.
    pub fn load(context: PluginContext) -> ReaperBuilder {
        require_main_thread(&context);
        ReaperBuilder::new(context)
    }

    /// This has an effect only if there isn't an instance already.
    pub fn setup_with_defaults(context: PluginContext, logger: Logger, crash_info: CrashInfo) {
        require_main_thread(&context);
        Reaper::load(context).logger(logger.clone()).setup();
        std::panic::set_hook(create_reaper_panic_hook(
            logger,
            Some(create_default_console_msg_formatter(crash_info)),
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
        debug!(self.logger(), "Waking up...");
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
        sleeping_state.csurf_inst.middleware().reset();
        sleeping_state.audio_hook.reset();
        // Functions
        let mut medium = self.medium_session();
        medium
            .plugin_register_add_hook_command::<HighLevelHookCommand>()
            .map_err(|_| "couldn't register hook command")?;
        medium
            .plugin_register_add_toggle_action::<HighLevelToggleAction>()
            .map_err(|_| "couldn't register toggle command")?;
        // This only works since Reaper 6.19+dev1226, so we must allow it to fail.
        let _ = medium.plugin_register_add_hook_post_command_2::<HighLevelHookPostCommand2>();
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
        debug!(self.logger(), "Woke up");
        Ok(())
    }

    pub fn go_to_sleep(&self) -> Result<(), &'static str> {
        debug!(self.logger(), "Going to sleep...");
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
        medium.plugin_register_remove_hook_post_command_2::<HighLevelHookPostCommand2>();
        medium.plugin_register_remove_toggle_action::<HighLevelToggleAction>();
        medium.plugin_register_remove_hook_command::<HighLevelHookCommand>();
        *session_status = SessionStatus::Sleeping(Some(SleepingState {
            csurf_inst,
            audio_hook,
        }));
        debug!(self.logger(), "Sleeping");
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

    pub(crate) fn find_last_action_value_change(
        &self,
        command_id: CommandId,
    ) -> Option<ActionValueChange> {
        self.action_value_change_history
            .borrow()
            .get(&command_id)
            .copied()
    }

    /// Only has an effect when compiled with the necessary feature.
    pub fn log_helper_metrics(&self) {
        let _ = self.helper_task_sender.send(HelperTask::LogMetrics);
    }

    /// Spawns a future for execution in main thread.
    pub fn spawn_in_main_thread(
        &self,
        future: impl std::future::Future<Output = ()> + 'static + Send,
    ) {
        let spawner = &self.main_thread_future_spawner;
        spawner.spawn(future);
    }

    /// Spawns a future for execution in main thread.
    ///
    /// Panics if not in main thread. The difference to `spawn_in_main_thread()` is that `Send` is
    /// not required. Perfect for capturing `Rc`s.
    pub fn spawn_in_main_thread_from_main_thread(
        &self,
        future: impl std::future::Future<Output = ()> + 'static,
    ) {
        self.require_main_thread();
        let spawner = &self.local_main_thread_future_spawner;
        spawner.spawn(future);
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread(
        &self,
        waiting_time: Duration,
        op: impl FnOnce() + Send + 'static,
    ) -> Result<(), &'static str> {
        unsafe { self.do_later_in_main_thread_internal(waiting_time, op) }
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread_from_main_thread(
        &self,
        waiting_time: Duration,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        self.require_main_thread();
        unsafe { self.do_later_in_main_thread_internal(waiting_time, op) }
    }

    /// Unsafe because doesn't require send (which should be required in the general case).
    unsafe fn do_later_in_main_thread_internal(
        &self,
        waiting_time: Duration,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        let sender = &self.main_thread_task_sender;
        sender
            .send(MainThreadTask::new(
                Box::new(op),
                Some(SystemTime::now() + waiting_time),
            ))
            .map_err(|_| "channel disconnected")
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_in_main_thread_asap(
        &self,
        op: impl FnOnce() + Send + 'static,
    ) -> Result<(), &'static str> {
        unsafe { self.do_in_main_thread_asap_internal(op) }
    }

    /// Panics if not in main thread. The difference to `do_in_main_thread_asap()` is that `Send` is
    /// not required. Perfect for capturing `Rc`s.
    pub fn do_in_main_thread_from_main_thread_asap(
        &self,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        self.require_main_thread();
        unsafe { self.do_in_main_thread_asap_internal(op) }
    }

    /// Unsafe because doesn't require send (which should be required in the general case).
    unsafe fn do_in_main_thread_asap_internal(
        &self,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        if Reaper::get().is_in_main_thread() {
            op();
            Ok(())
        } else {
            self.do_later_in_main_thread_asap_internal(op)
        }
    }

    // TODO-medium Proper errors
    pub async fn main_thread_future<R: 'static + Send>(
        &self,
        op: impl FnOnce() -> R + 'static + Send,
    ) -> Result<R, &'static str> {
        if Reaper::get().is_in_main_thread() {
            Ok(op())
        } else {
            let (tx, rx) = oneshot::channel();
            self.do_later_in_main_thread_asap(move || {
                tx.send(op()).ok().expect("couldn't send");
            })?;
            rx.await
                .map_err(|_| "error when awaiting main thread future")
        }
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread_asap(
        &self,
        op: impl FnOnce() + Send + 'static,
    ) -> Result<(), &'static str> {
        unsafe { self.do_later_in_main_thread_asap_internal(op) }
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread_from_main_thread_asap(
        &self,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        self.require_main_thread();
        unsafe { self.do_later_in_main_thread_asap_internal(op) }
    }

    /// Unsafe because doesn't require send (which should be required in the general case).
    unsafe fn do_later_in_main_thread_asap_internal(
        &self,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        let sender = &self.main_thread_task_sender;
        sender
            .send(MainThreadTask::new(Box::new(op), None))
            .map_err(|_| "channel disconnected")
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_real_time_audio_thread_asap(
        &self,
        op: impl FnOnce(&RealTimeReaper) + Send + 'static,
    ) -> Result<(), &'static str> {
        let sender = &self.audio_thread_task_sender;
        sender
            .send(Box::new(op))
            .map_err(|_| "channel was disconnected")
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
        let mut operation = operation.borrow_mut();
        operation();
        true
    }
}

// Called by REAPER directly (using a delegate function)!
// Processes main section only.
struct HighLevelHookPostCommand2 {}

impl HookPostCommand2 for HighLevelHookPostCommand2 {
    fn call(
        section: SectionContext,
        command_id: CommandId,
        value_change: ActionValueChange,
        _: WindowContext,
        _: ReaProject,
    ) {
        if section != SectionContext::MainSection {
            return;
        }
        let reaper = Reaper::get();
        reaper
            .action_value_change_history
            .borrow_mut()
            .insert(command_id, value_change);
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
