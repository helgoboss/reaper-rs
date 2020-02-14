use num_enum::IntoPrimitive;

#[derive(Debug, Eq, PartialEq, IntoPrimitive)]
#[repr(i32)]
pub enum StuffMidiMessageTarget {
    VirtualMidiKeyboard,
    MidiAsControlInputQueue,
    VirtualMidiKeyboardOnCurrentChannel
}