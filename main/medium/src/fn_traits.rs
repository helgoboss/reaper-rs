use crate::CommandId;

pub(crate) type HookCommandFn = extern "C" fn(command_id: i32, flag: i32) -> bool;

pub trait MediumHookCommand {
    fn call(command_id: CommandId, flag: i32) -> bool;
}

pub(crate) type HookPostCommandFn = extern "C" fn(command_id: i32, flag: i32);

pub trait MediumToggleAction {
    fn call(command_id: CommandId) -> i32;
}

pub(crate) type ToggleActionFn = extern "C" fn(command_id: i32) -> i32;

pub trait MediumHookPostCommand {
    fn call(command_id: CommandId, flag: i32);
}
