use enumflags2::BitFlags;
use reaper_medium::{AcceleratorBehavior, AcceleratorKeyCode, VirtKey};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AcceleratorKey {
    VirtKey(VirtKey),
    // TODO-high Can this be more than u8?
    Character(u8),
}

impl AcceleratorKey {
    pub fn from_behavior_and_key_code(
        f_virt: BitFlags<AcceleratorBehavior>,
        code: AcceleratorKeyCode,
    ) -> Self {
        if f_virt.contains(AcceleratorBehavior::VirtKey) {
            Self::VirtKey(VirtKey::new(code.get() as i32))
        } else {
            Self::Character(code.get() as u8)
        }
    }

    pub fn to_code(self) -> u16 {
        use AcceleratorKey::*;
        match self {
            // TODO-high Is VirtKey i32 too broad?
            VirtKey(key) => key.get() as u16,
            Character(ch) => ch as u16,
        }
    }
}
