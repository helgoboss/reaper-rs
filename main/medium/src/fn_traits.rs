use crate::CommandId;
use reaper_low::firewall;
use std::os::raw::c_int;

/// Consumers need to implement this trait in order to define what should happen when a certain
/// action is invoked.
pub trait MediumHookCommand {
    /// The actual callback function called by REAPER whenever an action was triggered to run.
    ///
    /// Must return `true` to indicate that the given command has been processed.
    ///
    /// `flag` is usually 0 but can sometimes have useful info depending on the message.
    ///
    /// It's okay to call another command within your command, however you *must* check for
    /// recursion if doing so!
    fn call(command_id: CommandId, flag: i32) -> bool;
}

pub(crate) extern "C" fn delegating_hook_command<T: MediumHookCommand>(
    command_id: c_int,
    flag: c_int,
) -> bool {
    firewall(|| T::call(CommandId(command_id as _), flag)).unwrap_or(false)
}

/// Consumers need to implement this trait in order to let REAPER know if a toggleable action is
/// currently *on* or *off*.
pub trait MediumToggleAction {
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

pub(crate) extern "C" fn delegating_toggle_action<T: MediumToggleAction>(
    command_id: c_int,
) -> c_int {
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

/// Consumers need to implement this trait in order to get notified after an action of the main
/// section has run.
pub trait MediumHookPostCommand {
    // The actual callback called after an action of the main section has been performed.
    fn call(command_id: CommandId, flag: i32);
}

pub(crate) extern "C" fn delegating_hook_post_command<T: MediumHookPostCommand>(
    command_id: c_int,
    flag: c_int,
) {
    firewall(|| {
        T::call(CommandId(command_id as _), flag);
    });
}
