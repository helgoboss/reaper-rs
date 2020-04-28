use crate::CommandId;
use reaper_rs_low::firewall;

pub(crate) type HookCommandFn = extern "C" fn(command_id: i32, flag: i32) -> bool;

pub trait MediumHookCommand {
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
    fn call(command_id: CommandId) -> i32;
}

pub(crate) type ToggleActionFn = extern "C" fn(command_id: i32) -> i32;

pub(crate) extern "C" fn delegating_toggle_action<T: MediumToggleAction>(command_id: i32) -> i32 {
    firewall(|| T::call(CommandId(command_id as u32))).unwrap_or(-1)
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
