use c_str_macro::c_str;

use reaper_rs_high::{setup_reaper_with_defaults, ActionKind, Reaper};
use reaper_rs_low::ReaperPluginContext;
use vst::plugin::{HostCallback, Info, Plugin};
use vst::plugin_main;

#[derive(Default)]
struct TestVstPlugin {
    host: HostCallback,
}

impl Plugin for TestVstPlugin {
    fn new(host: HostCallback) -> Self {
        Self { host }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "reaper-rs test".to_string(),
            unique_id: 8372,
            ..Default::default()
        }
    }

    fn init(&mut self) {
        let context = ReaperPluginContext::from_vst_plugin(self.host).unwrap();
        setup_reaper_with_defaults(&context, "info@helgoboss.org");
        let reaper = Reaper::get();
        reaper.activate();
        reaper.show_console_msg(c_str!("Loaded reaper-rs integration test VST plugin\n"));
        reaper.register_action(
            c_str!("reaperRsVstIntegrationTests"),
            c_str!("reaper-rs VST integration tests"),
            || reaper_rs_test::execute_integration_test(),
            ActionKind::NotToggleable,
        );
    }
}

impl Drop for TestVstPlugin {
    fn drop(&mut self) {
        Reaper::teardown();
    }
}

plugin_main!(TestVstPlugin);
