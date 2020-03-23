use c_str_macro::c_str;
use reaper_rs::high_level::{ActionKind, Reaper};
use reaper_rs_macros::reaper_plugin;
use std::error::Error;

#[reaper_plugin(email_address = "info@helgoboss.org")]
fn main() -> Result<(), Box<dyn Error>> {
    let reaper = Reaper::get();
    reaper.activate();
    reaper.show_console_msg(c_str!("Loaded reaper-rs integration test plugin\n"));
    reaper.register_action(
        c_str!("reaperRsIntegrationTests"),
        c_str!("reaper-rs integration tests"),
        || reaper_rs_test::execute_integration_test(),
        ActionKind::NotToggleable,
    );
    Ok(())
}
