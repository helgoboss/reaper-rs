use reaper_high::{ActionKind, Reaper};

use reaper_macros::reaper_extension_plugin;
use reaper_test::IntegrationTest;
use std::error::Error;
use std::process;
use std::time::Duration;
use futures_timer::Delay;

#[reaper_extension_plugin(
    name = "reaper-rs test extension plug-in",
    support_email_address = "info@helgoboss.org",
    update_url = "https://www.helgoboss.org/projects/helgobox"
)]
fn main() -> Result<(), Box<dyn Error>> {
    println!("From REAPER: Loaded reaper-rs integration test plugin");
    let run_integration_test = std::env::var("RUN_REAPER_RS_INTEGRATION_TEST").is_ok();
    if run_integration_test {
        println!("From REAPER: Launching reaper-rs reaper-test-extension-plugin...");
    }
    let reaper = Reaper::get();
    reaper.wake_up()?;
    let integration_test = IntegrationTest::setup();
    if run_integration_test {
        let future_support_clone = integration_test.future_support().clone();
        future_support_clone.spawn_in_main_thread_from_main_thread(async {
            // On Linux, we shouldn't start executing tests right after starting REAPER. Otherwise,
            // some events will not be raised.
            println!("From REAPER: Waiting a bit before starting the test...");
            millis(2000).await;
            let exit_code = match reaper_test::execute_integration_test().await {
                Ok(_) => {
                    println!("From REAPER: reaper-rs integration test executed successfully");
                    0
                }
                Err(reason) => {
                    // We use a particular exit code to distinguish test failure from other possible
                    // exit paths.
                    eprintln!("From REAPER: reaper-rs integration test failed: {reason}");
                    172
                }
            };
            // Waiting somehow lowers the risk of exiting with SIGSEGV (signal 11) on Linux.
            // Not sure where this SIGSEGV comes from.
            println!("From REAPER: Waiting a bit before exiting the REAPER process...");
            millis(5000).await;
            process::exit(exit_code);
        });
    }
    let future_support_clone = integration_test.future_support().clone();
    reaper.register_action(
        "reaperRsIntegrationTests",
        "reaper-rs integration tests",
        None,
        move || {
            future_support_clone.spawn_in_main_thread_from_main_thread(async {
                reaper_test::execute_integration_test().await?;
                Ok(())
            });
        },
        ActionKind::NotToggleable,
    );
    Ok(())
}

fn millis(millis: u64) -> Delay {
    futures_timer::Delay::new(Duration::from_millis(millis))
}
