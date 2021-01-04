use crate::Reaper;
use crossbeam_channel::Receiver;
use reaper_medium::ControlSurface;

/// In the past this was a generic control surface that did lots of stuff, also user-provided code.
/// Now it's just for internal usage and should be used very sparingly - because we want to reduce
/// the side effects of setting up the high-level API to a minimum.
#[derive(Debug)]
pub(crate) struct HelperControlSurface {
    task_receiver: Receiver<HelperTask>,
}

pub(crate) enum HelperTask {
    ShowConsoleMsg(String),
}

impl HelperControlSurface {
    pub fn new(task_receiver: Receiver<HelperTask>) -> HelperControlSurface {
        HelperControlSurface { task_receiver }
    }
}

impl ControlSurface for HelperControlSurface {
    fn run(&mut self) {
        for task in self.task_receiver.try_iter().take(1) {
            use HelperTask::*;
            match task {
                ShowConsoleMsg(msg) => {
                    Reaper::get().show_console_msg(msg);
                }
            }
        }
    }
}
