use c_str_macro::c_str;

use reaper_rs_high::{ActionKind, Reaper, ReaperGuard};
use reaper_rs_low::ReaperPluginContext;
use reaper_rs_medium::{
    CommandId, MediumHookPostCommand, MediumOnAudioBuffer, MediumReaperControlSurface,
    OnAudioBufferArgs,
};
use std::cell::RefCell;
use std::panic::RefUnwindSafe;
use std::rc::{Rc, Weak};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use vst::plugin::{HostCallback, Info, Plugin};
use vst::plugin_main;

plugin_main!(TestVstPlugin);

#[derive(Default)]
struct TestVstPlugin {
    host: HostCallback,
    reaper: Option<Rc<RefCell<reaper_rs_medium::Reaper>>>,
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
        self.use_medium_level_reaper();
        // self.use_high_level_reaper();
    }
}

struct MyOnAudioBuffer {
    sender: std::sync::mpsc::Sender<String>,
    counter: u64,
}

impl MediumOnAudioBuffer for MyOnAudioBuffer {
    fn call(&mut self, args: OnAudioBufferArgs) {
        if self.counter % 100 == 0 {
            self.sender.send(format!(
                "Counter: {}, Args: {:?}, Channels: {:?}\n",
                self.counter,
                args,
                (args.reg.input_nch(), args.reg.output_nch())
            ));
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

struct MyControlSurface {
    reaper: Weak<RefCell<reaper_rs_medium::Reaper>>,
    receiver: Receiver<String>,
}

impl RefUnwindSafe for MyControlSurface {}

impl MediumReaperControlSurface for MyControlSurface {
    fn run(&mut self) {
        let reaper = self.reaper.upgrade().unwrap();
        let reaper = reaper.borrow();
        for msg in self.receiver.try_iter() {
            reaper.functions().show_console_msg(msg);
        }
    }

    fn set_track_list_change(&self) {
        println!("Track list changed!")
    }
}

impl TestVstPlugin {
    fn use_medium_level_reaper(&mut self) {
        let context = ReaperPluginContext::from_vst_plugin(self.host).unwrap();
        let low = reaper_rs_low::Reaper::load(&context);
        let medium = Rc::new(RefCell::new(reaper_rs_medium::Reaper::new(low)));
        {
            let (sender, receiver) = channel::<String>();
            let mut med = medium.borrow_mut();
            med.functions()
                .show_console_msg("Registering control surface ...");
            med.plugin_register_add_csurf_inst(MyControlSurface {
                reaper: Rc::downgrade(&medium),
                receiver,
            });
            med.functions().show_console_msg("Registering action ...");
            med.plugin_register_add_hookpostcommand::<MyHookPostCommand>();
            med.audio_reg_hardware_hook_add(MyOnAudioBuffer { sender, counter: 0 });
        }
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
