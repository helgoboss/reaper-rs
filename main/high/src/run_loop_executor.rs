//! Provides an executor for executing futures on a custom run loop.
use crossbeam_channel::{Receiver, Sender};
use {
    futures::{
        future::{BoxFuture, FutureExt},
        task::{waker_ref, ArcWake},
    },
    std::{
        future::Future,
        sync::{Arc, Mutex},
        task::{Context, Poll},
    },
};

/// Task executor that receives tasks off of a channel and runs them.
#[derive(Clone, Debug)]
pub struct RunLoopExecutor {
    ready_queue: Receiver<Arc<Task>>,
    bulk_size: usize,
}

/// `Spawner` spawns new futures onto the task channel.
#[derive(Clone, Debug)]
pub struct Spawner {
    task_sender: Sender<Arc<Task>>,
}

/// A future that can reschedule itself to be polled by an `Executor`.
struct Task {
    /// In-progress future that should be pushed to completion.
    ///
    /// The `Mutex` is not necessary for correctness, since we only have
    /// one thread executing tasks at once. However, Rust isn't smart
    /// enough to know that `future` is only mutated from one thread,
    /// so we need use the `Mutex` to prove thread-safety. A production
    /// executor would not need this, and could use `UnsafeCell` instead.
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    /// Handle to place the task itself back onto the task queue.
    task_sender: Sender<Arc<Task>>,
}

pub fn new_spawner_and_executor(capacity: usize, bulk_size: usize) -> (Spawner, RunLoopExecutor) {
    let (task_sender, ready_queue) = crossbeam_channel::bounded(capacity);
    (
        Spawner { task_sender },
        RunLoopExecutor {
            ready_queue,
            bulk_size,
        },
    )
}

impl Spawner {
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Implement `wake` by sending this task back onto the task channel
        // so that it will be polled again by the executor.
        let cloned = arc_self.clone();
        arc_self
            .task_sender
            .send(cloned)
            .expect("too many tasks queued");
    }
}

impl RunLoopExecutor {
    pub fn run(&self) {
        for task in self.ready_queue.try_iter().take(self.bulk_size) {
            // Take the future, and if it has not yet completed (is still Some),
            // poll it in an attempt to complete it.
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // Create a `LocalWaker` from the task itself
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                // `BoxFuture<T>` is a type alias for
                // `Pin<Box<dyn Future<Output = T> + Send + 'static>>`.
                // We can get a `Pin<&mut dyn Future + Send + 'static>`
                // from it by calling the `Pin::as_mut` method.
                if let Poll::Pending = future.as_mut().poll(context) {
                    // We're not done processing the future, so put it
                    // back in its task to be run again in the future.
                    *future_slot = Some(future);
                }
            }
        }
    }
}
