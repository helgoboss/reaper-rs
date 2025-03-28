use futures_timer::Delay;
use reaper_high::{
    ActionKind, FutureMiddleware, FutureSupport, PluginInfo, Reaper, ReaperGuard,
    DEFAULT_MAIN_THREAD_TASK_BULK_SIZE,
};
use reaper_low::{reaper_vst_plugin, static_plugin_context, PluginContext};
use reaper_medium::{CommandId, ControlSurface, HookPostCommand, OnAudioBuffer, OnAudioBufferArgs};
use reaper_rx::{ControlSurfaceRx, ControlSurfaceRxMiddleware};
use rxrust::prelude::*;
use std::error::Error;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;
#[allow(deprecated)]
use vst::plugin::{HostCallback, Info, Plugin};
use vst::plugin_main;

plugin_main!(TestVstPlugin);
reaper_vst_plugin!();

#[derive(Default)]
struct TestVstPlugin {
    host: HostCallback,
    _session: Option<reaper_medium::ReaperSession>,
    _reaper_guard: Option<Arc<ReaperGuard>>,
}

#[allow(deprecated)]
impl Plugin for TestVstPlugin {
    fn new(host: HostCallback) -> Self {
        Self {
            host,
            _session: None,
            _reaper_guard: None,
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

impl OnAudioBuffer for MyOnAudioBuffer {
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

impl HookPostCommand for MyHookPostCommand {
    fn call(command_id: CommandId, _flag: i32) {
        println!("Command {command_id:?} executed")
    }
}

impl TestVstPlugin {
    // Exists for demonstration purposes and quick tests
    #[allow(dead_code)]
    fn use_medium_level_reaper(&mut self) {
        let context = PluginContext::from_vst_plugin(&self.host, static_plugin_context()).unwrap();
        let low = reaper_low::Reaper::load(context);
        let mut med = reaper_medium::ReaperSession::new(low);
        {
            let (sender, receiver) = channel::<String>();
            med.reaper()
                .show_console_msg("Registering control surface ...");
            #[derive(Debug)]
            struct MyControlSurface {
                reaper: reaper_medium::Reaper,
                receiver: Receiver<String>,
            }

            impl ControlSurface for MyControlSurface {
                fn run(&mut self) {
                    for msg in self.receiver.try_iter() {
                        self.reaper.show_console_msg(msg);
                    }
                }

                fn set_track_list_change(&self) {
                    println!("Track list changed!")
                }
            }
            med.plugin_register_add_csurf_inst(Box::new(MyControlSurface {
                reaper: med.reaper().clone(),
                receiver,
            }))
            .expect("couldn't register control surface");
            med.reaper().show_console_msg("Registering action ...");
            med.plugin_register_add_hook_post_command::<MyHookPostCommand>()
                .expect("couldn't register hook post command");
            med.audio_reg_hardware_hook_add(Box::new(MyOnAudioBuffer { sender, counter: 0 }))
                .expect("couldn't register audio hook");
        }
        self._session = Some(med);
    }

    fn use_high_level_reaper(&mut self) {
        let control_surface_rx = ControlSurfaceRx::new();
        let (spawner, executor) = reaper_high::run_loop_executor::new_spawner_and_executor(
            DEFAULT_MAIN_THREAD_TASK_BULK_SIZE,
        );
        let (local_spawner, local_executor) =
            reaper_high::local_run_loop_executor::new_spawner_and_executor(
                DEFAULT_MAIN_THREAD_TASK_BULK_SIZE,
            );
        let future_support = FutureSupport::new(spawner, local_spawner);
        let future_support_clone = future_support.clone();
        let host = self.host;
        let guard = Reaper::guarded(
            true,
            move || {
                let context =
                    PluginContext::from_vst_plugin(&host, static_plugin_context()).unwrap();
                let version = env!("CARGO_PKG_VERSION").to_string();
                Reaper::setup_with_defaults(
                    context,
                    PluginInfo {
                        plugin_name: "reaper-rs test VST plug-in".to_string(),
                        plugin_version: version.clone(),
                        plugin_version_long: version,
                        support_email_address: "info@helgoboss.org".to_string(),
                        update_url: "https://www.helgoboss.org/projects/helgobox".to_string(),
                    },
                )
                .expect("should be done only once");
                let reaper = Reaper::get();
                reaper.wake_up().unwrap();
                debug!("Loaded reaper-rs integration test VST plugin");
                reaper.register_action(
                    "reaperRsVstIntegrationTests",
                    "reaper-rs VST integration tests",
                    None,
                    move || {
                        future_support_clone.spawn_in_main_thread_from_main_thread(async {
                            reaper_test::execute_integration_test().await?;
                            Ok(())
                        });
                    },
                    ActionKind::NotToggleable,
                );
            },
            || || {},
        );
        self._reaper_guard = Some(guard);
        // Some Rx stuff
        #[derive(Debug)]
        struct CustomControlSurface {
            rx_middleware: ControlSurfaceRxMiddleware,
            future_middleware: FutureMiddleware,
        }
        impl ControlSurface for CustomControlSurface {
            fn run(&mut self) {
                self.rx_middleware.run();
                self.future_middleware.run();
            }
        }
        impl CustomControlSurface {
            fn new(
                rx_middleware: ControlSurfaceRxMiddleware,
                future_middleware: FutureMiddleware,
            ) -> Self {
                CustomControlSurface {
                    rx_middleware,
                    future_middleware,
                }
            }
        }
        let mut counter = 0;
        let control_surface = CustomControlSurface::new(
            ControlSurfaceRxMiddleware::new(control_surface_rx.clone()),
            FutureMiddleware::new(executor, local_executor),
        );
        let reaper = Reaper::get();
        // TODO-medium This should be unregistered when VST plug-in removed.
        reaper
            .medium_session()
            .plugin_register_add_csurf_inst(Box::new(control_surface))
            .unwrap();
        control_surface_rx.main_thread_idle().subscribe(move |_| {
            if counter > 10 {
                return;
            }
            Reaper::get().show_console_msg(format!("Main thread counter: {counter}\n"));
            counter += 1;
        });
        // Some future stuff
        future_support.spawn_in_main_thread(future_main());
    }
}

async fn future_main() -> Result<(), Box<dyn Error>> {
    Reaper::get().show_console_msg("Hello from future!\n");
    let result = calculate_something().await;
    Reaper::get().show_console_msg(format!("Calculated: {result}\n"));
    let result = calculate_something_else().await;
    Reaper::get().show_console_msg(format!("Calculated something else: {result}\n"));
    Ok(())
}

async fn calculate_something() -> i32 {
    Reaper::get().show_console_msg("Calculating something...\n");
    Delay::new(Duration::from_secs(3)).await;
    5
}

async fn calculate_something_else() -> i32 {
    Reaper::get().show_console_msg("Calculating something else...\n");
    Delay::new(Duration::from_secs(5)).await;
    10
}
