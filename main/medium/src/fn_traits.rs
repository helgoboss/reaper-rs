use crate::{
    ActionValueChange, CommandId, KbdSectionInfo, ReaProject, SectionContext, WindowContext,
};
use reaper_low::{firewall, raw};
use std::os::raw::c_int;
use std::ptr::NonNull;

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
            NonNull::new(proj).expect("no project given"),
        );
    });
}
