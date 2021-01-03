use crossbeam_channel::{Receiver, Sender};

use crate::{
    local_run_loop_executor, run_loop_executor, ControlSurfaceMiddleware, MainThreadTask,
    MAIN_THREAD_TASK_BULK_SIZE,
};
use winapi::_core::time::Duration;

pub(crate) enum HelperTask {
    LogMetrics,
}

#[derive(Debug)]
pub(crate) struct HelperMiddleware {
    logger: slog::Logger,
    // These two are for very simple scheduling. Most light-weight.
    main_thread_task_sender: Sender<MainThreadTask>,
    main_thread_task_receiver: Receiver<MainThreadTask>,
    // This is for executing futures.
    main_thread_executor: run_loop_executor::RunLoopExecutor,
    local_main_thread_executor: local_run_loop_executor::RunLoopExecutor,
    helper_middleware_task_receiver: Receiver<HelperTask>,
    #[cfg(feature = "control-surface-meter")]
    performance_monitor: crate::ControlSurfacePerformanceMonitor,
}

impl HelperMiddleware {
    pub fn new(
        logger: slog::Logger,
        main_thread_task_sender: Sender<MainThreadTask>,
        main_thread_task_receiver: Receiver<MainThreadTask>,
        helper_task_receiver: Receiver<HelperTask>,
        executor: run_loop_executor::RunLoopExecutor,
        local_executor: local_run_loop_executor::RunLoopExecutor,
    ) -> HelperMiddleware {
        HelperMiddleware {
            logger: logger.clone(),
            main_thread_task_sender,
            main_thread_task_receiver,
            main_thread_executor: executor,
            local_main_thread_executor: local_executor,
            helper_middleware_task_receiver: helper_task_receiver,
            #[cfg(feature = "control-surface-meter")]
            performance_monitor: crate::ControlSurfacePerformanceMonitor::new(
                logger,
                Duration::from_secs(30),
            ),
        }
    }

    pub fn reset(&self) {
        self.discard_tasks();
    }

    fn discard_tasks(&self) {
        self.discard_main_thread_tasks();
        self.discard_future_tasks();
    }

    fn discard_future_tasks(&self) {
        let shared_task_count = self.main_thread_executor.discard_tasks();
        let local_task_count = self.local_main_thread_executor.discard_tasks();
        let total_task_count = shared_task_count + local_task_count;
        if total_task_count > 0 {
            slog::warn!(self.logger, "Discarded future tasks on reactivation";
                "task_count" => total_task_count,
            );
        }
    }

    fn discard_main_thread_tasks(&self) {
        let task_count = self.main_thread_task_receiver.try_iter().count();
        if task_count > 0 {
            slog::warn!(self.logger, "Discarded main thread tasks on reactivation";
                "task_count" => task_count,
            );
        }
    }
}

impl ControlSurfaceMiddleware for HelperMiddleware {
    fn run(&mut self) {
        // Process plain main thread tasks in queue
        for task in self
            .main_thread_task_receiver
            .try_iter()
            .take(MAIN_THREAD_TASK_BULK_SIZE)
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
        // Execute futures
        self.main_thread_executor.run();
        self.local_main_thread_executor.run();
    }

    #[cfg(feature = "control-surface-meter")]
    fn handle_metrics(&mut self, metrics: &reaper_medium::ControlSurfaceMetrics) {
        self.performance_monitor.handle_metrics(metrics);
        // As long as the middleware task receiver doesn't get other kinds of tasks, we can do it
        // here - which has the advantage that we have the metrics at hand already.
        if let Ok(task) = self.helper_middleware_task_receiver.try_recv() {
            use HelperTask::*;
            match task {
                LogMetrics => self.performance_monitor.log_metrics(metrics),
            }
        }
    }
}
