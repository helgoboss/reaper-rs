use crate::CommandId;

pub trait HookCommand {
    fn call(command_id: CommandId, flag: i32) -> bool;
}

pub trait ToggleAction {
    fn call(command_id: CommandId) -> i32;
}

pub trait HookPostCommand {
    fn call(command_id: CommandId, flag: i32);
}
