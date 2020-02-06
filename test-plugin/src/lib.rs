use reaper_rs_macros::reaper_plugin;
use reaper_rs::high_level::{Reaper, ActionKind};
use c_str_macro::c_str;

#[reaper_plugin(email_address = "info@helgoboss.org")]
fn main() -> Result<(), &'static str> {
    let reaper = Reaper::instance();
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
