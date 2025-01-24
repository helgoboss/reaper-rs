use crate::{CrashHandler, CrashHandlerConfig, KeyBinding, KeyBindingKind, PluginInfo};
use std::cell::{Cell, RefCell, RefMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use std::rc::Rc;
use std::sync::{Arc, OnceLock, Weak};

use crate::undo_block::UndoBlock;
use crate::ActionKind::Toggleable;
use crate::{DefaultConsoleMessageFormatter, Project};
use once_cell::sync::Lazy;
use reaper_low::{raw, register_plugin_destroy_hook, PluginDestroyHook};

use reaper_low::PluginContext;

use crate::helper_control_surface::{HelperControlSurface, HelperTask};
use crate::mutex_util::lock_ignoring_poisoning;
use fragile::Fragile;
use reaper_medium::ProjectContext::Proj;
use reaper_medium::UndoScope::All;
use reaper_medium::{
    ActionValueChange, CommandId, Handle, HookCommand, HookPostCommand2, OwnedGaccelRegister,
    ReaProject, RealTimeAudioThreadScope, ReaperSession, ReaperStr, ReaperString, ReaperStringArg,
    SectionContext, ToggleAction, ToggleActionResult, WindowContext,
};
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tracing::debug;

/// How many tasks to process at a maximum in one main loop iteration.
pub const DEFAULT_MAIN_THREAD_TASK_BULK_SIZE: usize = 100;

/// We  make sure in **each** public function/method that it's called from the correct thread.
/// Similar with other methods. We basically make this struct thread-safe by panicking whenever we
/// are in the wrong thread.
///
/// We could also go the easy way of using one Reaper instance wrapped in a Mutex. Downside: This is
/// more guarantees than we need. Why should audio thread and main thread fight for access to one
/// Reaper instance. That results in performance loss and possible deadlocks.
static INSTANCE: OnceLock<Reaper> = OnceLock::new();

/// This value can be set more than once and we don't necessarily have REAPER API access at our
/// disposal when accessing it, that's why we can't use `call_once` in combination with thread check
/// in order to get safe access. Let's use a Mutex instead.
static REAPER_GUARD: Lazy<Mutex<Weak<ReaperGuard>>> = Lazy::new(|| Mutex::new(Weak::new()));

pub struct ReaperBuilder {
    medium: reaper_medium::ReaperSession,
}

impl ReaperBuilder {
    fn new(context: PluginContext) -> ReaperBuilder {
        ReaperBuilder {
            medium: {
                let low = reaper_low::Reaper::load(context);
                reaper_medium::ReaperSession::new(low)
            },
        }
    }

    /// This has an effect only if there isn't an instance already.
    pub fn setup(self) -> Result<(), Reaper> {
        self.require_main_thread();
        // At the moment this is just for logging to console when audio thread panics so
        // we don't need it to be big.
        let (helper_task_sender, helper_task_receiver) = crossbeam_channel::bounded(10);
        let medium_reaper = self.medium.reaper().clone();
        let medium_real_time_reaper = self.medium.create_real_time_reaper();
        let reaper_main = ReaperMain {
            medium_session: RefCell::new(self.medium),
            command_by_id: RefCell::new(HashMap::new()),
            action_value_change_history: RefCell::new(Default::default()),
            undo_block_is_active: Cell::new(false),
            session_status: RefCell::new(SessionStatus::Sleeping),
        };
        let reaper = Reaper {
            reaper_main: Fragile::new(reaper_main),
            medium_reaper,
            medium_real_time_reaper,
            helper_task_sender,
            log_crashes_to_console: Default::default(),
            report_crashes_to_sentry: Default::default(),
            #[cfg(feature = "sentry")]
            sentry_initialized: Default::default(),
        };
        INSTANCE.set(reaper)?;
        // After init
        register_plugin_destroy_hook(PluginDestroyHook {
            name: "reaper_high::Reaper",
            callback: || {
                let _ = Reaper::get().go_to_sleep();
            },
        });
        // We register a tiny control surface permanently just for the most essential stuff.
        // It will be unregistered automatically using reaper-medium's Drop implementation.
        let helper_control_surface = HelperControlSurface::new(helper_task_receiver);
        Reaper::get()
            .reaper_main
            .get()
            .medium_session
            .borrow_mut()
            .plugin_register_add_csurf_inst(Box::new(helper_control_surface))
            .unwrap();
        Ok(())
    }

    fn require_main_thread(&self) {
        require_main_thread(self.medium.reaper().low().plugin_context());
    }
}

pub struct RealTimeReaper {}

#[derive(Debug)]
pub struct Reaper {
    reaper_main: Fragile<ReaperMain>,
    pub(crate) medium_reaper: reaper_medium::Reaper,
    pub(crate) medium_real_time_reaper: reaper_medium::Reaper<RealTimeAudioThreadScope>,
    helper_task_sender: crossbeam_channel::Sender<HelperTask>,
    /// Whether to log to the REAPER console (user can toggle this at runtime).
    log_crashes_to_console: Arc<AtomicBool>,
    /// Whether to report to Sentry (user can toggle this at runtime).
    report_crashes_to_sentry: Arc<AtomicBool>,
    /// Whether to log to the REAPER console (user can toggle this at runtime).
    #[cfg(feature = "sentry")]
    sentry_initialized: AtomicBool,
}

#[derive(Debug)]
struct ReaperMain {
    medium_session: RefCell<reaper_medium::ReaperSession>,
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
    session_status: RefCell<SessionStatus>,
}

#[derive(Debug)]
enum SessionStatus {
    Sleeping,
    Awake(AwakeState),
}

#[derive(Debug)]
struct AwakeState {
    action_regs: HashMap<CommandId, ActionReg>,
}

#[derive(Debug)]
struct ActionReg {
    handle: Handle<raw::gaccel_register_t>,
    key_binding_kind: KeyBindingKind,
}

impl ActionReg {
    pub fn new(handle: Handle<raw::gaccel_register_t>, key_binding_kind: KeyBindingKind) -> Self {
        Self {
            handle,
            key_binding_kind,
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

pub struct ReaperGuard {
    manage_reaper_state: bool,
    go_to_sleep: Option<Box<dyn FnOnce() + Sync + Send>>,
}

impl Drop for ReaperGuard {
    fn drop(&mut self) {
        debug!("REAPER guard dropped. Going to sleep...");
        (self.go_to_sleep.take().unwrap())();
        if self.manage_reaper_state {
            let _ = Reaper::get().go_to_sleep();
        }
    }
}

static GUARD_INITIALIZER: std::sync::Once = std::sync::Once::new();

impl Reaper {
    /// The given initializer is executed only the first time this is called.
    ///
    /// `wake_up()` is called whenever the first instance pops up. `go_to_sleep()` is called
    /// whenever the last instance goes away.
    ///
    /// If `manage_reaper_state` is `true`, waking up will also wake up reaper-rs (e.g. register actions) and going
    /// to sleep will put reaper-rs to sleep (e.g. unregister actions). If this is `false`, you must take care of that
    /// manually. This flag is provided so you can keep reaper-rs awake even if no VST plug-in instance is around.
    pub fn guarded<S: FnOnce() + Sync + Send + 'static>(
        manage_reaper_state: bool,
        initializer: impl FnOnce(),
        wake_up: impl FnOnce() -> S,
    ) -> Arc<ReaperGuard> {
        // This is supposed to be called in the main thread. A check is not necessary, because this
        // is protected by a mutex and it will fail in the initializer and getter if called from
        // wrong thread.
        let mut guard = lock_ignoring_poisoning(&REAPER_GUARD);
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
            manage_reaper_state,
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
    pub fn setup_with_defaults(
        plugin_context: PluginContext,
        plugin_info: PluginInfo,
    ) -> Result<(), Reaper> {
        require_main_thread(&plugin_context);
        Reaper::load(plugin_context).setup()?;
        let reaper = Reaper::get();
        // Add custom panic hook
        let crash_handler_config = CrashHandlerConfig {
            plugin_info,
            crash_formatter: Box::new(DefaultConsoleMessageFormatter),
            console_logging_enabled: reaper.log_crashes_to_console.clone(),
            sentry_enabled: reaper.report_crashes_to_sentry.clone(),
        };
        let crash_handler = CrashHandler::new(crash_handler_config);
        std::panic::set_hook(Box::new(move |panic_info| {
            crash_handler.handle_crash(panic_info);
        }));
        Ok(())
    }

    pub fn log_crashes_to_console(&self) -> bool {
        self.log_crashes_to_console.load(Ordering::Relaxed)
    }

    pub fn set_log_crashes_to_console(&self, value: bool) {
        self.log_crashes_to_console.store(value, Ordering::Relaxed);
    }

    pub fn report_crashes_to_sentry(&self) -> bool {
        self.report_crashes_to_sentry.load(Ordering::Relaxed)
    }

    pub fn set_report_crashes_to_sentry(&self, value: bool) {
        self.report_crashes_to_sentry
            .store(value, Ordering::Relaxed);
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
        INSTANCE
            .get()
            .expect("Reaper::load().setup() must be called before Reaper::get()")
    }

    /// Returns whether the instance is loaded already.
    pub fn is_loaded() -> bool {
        INSTANCE.get().is_some()
    }

    /// This wakes reaper-rs up.
    ///
    /// In particular, it does the following:
    ///
    /// - Registers command hooks (to actually execute invoked custom actions or menu entries)
    /// - Registers post command hooks (to inform listeners of executed actions)
    /// - Registers toggle actions (to report action on/off states)
    /// - Registers all previously defined actions
    pub fn wake_up(&self) -> ReaperResult<()> {
        let reaper_main = self.reaper_main.get();
        let mut session_status = reaper_main.session_status.borrow_mut();
        if matches!(session_status.deref(), SessionStatus::Awake(_)) {
            return Err("Session is already awake".into());
        }
        debug!("Waking up...");
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
            action_regs: reaper_main
                .command_by_id
                .borrow()
                .iter()
                .map(|(id, command)| {
                    let reg = register_action(
                        &mut medium,
                        *id,
                        command.description.clone(),
                        command.key_binding,
                    );
                    (*id, reg)
                })
                .collect(),
        });
        debug!("Woke up");
        Ok(())
    }

    pub fn go_to_sleep(&self) -> ReaperResult<()> {
        let mut session_status = self.reaper_main.get().session_status.borrow_mut();
        let awake_state = match session_status.deref() {
            SessionStatus::Sleeping => return Err("Session is already sleeping".into()),
            SessionStatus::Awake(s) => s,
        };
        debug!("Going to sleep...");
        let mut medium = self.medium_session();
        // Unregister actions
        for reg in awake_state.action_regs.values() {
            match reg.key_binding_kind {
                KeyBindingKind::Local => {
                    medium.plugin_register_remove_gaccel(reg.handle);
                }
                KeyBindingKind::Global => {
                    medium.plugin_register_remove_gaccel_global(reg.handle);
                }
                KeyBindingKind::GlobalText => {
                    medium.plugin_register_remove_gaccel_global_text(reg.handle);
                }
            }
        }
        // Remove functions
        medium.plugin_register_remove_hook_post_command_2::<HighLevelHookPostCommand2>();
        medium.plugin_register_remove_toggle_action::<HighLevelToggleAction>();
        medium.plugin_register_remove_hook_command::<HighLevelHookCommand>();
        *session_status = SessionStatus::Sleeping;
        debug!("Sleeping");
        Ok(())
    }

    pub fn medium_session(&self) -> RefMut<reaper_medium::ReaperSession> {
        self.reaper_main.get().medium_session.borrow_mut()
    }

    pub(crate) fn show_console_msg_thread_safe<'a>(&self, msg: impl Into<ReaperStringArg<'a>>) {
        // When calling from non-main thread with REAPER's feature "show_console_msg_from_any_thread",
        // the message doesn't pop up (just a MSG indicator in the menu bar). That's bad. We want it to pop up.
        let call_directly = self.is_in_main_thread();
        // || self
        //     .medium_reaper
        //     .features()
        //     .show_console_msg_from_any_thread;
        if call_directly {
            self.show_console_msg(msg);
        } else {
            let _ = self.helper_task_sender.try_send(HelperTask::ShowConsoleMsg(
                msg.into().into_inner().to_reaper_string().into_string(),
            ));
        }
    }

    /// This looks for a command that has been registered via [`Self::register_action`].
    ///
    /// This is a pure reaper-rs feature, it doesn't communicate with REAPER.
    pub fn with_our_command<R>(
        &self,
        command_id: CommandId,
        use_command: impl FnOnce(Option<&Command>) -> R,
    ) -> R {
        let command_by_id = self.reaper_main.get().command_by_id.borrow();
        let command = command_by_id.get(&command_id);
        use_command(command)
    }

    pub fn register_action(
        &self,
        command_name: impl Into<ReaperStringArg<'static>> + Clone,
        description: impl Into<ReaperStringArg<'static>>,
        default_key_binding: Option<KeyBinding>,
        operation: impl FnMut() + 'static,
        kind: ActionKind,
    ) -> RegisteredAction {
        let reaper_main = self.reaper_main.get();
        let mut medium = self.medium_session();
        let command_id = medium
            .plugin_register_add_command_id(command_name.clone())
            .unwrap();
        let description = description.into().into_inner();
        let command = Command::new(
            command_name.into().into_inner().to_reaper_string(),
            Rc::new(RefCell::new(operation)),
            kind,
            description.to_reaper_string(),
            default_key_binding,
        );
        if let Entry::Vacant(p) = reaper_main.command_by_id.borrow_mut().entry(command_id) {
            p.insert(command);
        }
        let registered_action = RegisteredAction::new(command_id);
        // Immediately register if active
        let mut session_status = reaper_main.session_status.borrow_mut();
        let awake_state = match session_status.deref_mut() {
            SessionStatus::Sleeping => return registered_action,
            SessionStatus::Awake(s) => s,
        };
        let action_reg = register_action(
            &mut medium,
            command_id,
            description.into_owned(),
            default_key_binding,
        );
        awake_state.action_regs.insert(command_id, action_reg);
        registered_action
    }

    fn unregister_action(&self, command_id: CommandId) {
        let reaper_main = self.reaper_main.get();
        // Unregistering command when it's destroyed via RAII (implementing Drop)? Bad idea, because
        // this is the wrong point in time. The right point in time for unregistering is when it's
        // removed from the command hash map. Because even if the command still exists in memory,
        // if it's not in the map anymore, REAPER won't be able to find it.
        reaper_main.command_by_id.borrow_mut().remove(&command_id);
        // Unregister if active
        let mut session_status = reaper_main.session_status.borrow_mut();
        let awake_state = match session_status.deref_mut() {
            SessionStatus::Sleeping => return,
            SessionStatus::Awake(s) => s,
        };
        if let Some(reg) = awake_state.action_regs.get(&command_id) {
            match reg.key_binding_kind {
                KeyBindingKind::Local => {
                    self.medium_session()
                        .plugin_register_remove_gaccel(reg.handle);
                }
                KeyBindingKind::Global => {
                    self.medium_session()
                        .plugin_register_remove_gaccel_global(reg.handle);
                }
                KeyBindingKind::GlobalText => {
                    self.medium_session()
                        .plugin_register_remove_gaccel_global_text(reg.handle);
                }
            }
        }
    }

    pub(crate) fn find_last_action_value_change(
        &self,
        command_id: CommandId,
    ) -> Option<ActionValueChange> {
        self.reaper_main
            .get()
            .action_value_change_history
            .borrow()
            .get(&command_id)
            .copied()
    }

    pub fn undoable_action_is_running(&self) -> bool {
        self.reaper_main.get().undo_block_is_active.get()
    }

    // Doesn't start a new block if we already are in an undo block.
    #[must_use = "Return value determines the scope of the undo block (RAII)"]
    pub(super) fn enter_undo_block_internal<'a>(
        &self,
        project: Project,
        label: &'a ReaperStr,
    ) -> Option<UndoBlock<'a>> {
        let reaper_main = self.reaper_main.get();
        if reaper_main.undo_block_is_active.get() {
            return None;
        }
        reaper_main.undo_block_is_active.replace(true);
        self.medium_reaper().undo_begin_block_2(Proj(project.raw()));
        Some(UndoBlock::new(project, label))
    }

    // Doesn't attempt to end a block if we are not in an undo block.
    pub(super) fn leave_undo_block_internal(&self, project: Project, label: &ReaperStr) {
        let reaper_main = self.reaper_main.get();
        if !reaper_main.undo_block_is_active.get() {
            return;
        }
        self.medium_reaper()
            .undo_end_block_2(Proj(project.raw()), label, All);
        reaper_main.undo_block_is_active.replace(false);
    }

    pub fn require_main_thread(&self) {
        require_main_thread(Reaper::get().medium_reaper().low().plugin_context());
    }
}

