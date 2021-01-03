use crate::{ActionRx, ControlSurfaceRx};

#[derive(Default)]
pub struct MainRx {
    action: ActionRx,
    control_surface: ControlSurfaceRx,
}

impl MainRx {
    pub fn action(&self) -> &ActionRx {
        &self.action
    }

    pub fn control_surface(&self) -> &ControlSurfaceRx {
        &self.control_surface
    }
}
