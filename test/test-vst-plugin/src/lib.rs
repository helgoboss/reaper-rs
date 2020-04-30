use c_str_macro::c_str;

use reaper_rs_high::{ActionKind, Reaper, ReaperGuard};
use reaper_rs_low::ReaperPluginContext;
use reaper_rs_medium::{
    AudioHookRegister, CommandId, MediumAudioHookRegister, MediumHookPostCommand,
    MediumOnAudioBuffer, MediumReaperControlSurface,
};
use std::sync::Arc;
use vst::plugin::{HostCallback, Info, Plugin};
use vst::plugin_main;

plugin_main!(TestVstPlugin);

#[derive(Default)]
struct TestVstPlugin {
    host: HostCallback,
    reaper: Option<reaper_rs_medium::Reaper>,
    reaper_guard: Option<Arc<ReaperGuard>>,
}

impl Plugin for TestVstPlugin {
    fn new(host: HostCallback) -> Self {
        Self {
            host,
            reaper: None,
            reaper_guard: None,
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
        self.use_high_level_reaper();
    }
}

struct MyOnAudioBuffer {
    pub counter: u64,
}

impl MediumOnAudioBuffer for MyOnAudioBuffer {
    type UserData1 = MyOnAudioBuffer;
    type UserData2 = ();

    fn call(is_post: bool, len: i32, srate: f64, reg: AudioHookRegister<MyOnAudioBuffer, ()>) {
        let state = reg.user_data_1().unwrap();
        state.counter += 1;
        // println!("Audio hook counter: {}", state.counter)
    }
}

struct MyHookPostCommand;

impl MediumHookPostCommand for MyHookPostCommand {
    fn call(command_id: CommandId, flag: i32) {
        println!("Command {:?} executed", command_id)
    }
}

struct MyControlSurface;

impl MediumReaperControlSurface for MyControlSurface {
    fn set_track_list_change(&self) {
        println!("Track list changed!")
    }
}

impl TestVstPlugin {
    fn use_medium_level_reaper(&mut self) {
        let context = ReaperPluginContext::from_vst_plugin(self.host).unwrap();
        let low = reaper_rs_low::Reaper::load(&context);
        let mut medium = reaper_rs_medium::Reaper::new(low);
        medium.show_console_msg("Registering control surface ...");
        medium.plugin_register_add_csurf_inst(MyControlSurface);
        medium.show_console_msg("Registering action ...");
        medium.plugin_register_add_hookpostcommand::<MyHookPostCommand>();
        medium.audio_reg_hardware_hook_add(MediumAudioHookRegister::new::<MyOnAudioBuffer, _, _>(
            Some(MyOnAudioBuffer { counter: 0 }),
            None,
        ));
        self.reaper = Some(medium);
    }

    fn use_high_level_reaper(&mut self) {
        let guard = Reaper::guarded(|| {
            let context = ReaperPluginContext::from_vst_plugin(self.host).unwrap();
            // TODO-medium This is bad. There must be only one static Reaper instance per module,
            // not  per VST plug-in instance! Even considering the fact that high-level
            // Reaper is static,  we should provide some Rc/RAII mechanism to easily
            // manage the singleton instance.
            Reaper::setup_with_defaults(&context, "info@helgoboss.org");
            let reaper = Reaper::get();
            reaper.activate();
            reaper.show_console_msg(c_str!("Loaded reaper-rs integration test VST plugin\n"));
            reaper.register_action(
                c_str!("reaperRsVstIntegrationTests"),
                c_str!("reaper-rs VST integration tests"),
                || reaper_rs_test::execute_integration_test(),
                ActionKind::NotToggleable,
            );
        });
        self.reaper_guard = Some(guard);
    }
}
