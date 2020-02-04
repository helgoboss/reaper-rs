use std::panic::{UnwindSafe, catch_unwind};

// TODO Have a look at ReaLearn fault barrier (what exactly will be logged)
// TODO Maybe make exact behavior configurable application-wide
/// Use this in each function called directly by REAPER to establish a fault barrier = not
/// letting REAPER crash if anything goes wrong within the plugin.
pub fn firewall<F: FnOnce() -> R + UnwindSafe, R>(default_result: R, f: F) -> R {
    match catch_unwind(f) {
        Ok(result) => result,
        Err(cause) => {
            let error_msg = match cause.downcast::<&str>() {
                Ok(cause) => cause.to_string(),
                Err(cause) => match cause.downcast::<String>() {
                    Ok(cause) => *cause,
                    Err(cause) => String::from("Unknown error")
                }
            };
            default_result
        }
    }
}