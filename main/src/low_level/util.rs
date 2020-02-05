use std::panic::{UnwindSafe, catch_unwind};

// TODO Have a look at ReaLearn fault barrier (what exactly will be logged)
// TODO Maybe make exact behavior configurable application-wide
/// Use this in each function called directly by REAPER to establish a fault barrier = not
/// letting REAPER crash if anything goes wrong within the plugin.
/// Right now it's used in control surface callbacks (and in some high-level API command hooks).
/// Right now this doesn't do anything else than calling catch_unwind. But it might do
/// more in future.
pub fn firewall<F: FnOnce() -> R + UnwindSafe, R>(f: F) -> Option<R> {
    catch_unwind(f).ok()
}