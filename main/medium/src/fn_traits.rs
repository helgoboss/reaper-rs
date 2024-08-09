use crate::{
    ActionValueChange, CommandId, Hmenu, Hwnd, HwndInfoType, KbdSectionInfo, MenuHookFlag,
    ReaProject, ReaperStr, SectionContext, WindowContext,
};
use reaper_low::raw::{HWND, INT_PTR};
use reaper_low::{firewall, raw};
use std::ffi::c_char;
use std::os::raw::c_int;
use std::ptr::{null, NonNull};

/// Consumers need to implement this trait in order to define what should happen when a certain
/// action is invoked.
pub trait HookCommand {
    /// The actual callback function invoked by REAPER whenever an action was triggered to run.
    ///
    /// Must return `true` to indicate that the given command has been processed.
    ///
    /// `flag` is usually 0 but can sometimes have useful info depending on the message.
    ///
    /// It's okay to call another command within your command, however you *must* check for
    /// recursion if doing so!
    fn call(command_id: CommandId, flag: i32) -> bool;
}

pub(crate) extern "C" fn delegating_hook_command<T: HookCommand>(
    command_id: c_int,
    flag: c_int,
) -> bool {
    firewall(|| T::call(CommandId(command_id as _), flag)).unwrap_or(false)
}

/// Consumers need to implement this trait in order to define what should happen when a certain
/// MIDI CC/mousewheel action is invoked.
pub trait HookCommand2 {
    /// The actual callback function invoked by REAPER whenever an action was triggered to run.
    ///
    /// Must return `true` to indicate that the given command has been processed.
    fn call(
        section: SectionContext,
        command_id: CommandId,
        value_change: ActionValueChange,
        window: WindowContext,
    ) -> bool;
}

pub(crate) extern "C" fn delegating_hook_command_2<T: HookCommand2>(
    sec: *mut raw::KbdSectionInfo,
    command_id: c_int,
    val: c_int,
    valhw: c_int,
    relmode: c_int,
    hwnd: raw::HWND,
) -> bool {
    firewall(|| {
        let kbd_section_info = NonNull::new(sec).map(KbdSectionInfo);
        let section_context = SectionContext::from_medium(kbd_section_info.as_ref());
        let value_change = ActionValueChange::from_raw((val, valhw, relmode));
        let window_context = WindowContext::from_raw(hwnd);
        T::call(
            section_context,
            CommandId(command_id as _),
            value_change,
            window_context,
        )
    })
    .unwrap_or(false)
}
/// Consumers need to implement this trait in order to define what should happen when REAPER wants to know something
/// about a specific window.
pub trait HwndInfo {
    /// The actual callback function invoked by REAPER whenever it needs to know something about a window.
    ///
    /// Return 0 if not a known window, or if `info_type` is unknown.
    fn call(window: Hwnd, info_type: HwndInfoType) -> i32;
}

pub(crate) extern "C" fn delegating_hwnd_info<T: HwndInfo>(
    hwnd: HWND,
    info_type: INT_PTR,
) -> c_int {
    firewall(|| {
        let window = Hwnd::new(hwnd).expect("REAPER hwnd_info hwnd pointer was null");
        let info_type = HwndInfoType::from_raw(info_type);
        T::call(window, info_type)
    })
    .unwrap_or(0)
}

/// Consumers need to implement this trait in order to define what should happen when a custom menu is initialized or
/// populated.
pub trait HookCustomMenu {
    /// The actual callback function invoked by REAPER whenever a custom menu is initialized or populated.
    fn call(menuidstr: &ReaperStr, menu: Hmenu, flag: MenuHookFlag);
}

pub(crate) extern "C" fn delegating_hook_custom_menu<T: HookCustomMenu>(
    menuidstr: *const c_char,
    menu: raw::HMENU,
    flag: c_int,
) {
    firewall(|| {
        let menuidstr = unsafe { ReaperStr::from_ptr(menuidstr) };
        let menu = Hmenu::new(menu).expect("menu ptr should not be null");
        let flag = MenuHookFlag::from_raw(flag);
        T::call(menuidstr, menu, flag);
    });
}

