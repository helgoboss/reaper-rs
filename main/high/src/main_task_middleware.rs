use crossbeam_channel::{Receiver, Sender};

use crate::{Reaper, DEFAULT_MAIN_THREAD_TASK_BULK_SIZE};
use fragile::Fragile;
use futures::channel::oneshot;
use std::time::{Duration, SystemTime};
use tracing::warn;

pub struct TaskSupport {
    sender: Sender<MainThreadTask>,
}

impl TaskSupport {
    pub fn new(sender: Sender<MainThreadTask>) -> TaskSupport {
        TaskSupport { sender }
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread_asap(
        &self,
        op: impl FnOnce() + Send + 'static,
    ) -> Result<(), &'static str> {
        let op = MainThreadTaskOp::Send(Box::new(op));
        self.do_later_in_main_thread_asap_internal(op)
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread_from_main_thread_asap(
        &self,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        Reaper::get().require_main_thread();
        let op = MainThreadTaskOp::NonSend(Fragile::new(Box::new(op)));
        self.do_later_in_main_thread_asap_internal(op)
    }

    fn do_later_in_main_thread_asap_internal(
        &self,
        op: MainThreadTaskOp,
    ) -> Result<(), &'static str> {
        self.sender
            .send(MainThreadTask::new(op, None))
            .map_err(|_| "channel disconnected")
    }

    // TODO-medium Proper errors
    pub async fn main_thread_future<R: 'static + Send>(
        &self,
        op: impl FnOnce() -> R + 'static + Send,
    ) -> Result<R, &'static str> {
        if Reaper::get().is_in_main_thread() {
            Ok(op())
        } else {
            let (tx, rx) = oneshot::channel();
            self.do_later_in_main_thread_asap(move || {
                tx.send(op()).ok().expect("couldn't send");
            })?;
            rx.await
                .map_err(|_| "error when awaiting main thread future")
        }
    }

    fn do_in_main_thread_asap_internal(&self, op: MainThreadTaskOp) -> Result<(), &'static str> {
        if Reaper::get().is_in_main_thread() {
            op.execute();
            Ok(())
        } else {
            self.do_later_in_main_thread_asap_internal(op)
        }
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread(
        &self,
        waiting_time: Duration,
        op: impl FnOnce() + Send + 'static,
    ) -> Result<(), &'static str> {
        let op = MainThreadTaskOp::Send(Box::new(op));
        self.do_later_in_main_thread_internal(waiting_time, op)
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread_from_main_thread(
        &self,
        waiting_time: Duration,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        Reaper::get().require_main_thread();
        let op = MainThreadTaskOp::NonSend(Fragile::new(Box::new(op)));
        self.do_later_in_main_thread_internal(waiting_time, op)
    }

    fn do_later_in_main_thread_internal(
        &self,
        waiting_time: Duration,
        op: MainThreadTaskOp,
    ) -> Result<(), &'static str> {
        let task = MainThreadTask::new(op, Some(SystemTime::now() + waiting_time));
        self.sender.send(task).map_err(|_| "channel disconnected")
    }

    /// Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    /// deactivated).
    pub fn do_in_main_thread_asap(
        &self,
        op: impl FnOnce() + Send + 'static,
    ) -> Result<(), &'static str> {
        let op = MainThreadTaskOp::Send(Box::new(op));
        self.do_in_main_thread_asap_internal(op)
    }

    /// Panics if not in main thread. The difference to `do_in_main_thread_asap()` is that `Send` is
    /// not required. Perfect for capturing `Rc`s.
    pub fn do_in_main_thread_from_main_thread_asap(
        &self,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        Reaper::get().require_main_thread();
        let op = MainThreadTaskOp::NonSend(Fragile::new(Box::new(op)));
        self.do_in_main_thread_asap_internal(op)
    }
}

#[derive(Debug)]
pub struct MainTaskMiddleware {
    main_thread_task_sender: Sender<MainThreadTask>,
    main_thread_task_receiver: Receiver<MainThreadTask>,
}

impl MainTaskMiddleware {
    pub fn new(
        main_thread_task_sender: Sender<MainThreadTask>,
        main_thread_task_receiver: Receiver<MainThreadTask>,
    ) -> MainTaskMiddleware {
        MainTaskMiddleware {
            main_thread_task_sender,
            main_thread_task_receiver,
        }
    }

    pub fn reset(&self) {
        self.discard_tasks();
    }

    fn discard_tasks(&self) {
        let task_count = self.main_thread_task_receiver.try_iter().count();
        if task_count > 0 {
            warn!(
                msg = "Discarded main thread tasks on reactivation",
                task_count,
            );
        }
    }

    pub fn run(&mut self) {
        // Process plain main thread tasks in queue
        for task in self
            .main_thread_task_receiver
            .try_iter()
            .take(DEFAULT_MAIN_THREAD_TASK_BULK_SIZE)
        {
            match task.desired_execution_time {
                None => task.op.execute(),
                Some(t) => {
                    if SystemTime::now() < t {
                        self.main_thread_task_sender
                            .send(task)
                            .expect("couldn't reschedule main thread task");
                    } else {
                        task.op.execute()
                    }
                }
            }
        }
    }
}

enum MainThreadTaskOp {
    NonSend(Fragile<Box<dyn FnOnce() + 'static>>),
    Send(Box<dyn FnOnce() + Send + 'static>),
}

impl MainThreadTaskOp {
    fn execute(self) {
        match self {
            MainThreadTaskOp::NonSend(op) => op.into_inner()(),
            MainThreadTaskOp::Send(op) => op(),
        }
    }
}

pub struct MainThreadTask {
    desired_execution_time: Option<SystemTime>,
    op: MainThreadTaskOp,
}

impl MainThreadTask {
    fn new(op: MainThreadTaskOp, desired_execution_time: Option<SystemTime>) -> MainThreadTask {
        MainThreadTask {
            desired_execution_time,
            op,
        }
    }
}
