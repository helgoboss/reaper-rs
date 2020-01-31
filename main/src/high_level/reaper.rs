use std::borrow::{Borrow, BorrowMut, Cow};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_ushort, c_void};
use std::ptr::{null, null_mut};
use std::sync::Once;

use c_str_macro::c_str;

use crate::high_level::ActionKind::Toggleable;
use crate::high_level::{Project, Section};
use crate::low_level::{ACCEL, gaccel_register_t, MediaTrack, ReaProject};
use crate::low_level;
use crate::medium_level;
use rxrust::observable::Observable;
use rxrust::subscriber::Subscriber;
use crate::high_level::helper_control_surface::HelperControlSurface;
use rxrust::subscription::SubscriptionLike;
use rxrust::observer::Observer;
use rxrust::prelude::*;
use rxrust::subject::{LocalSubjectObserver, SubjectValue};
use std::rc::Rc;

// See https://doc.rust-lang.org/std/sync/struct.Once.html why this is safe in combination with Once
static mut REAPER_INSTANCE: Option<Reaper> = None;
static INIT_REAPER_INSTANCE: Once = Once::new();

// Only for main section
fn hook_command(command_index: i32, flag: i32) -> bool {
    let mut operation = match Reaper::instance().command_by_index.borrow().get(&command_index) {
        Some(command) => command.operation.clone(),
        None => return false
    };
    (*operation).borrow_mut().call_mut(());
    true
}

// Only for main section
fn toggle_action(command_index: i32) -> i32 {
    if let Some(command) = Reaper::instance().command_by_index.borrow().get(&command_index) {
        match &command.kind {
            ActionKind::Toggleable(is_on) => if is_on() { 1 } else { 0 },
            ActionKind::NotToggleable => -1
        }
    } else {
        -1
    }
}


pub struct Reaper {
    pub medium: medium_level::Reaper,
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
    command_by_index: RefCell<HashMap<i32, Command>>,
    // This is a RefCell. So calling next() while another next() is still running will panic.
    // I guess it's good that way because this is very generic code, panicking or not panicking
    // depending on the user's code. And getting a panic is good for becoming aware of the problem
    // instead of running into undefined behavior. The developer can always choose to defer to
    // the next `ControlSurface::run()` invocation (execute things in next main loop cycle).
    pub(super) project_switched_subject: EventStreamSubject<Project>,
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

impl Reaper {
    pub fn setup(medium: medium_level::Reaper) {
        let reaper = Reaper {
            medium,
            command_by_index: RefCell::new(HashMap::new()),
            project_switched_subject: RefCell::new(Subject::local()),
        };
        unsafe {
            INIT_REAPER_INSTANCE.call_once(|| {
                REAPER_INSTANCE = Some(reaper);
            });
        }
        Reaper::instance().init();
    }

    fn init(&self) {
        self.medium.plugin_register(c_str!("hookcommand"), hook_command as *mut c_void);
        self.medium.plugin_register(c_str!("toggleaction"), toggle_action as *mut c_void);
        self.medium.install_control_surface(HelperControlSurface::new());
        self.medium.register_control_surface();
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
        let command_index = self.medium.plugin_register(c_str!("command_id"), command_id.as_ptr() as *mut c_void);
        let command = Command::new(command_index, description.into(), Rc::new(RefCell::new(operation)), kind);
        self.register_command(command_index, command);
        RegisteredAction::new(command_index)
    }

    fn register_command(&self, command_index: i32, command: Command) {
        if let Entry::Vacant(p) = self.command_by_index.borrow_mut().entry(command_index) {
            let command = p.insert(command);
            let acc = &mut command.accelerator_register;
            self.medium.plugin_register(c_str!("gaccel"), acc as *mut _ as *mut c_void);
        }
    }

    fn unregister_command(&self, command_index: i32) {
        // TODO Use RAII
        if let Some(command) = self.command_by_index.borrow_mut().get_mut(&command_index) {
            let acc = &mut command.accelerator_register;
            self.medium.plugin_register(c_str!("-gaccel"), acc as *mut _ as *mut c_void);
            self.command_by_index.borrow_mut().remove(&command_index);
        }
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

    pub fn get_main_section(&self) -> Section {
        Section::new(self.medium.section_from_unique_id(0))
    }

    pub fn create_empty_project_in_new_tab(&self) -> Project {
        self.get_main_section().action_by_command_id(41929).invoke_as_trigger(None);
        self.get_current_project()
    }

    pub fn project_switched(&self) -> EventStream<Project> {
        self.project_switched_subject.borrow().fork()
    }

    pub fn get_current_project(&self) -> Project {
        let (rp, _) = self.medium.enum_projects(-1, 0);
        Project::new(rp)
    }

    pub fn get_projects(&self) -> impl Iterator<Item=Project> + '_ {
        (0..)
            .map(move |i| self.medium.enum_projects(i, 0).0)
            .take_while(|p| !p.is_null())
            .map(|p| { Project::new(p) })
    }

    pub fn get_project_count(&self) -> i32 {
        self.get_projects().count() as i32
    }

    pub fn clear_console(&self) {
        self.medium.clear_console();
    }
}

struct Command {
    description: Cow<'static, CStr>,
    /// Reasoning for that type (from inner to outer):
    /// - `FnMut`: We don't use just `fn` because we want to support closures. We don't use just
    ///   `Fn` because we want to support closures that keep mutable references to their captures.
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
    fn new(command_index: i32, description: Cow<'static, CStr>, operation: Rc<RefCell<dyn FnMut()>>, kind: ActionKind) -> Command {
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
    command_index: i32,
}

impl RegisteredAction {
    fn new(command_index: i32) -> RegisteredAction {
        RegisteredAction {
            command_index,
        }
    }

    pub fn unregister(&self) {
        Reaper::instance().unregister_command(self.command_index);
    }
}