// TODO-medium Think about the consequences.
unsafe impl Sync for Reaper {}

pub struct Command {
    name: ReaperString,
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
    key_binding: Option<KeyBinding>,
}

impl Debug for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command").finish()
    }
}

impl Command {
    fn new(
        name: ReaperString,
        operation: Rc<RefCell<dyn FnMut()>>,
        kind: ActionKind,
        description: ReaperString,
        key_binding: Option<KeyBinding>,
    ) -> Command {
        Command {
            name,
            operation,
            kind,
            description,
            key_binding,
        }
    }

    pub fn command_name(&self) -> &str {
        self.name.to_str()
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
        let operation = match Reaper::get()
            .reaper_main
            .get()
            .command_by_id
            .borrow()
            .get(&command_id)
        {
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
            .reaper_main
            .get()
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
        if let Some(command) = Reaper::get()
            .reaper_main
            .get()
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

fn require_main_thread(context: &PluginContext) {
    assert!(
        context.is_in_main_thread(),
        "this function must be called in the main thread"
    );
}

fn register_action(
    session: &mut ReaperSession,
    command_id: CommandId,
    description: impl Into<ReaperStringArg<'static>>,
    default_key_binding: Option<KeyBinding>,
) -> ActionReg {
    let (reg, key_binding_kind) = match default_key_binding {
        None => (
            OwnedGaccelRegister::without_key_binding(command_id, description),
            KeyBindingKind::Local,
        ),
        Some(kb) => (
            OwnedGaccelRegister::with_key_binding(
                command_id,
                description,
                kb.behavior,
                kb.key_code,
            ),
            kb.kind,
        ),
    };
    let register_local = |session: &mut ReaperSession, reg| {
        ActionReg::new(
            session.plugin_register_add_gaccel(reg).unwrap(),
            KeyBindingKind::Local,
        )
    };
    match key_binding_kind {
        KeyBindingKind::Local => register_local(session, reg),
        KeyBindingKind::Global => {
            match session.plugin_register_add_gaccel_global(reg) {
                Ok(handle) => ActionReg::new(handle, key_binding_kind),
                Err(reg) => {
                    // REAPER < 7.07
                    register_local(session, reg)
                }
            }
        }
        KeyBindingKind::GlobalText => {
            match session.plugin_register_add_gaccel_global_text(reg) {
                Ok(handle) => ActionReg::new(handle, key_binding_kind),
                Err(reg) => {
                    // REAPER < 7.07
                    register_local(session, reg)
                }
            }
        }
    }
}

use crate::error::ReaperResult;
#[cfg(feature = "sentry")]
pub use sentry_impl::SentryConfig;

#[cfg(feature = "sentry")]
mod sentry_impl {
    use super::*;
    use sentry::types::Dsn;
    use sentry::ClientOptions;
    use std::mem;

    pub struct SentryConfig<'a> {
        pub dsn: Dsn,
        pub in_app_include: Vec<&'static str>,
        pub plugin_info: &'a PluginInfo,
    }

    impl Reaper {
        /// Initializes Sentry with the given configuration.
        ///
        /// Later calls will be ignored.
        pub fn init_sentry(&self, config: SentryConfig) {
            if self.sentry_initialized.load(Ordering::Relaxed) {
                return;
            }
            let client_options = ClientOptions {
                dsn: Some(config.dsn),
                release: Some(
                    format!(
                        "{}@{}",
                        &config.plugin_info.plugin_name, &config.plugin_info.plugin_version
                    )
                    .into(),
                ),
                // We don't want default integrations because we have our own panic handler that uses the
                // Sentry panic handler.
                default_integrations: false,
                integrations: vec![
                    Arc::new(sentry::integrations::backtrace::AttachStacktraceIntegration),
                    Arc::new(sentry::integrations::debug_images::DebugImagesIntegration::default()),
                    Arc::new(sentry::integrations::contexts::ContextIntegration::default()),
                    // Skip the panic integration
                    Arc::new(sentry::integrations::backtrace::ProcessStacktraceIntegration),
                ],
                attach_stacktrace: false,
                send_default_pii: false,
                in_app_include: config.in_app_include,
                // shutdown_timeout: Default::default(),
                ..Default::default()
            };
            let sentry_guard = sentry::init(client_options);
            // Keeping the sentry guard around will cause Sentry destructor code to run when
            // Reaper is dropped. This destructor code does thread-local and/or std::thread::current
            // calls, which would panic when executed after the DLL is detached (in plug-in
            // destroy hooks). That in turn would cause a crash.
            // Therefore, we just forget about the sentry guard. We don't mind if some queued
            // messages don't get sent anymore on exit.
            mem::forget(sentry_guard);
            self.sentry_initialized.store(true, Ordering::Relaxed);
        }
    }
}
