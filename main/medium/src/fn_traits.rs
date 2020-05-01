use crate::CommandId;
use reaper_rs_low::firewall;

pub(crate) type HookCommandFn = extern "C" fn(command_id: i32, flag: i32) -> bool;

pub trait MediumHookCommand {
    // Returning true means eating (processing) the command.
    fn call(command_id: CommandId, flag: i32) -> bool;
}

pub(crate) extern "C" fn delegating_hook_command<T: MediumHookCommand>(
    command_id: i32,
    flag: i32,
) -> bool {
    firewall(|| T::call(CommandId(command_id as u32), flag)).unwrap_or(false)
}

pub(crate) type HookPostCommandFn = extern "C" fn(command_id: i32, flag: i32);

pub trait MediumToggleAction {
    fn call(command_id: CommandId) -> ToggleActionResult;
}

// Possible returns:
// -1=action does not belong to this extension, or does not toggle
// 0=action belongs to this extension and is currently set to "off"
// 1=action belongs to this extension and is currently set to "on"
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

pub enum ToggleActionResult {
    NotRelevant,
    Off,
    On,
}

pub trait MediumHookPostCommand {
    fn call(command_id: CommandId, flag: i32);
}

pub(crate) extern "C" fn delegating_hook_post_command<T: MediumHookPostCommand>(
    command_id: i32,
    flag: i32,
) {
    firewall(|| {
        T::call(CommandId(command_id as u32), flag);
    });
}
