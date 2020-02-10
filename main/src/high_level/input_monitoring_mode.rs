use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum InputMonitoringMode {
    Off = 0,
    Normal = 1,
    NotWhenPlaying = 2,
}