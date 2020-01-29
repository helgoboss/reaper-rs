use crate::medium_level::ControlSurface;

pub struct HelperControlSurface {
}

impl ControlSurface for HelperControlSurface {
    fn run(&self) {
        println!("Moin from HelperControlSurface!!!")
    }
}