use crate::CommandId;
use reaper_low::firewall;

pub(crate) type HookCommandFn = extern "C" fn(command_id: i32, flag: i32) -> bool;

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

/// Consumers need to implement this trait in order to let REAPER know if a toggleable action is
/// currently *on* or *off*.
pub trait MediumToggleAction {
    /// The actual callback function called by REAPER to check if an action registered by an
    /// extension has an *on* or *off* state.
    fn call(command_id: CommandId) -> ToggleActionResult;
}

/// Consumers need to implement this trait in order to get notified after an action of the main
/// section has run.
pub trait MediumHookPostCommand {
    // The actual callback called after an action of the main section has been performed.
    fn call(command_id: CommandId, flag: i32);
}

pub enum ToggleActionResult {
    /// Action doesn't belong to this extension or doesn't toggle.
    NotRelevant,
    /// Action belongs to this extension and is currently set to *off*
    Off,
    /// Action belongs to this extension and is currently set to *on*
    On,
}

pub(crate) extern "C" fn delegating_hook_command<T: MediumHookCommand>(
    command_id: i32,
    flag: i32,
) -> bool {
    firewall(|| T::call(CommandId(command_id as u32), flag)).unwrap_or(false)
}

pub(crate) type HookPostCommandFn = extern "C" fn(command_id: i32, flag: i32);

pub(crate) type ToggleActionFn = extern "C" fn(command_id: i32) -> i32;

pub(crate) extern "C" fn delegating_toggle_action<T: MediumToggleAction>(command_id: i32) -> i32 {
    firewall(|| {
        use ToggleActionResult::*;
        match T::call(CommandId(command_id as u32)) {
            NotRelevant => -1,
            Off => 0,
            On => 1,
        }
    })
    .unwrap_or(-1)
}

pub(crate) extern "C" fn delegating_hook_post_command<T: MediumHookPostCommand>(
    command_id: i32,
    flag: i32,
) {
    firewall(|| {
        T::call(CommandId(command_id as u32), flag);
    });
}
