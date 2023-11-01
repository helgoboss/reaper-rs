use crossbeam_channel::{Receiver, Sender};

use crate::{Reaper, DEFAULT_MAIN_THREAD_TASK_BULK_SIZE};
use futures::channel::oneshot;
use std::time::{Duration, SystemTime};

pub struct TaskSupport {
    sender: Sender<MainThreadTask>,
}

// TODO-medium Is this correct? It was already like that when TaskSupport was a part of Reaper
// struct.
unsafe impl Sync for TaskSupport {}

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
        unsafe { self.do_later_in_main_thread_asap_internal(op) }
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread_from_main_thread_asap(
        &self,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        Reaper::get().require_main_thread();
        unsafe { self.do_later_in_main_thread_asap_internal(op) }
    }

    /// Unsafe because doesn't require send (which should be required in the general case).
    unsafe fn do_later_in_main_thread_asap_internal(
        &self,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        self.sender
            .send(MainThreadTask::new(Box::new(op), None))
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

    /// Unsafe because doesn't require send (which should be required in the general case).
    unsafe fn do_in_main_thread_asap_internal(
        &self,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        if Reaper::get().is_in_main_thread() {
            op();
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
        unsafe { self.do_later_in_main_thread_internal(waiting_time, op) }
    }

    // Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    // deactivated).
    pub fn do_later_in_main_thread_from_main_thread(
        &self,
        waiting_time: Duration,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        Reaper::get().require_main_thread();
        unsafe { self.do_later_in_main_thread_internal(waiting_time, op) }
    }

    /// Unsafe because doesn't require send (which should be required in the general case).
    unsafe fn do_later_in_main_thread_internal(
        &self,
        waiting_time: Duration,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        self.sender
            .send(MainThreadTask::new(
                Box::new(op),
                Some(SystemTime::now() + waiting_time),
            ))
            .map_err(|_| "channel disconnected")
    }

    /// Thread-safe. Returns an error if task queue is full (typically if Reaper has been
    /// deactivated).
    pub fn do_in_main_thread_asap(
        &self,
        op: impl FnOnce() + Send + 'static,
    ) -> Result<(), &'static str> {
        unsafe { self.do_in_main_thread_asap_internal(op) }
    }

    /// Panics if not in main thread. The difference to `do_in_main_thread_asap()` is that `Send` is
    /// not required. Perfect for capturing `Rc`s.
    pub fn do_in_main_thread_from_main_thread_asap(
        &self,
        op: impl FnOnce() + 'static,
    ) -> Result<(), &'static str> {
        Reaper::get().require_main_thread();
        unsafe { self.do_in_main_thread_asap_internal(op) }
    }
}

#[derive(Debug)]
pub struct MainTaskMiddleware {
    logger: slog::Logger,
    main_thread_task_sender: Sender<MainThreadTask>,
    main_thread_task_receiver: Receiver<MainThreadTask>,
}

impl MainTaskMiddleware {
    pub fn new(
        logger: slog::Logger,
        main_thread_task_sender: Sender<MainThreadTask>,
        main_thread_task_receiver: Receiver<MainThreadTask>,
    ) -> MainTaskMiddleware {
        MainTaskMiddleware {
            logger,
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
            slog::warn!(self.logger, "Discarded main thread tasks on reactivation";
                "task_count" => task_count,
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
                None => (task.op)(),
                Some(t) => {
                    if std::time::SystemTime::now() < t {
                        self.main_thread_task_sender
                            .send(task)
                            .expect("couldn't reschedule main thread task");
                    } else {
                        (task.op)()
                    }
                }
            }
        }
    }
}

type MainThreadTaskOp = Box<dyn FnOnce() + 'static>;

pub struct MainThreadTask {
    pub desired_execution_time: Option<std::time::SystemTime>,
    pub op: MainThreadTaskOp,
}

impl MainThreadTask {
    pub fn new(
        op: MainThreadTaskOp,
        desired_execution_time: Option<std::time::SystemTime>,
    ) -> MainThreadTask {
        MainThreadTask {
            desired_execution_time,
            op,
        }
    }
}
