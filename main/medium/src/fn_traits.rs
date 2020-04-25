pub trait HookCommand {
    fn call(command_id: u32, flag: i32) -> bool;
}

pub trait ToggleAction {
    fn call(command_id: u32) -> i32;
}

pub trait HookPostCommand {
    fn call(command_id: u32, flag: i32);
}
