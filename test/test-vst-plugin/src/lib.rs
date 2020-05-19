use c_str_macro::c_str;

use reaper_high::{ActionKind, Reaper, ReaperGuard};
use reaper_low::{reaper_vst_plugin, ReaperPluginContext};
use reaper_medium::{
    CommandId, MediumHookPostCommand, MediumOnAudioBuffer, MediumReaperControlSurface,
    OnAudioBufferArgs,
};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use vst::plugin::{HostCallback, Info, Plugin};
use vst::plugin_main;

plugin_main!(TestVstPlugin);
reaper_vst_plugin!();

#[derive(Default)]
struct TestVstPlugin {
    host: HostCallback,
    session: Option<reaper_medium::ReaperSession>,
    reaper_guard: Option<Arc<ReaperGuard>>,
}

impl Plugin for TestVstPlugin {
    fn new(host: HostCallback) -> Self {
        Self {
            host,
            session: None,
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
        // self.use_medium_level_reaper();
        self.use_high_level_reaper();
    }
}

struct MyOnAudioBuffer {
    sender: std::sync::mpsc::Sender<String>,
    counter: u64,
}

impl MediumOnAudioBuffer for MyOnAudioBuffer {
    fn call(&mut self, args: OnAudioBufferArgs) {
        if self.counter % 100 == 0 {
            self.sender
                .send(format!(
                    "Counter: {}, Args: {:?}, Channels: {:?}\n",
                    self.counter,
                    args,
                    (args.reg.input_nch(), args.reg.output_nch())
                ))
                .expect("couldn't send console logging message to main thread");
        }
        self.counter += 1;
    }
}

struct MyHookPostCommand;

impl MediumHookPostCommand for MyHookPostCommand {
    fn call(command_id: CommandId, _flag: i32) {
        println!("Command {:?} executed", command_id)
    }
}

#[derive(Debug)]
struct MyControlSurface {
    reaper: reaper_medium::Reaper,
    receiver: Receiver<String>,
}

impl MediumReaperControlSurface for MyControlSurface {
    fn run(&mut self) {
        for msg in self.receiver.try_iter() {
            self.reaper.show_console_msg(msg);
        }
    }

    fn set_track_list_change(&self) {
        println!("Track list changed!")
    }
}

impl TestVstPlugin {
    // Exists for demonstration purposes and quick tests
    #[allow(dead_code)]
    fn use_medium_level_reaper(&mut self) {
        let context =
            ReaperPluginContext::from_vst_plugin(&self.host, reaper_vst_plugin::static_context())
                .unwrap();
        let low = reaper_low::Reaper::load(context);
        let mut med = reaper_medium::ReaperSession::new(low);
        {
            let (sender, receiver) = channel::<String>();
            med.reaper()
                .show_console_msg("Registering control surface ...");
            med.plugin_register_add_csurf_inst(MyControlSurface {
                reaper: med.reaper().clone(),
                receiver,
            })
            .expect("couldn't register control surface");
            med.reaper().show_console_msg("Registering action ...");
            med.plugin_register_add_hook_post_command::<MyHookPostCommand>()
                .expect("couldn't register hook post command");
            med.audio_reg_hardware_hook_add(MyOnAudioBuffer { sender, counter: 0 })
                .expect("couldn't register audio hook");
        }
        self.session = Some(med);
    }

    fn use_high_level_reaper(&mut self) {
        let guard = Reaper::guarded(|| {
            let context = ReaperPluginContext::from_vst_plugin(
                &self.host,
                reaper_vst_plugin::static_context(),
            )
            .unwrap();
            Reaper::setup_with_defaults(context, "info@helgoboss.org");
            let reaper = Reaper::get();
            reaper.activate();
            reaper.show_console_msg(c_str!("Loaded reaper-rs integration test VST plugin\n"));
            reaper.register_action(
                c_str!("reaperRsVstIntegrationTests"),
                c_str!("reaper-rs VST integration tests"),
                || reaper_test::execute_integration_test(|_| ()),
                ActionKind::NotToggleable,
            );
        });
        self.reaper_guard = Some(guard);
    }
}
