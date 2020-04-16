use c_str_macro::c_str;

use reaper_rs_high::{setup_all_with_defaults, Reaper};
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
        setup_all_with_defaults(&context, "info@helgoboss.org");
        let reaper = Reaper::get();
        reaper.show_console_msg(c_str!("Loaded reaper-rs integration test VST plugin\n"));
    }
}

plugin_main!(TestVstPlugin);
