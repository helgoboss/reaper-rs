use enumflags2::BitFlags;
use reaper_medium::{AcceleratorBehavior, AcceleratorKeyCode, VirtKey};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AcceleratorKey {
    VirtKey(VirtKey),
    Character(u16),
}

impl AcceleratorKey {
    pub fn from_behavior_and_key_code(
        f_virt: BitFlags<AcceleratorBehavior>,
        code: AcceleratorKeyCode,
    ) -> Self {
        if f_virt.contains(AcceleratorBehavior::VirtKey) {
            Self::VirtKey(VirtKey::new(
                code.get()
                    .try_into()
                    .expect("accelerator virtual key code larger than byte"),
            ))
        } else {
            Self::Character(code.get())
        }
    }

    pub fn to_code(self) -> u16 {
        use AcceleratorKey::*;
        match self {
            VirtKey(key) => key.get() as u16,
            Character(ch) => ch,
        }
    }
}