/// Consumers need to implement this trait in order to define what should happen when a custom menu is initialized or
/// populated.
pub trait ToolbarIconMap {
    /// The actual callback function invoked by REAPER whenever a custom menu is initialized or populated.
    fn call(
        toolbar_name: &ReaperStr,
        command_id: CommandId,
        toggle_state: Option<bool>,
    ) -> Option<&'static ReaperStr>;
}

pub(crate) extern "C" fn delegating_toolbar_icon_map<T: ToolbarIconMap>(
    toolbar_name: *const c_char,
    command_id: c_int,
    state: c_int,
) -> *const c_char {
    firewall(|| {
        let toolbar_name = unsafe { ReaperStr::from_ptr(toolbar_name) };
        let command_id = CommandId(command_id as _);
        let toggle_state = match state.signum() {
            -1 => None,
            0 => Some(false),
            _ => Some(true),
        };
        let icon = T::call(toolbar_name, command_id, toggle_state);
        icon.map(|i| i.as_ptr()).unwrap_or(null())
    })
    .unwrap_or(null())
}

/// Consumers need to implement this trait in order to let REAPER know if a toggleable action is
/// currently *on* or *off*.
pub trait ToggleAction {
    /// The actual callback function called by REAPER to check if an action registered by an
    /// extension has an *on* or *off* state.
    fn call(command_id: CommandId) -> ToggleActionResult;
}

pub enum ToggleActionResult {
    /// Action doesn't belong to this extension or doesn't toggle.
    NotRelevant,
    /// Action belongs to this extension and is currently set to *off*
    Off,
    /// Action belongs to this extension and is currently set to *on*
    On,
}

pub(crate) extern "C" fn delegating_toggle_action<T: ToggleAction>(command_id: c_int) -> c_int {
    firewall(|| {
        use ToggleActionResult::*;
        match T::call(CommandId(command_id as _)) {
            NotRelevant => -1,
            Off => 0,
            On => 1,
        }
    })
    .unwrap_or(-1)
}

/// Consumers need to implement this trait in order to get notified after a normal action of the
/// main section has run.
pub trait HookPostCommand {
    // The actual callback called after an action of the main section has been performed.
    fn call(command_id: CommandId, flag: i32);
}

pub(crate) extern "C" fn delegating_hook_post_command<T: HookPostCommand>(
    command_id: c_int,
    flag: c_int,
) {
    firewall(|| {
        T::call(CommandId(command_id as _), flag);
    });
}

/// Consumers need to implement this trait in order to get notified after a MIDI CC/mousewheel
/// action has run.
pub trait HookPostCommand2 {
    // The actual callback called after an action of the main section has been performed.
    fn call(
        section: SectionContext,
        command_id: CommandId,
        value_change: ActionValueChange,
        window: WindowContext,
        project: ReaProject,
    );
}

pub(crate) extern "C" fn delegating_hook_post_command_2<T: HookPostCommand2>(
    section: *mut raw::KbdSectionInfo,
    action_command_id: c_int,
    val: c_int,
    valhw: c_int,
    relmode: c_int,
    hwnd: raw::HWND,
    proj: *mut raw::ReaProject,
) {
    firewall(|| {
        let kbd_section_info = NonNull::new(section).map(KbdSectionInfo);
        let section_context = SectionContext::from_medium(kbd_section_info.as_ref());
        let value_change = ActionValueChange::from_raw((val, valhw, relmode));
        let window_context = WindowContext::from_raw(hwnd);
        T::call(
            section_context,
            CommandId(action_command_id as _),
            value_change,
            window_context,
            ReaProject::new(proj).expect("no project given"),
        );
    });
}
