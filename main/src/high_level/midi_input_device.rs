use std::ffi::CString;

// TODO Maybe use enum to distinguish between "All" devices and specific device
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MidiInputDevice {
    // TODO u32?
    id: i32
}

impl MidiInputDevice {
    pub fn new(id: i32) -> Self {
        MidiInputDevice {
            id
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_name(&self) -> CString {
        unimplemented!()
    }

    // For REAPER < 5.94 this is the same like isConnected(). For REAPER >=5.94 it returns true if the device ever
    // existed, even if it's disconnected now.
    pub fn is_available(&self) -> bool {
        unimplemented!()
    }

    pub fn is_connected(&self) -> bool {
        unimplemented!()
    }
}