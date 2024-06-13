use reaper_high::{ActionKind, Reaper};

use reaper_macros::reaper_extension_plugin;
use std::error::Error;
use std::process;
use tracing::debug;

#[reaper_extension_plugin(
    name = "reaper-rs test extension plug-in",
    support_email_address = "info@helgoboss.org"
)]
fn main() -> Result<(), Box<dyn Error>> {
    let run_integration_test = std::env::var("RUN_REAPER_RS_INTEGRATION_TEST").is_ok();
    if run_integration_test {
        println!("From REAPER: Launching reaper-rs reaper-test-extension-plugin...");
    }
    let reaper = Reaper::get();
    reaper.wake_up()?;
    debug!("Loaded reaper-rs integration test plugin");
    if run_integration_test {
        println!("From REAPER: Entering reaper-rs integration test...");
        reaper_test::execute_integration_test(|result| {
            match result {
                Ok(_) => {
                    println!("From REAPER: reaper-rs integration test executed successfully");
                    process::exit(0)
                }
                Err(reason) => {
                    // We use a particular exit code to distinguish test failure from other possible
                    // exit paths.
                    eprintln!("From REAPER: reaper-rs integration test failed: {reason}");
                    process::exit(172)
                }
            }
        });
    }
    reaper.register_action(
        "reaperRsIntegrationTests",
        "reaper-rs integration tests",
        None,
        || reaper_test::execute_integration_test(|_| ()),
        ActionKind::NotToggleable,
    );
    Ok(())
}
