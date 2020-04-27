use derive_more::*;

// TODO-medium 0 is used in some functions to represent "not found" and I think therefore not a
// valid command ID
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Display)]
pub struct CommandId(pub u32);

impl CommandId {
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl From<CommandId> for i32 {
    fn from(command_id: CommandId) -> Self {
        command_id.0 as i32
    }
}
