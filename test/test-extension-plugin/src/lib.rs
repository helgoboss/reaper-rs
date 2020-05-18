use c_str_macro::c_str;
use reaper_high::{ActionKind, ReaperSession};

use reaper_macros::reaper_extension_plugin;
use std::error::Error;
use std::process;

#[reaper_extension_plugin(email_address = "info@helgoboss.org")]
fn main() -> Result<(), Box<dyn Error>> {
    println!("From REAPER: Launching reaper-rs reaper-test-extension-plugin...");
    let reaper = ReaperSession::get();
    reaper.activate();
    reaper.show_console_msg(c_str!("Loaded reaper-rs integration test plugin\n"));
    if std::env::var("RUN_REAPER_RS_INTEGRATION_TEST").is_ok() {
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
                    eprintln!("From REAPER: reaper-rs integration test failed: {}", reason);
                    process::exit(172)
                }
            }
        });
    }
    reaper.register_action(
        c_str!("reaperRsIntegrationTests"),
        c_str!("reaper-rs integration tests"),
        || reaper_test::execute_integration_test(|_| ()),
        ActionKind::NotToggleable,
    );
    Ok(())
}
