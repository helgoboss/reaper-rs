use crate::CommandId;

pub trait MediumHookCommand {
    fn call(command_id: CommandId, flag: i32) -> bool;
}

pub trait MediumToggleAction {
    fn call(command_id: CommandId) -> i32;
}

pub trait MediumHookPostCommand {
    fn call(command_id: CommandId, flag: i32);
}
