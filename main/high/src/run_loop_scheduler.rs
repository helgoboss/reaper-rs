//! Provides a scheduler for observing rxRust observables on a custom run loop.
use rxrust::prelude::*;
use std::time::Duration;

pub type RxTask = Box<dyn FnOnce() + Send>;

#[derive(Debug, Clone)]
pub struct RunLoopScheduler {
    sender: crossbeam_channel::Sender<RxTask>,
}

impl RunLoopScheduler {
    pub fn new(sender: crossbeam_channel::Sender<RxTask>) -> RunLoopScheduler {
        RunLoopScheduler { sender }
    }
}

impl Scheduler for &RunLoopScheduler {
    fn schedule<T: Send + 'static>(
        &self,
        task: impl FnOnce(SharedSubscription, T) + Send + 'static,
        delay: Option<Duration>,
        state: T,
    ) -> SharedSubscription {
        let subscription = SharedSubscription::default();
        let mut c_subscription = subscription.clone();
        let boxed_task = Box::new(move || {
            if !subscription.is_closed() {
                task(subscription, state);
            }
        });
        if let Some(delay) = delay {
            let sender = self.sender.clone();
            c_subscription.add(delay_task(delay, move || {
                sender.send(boxed_task).expect("couldn't send task");
            }))
        } else {
            self.sender.send(boxed_task).expect("couldn't send task");
        }
        c_subscription
    }
}
