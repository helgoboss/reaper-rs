use vst::plugin::{HostCallback, Plugin, Info};
use vst::plugin_main;
use reaper_rs::low_level;
use reaper_rs::medium_level;
use reaper_rs::high_level;
use c_str_macro::c_str;

#[derive(Default)]
struct TestVstPlugin {
    host: HostCallback
}

impl Plugin for TestVstPlugin {
    fn new(host: HostCallback) -> Self {
        Self {
            host
        }
    }

    fn get_info(&self) -> Info {
        Info {
            name: "reaper-rs test".to_string(),
            unique_id: 8372,
            ..Default::default()
        }
    }

    fn init(&mut self) {
        let host_callback = self.host.raw_callback().unwrap();
        let low = low_level::Reaper::with_all_functions_loaded(
            &low_level::create_reaper_vst_plugin_function_provider(host_callback)
        );
        let medium = medium_level::Reaper::new(low);
        medium.show_console_msg(c_str!("Loaded reaper-rs integration test VST plugin\n"));
    }
}

plugin_main!(TestVstPlugin);