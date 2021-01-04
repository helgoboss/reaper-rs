use crossbeam_channel::{Receiver, Sender};

use crate::{
    local_run_loop_executor, run_loop_executor, ControlSurfaceMiddleware, MainThreadTask,
    MAIN_THREAD_TASK_BULK_SIZE,
};

#[derive(Debug)]
pub(crate) struct HelperMiddleware {
    logger: slog::Logger,
    // This is for executing futures.
    main_thread_executor: run_loop_executor::RunLoopExecutor,
    local_main_thread_executor: local_run_loop_executor::RunLoopExecutor,
}

impl HelperMiddleware {
    pub fn new(
        logger: slog::Logger,
        executor: run_loop_executor::RunLoopExecutor,
        local_executor: local_run_loop_executor::RunLoopExecutor,
    ) -> HelperMiddleware {
        HelperMiddleware {
            logger: logger.clone(),
            main_thread_executor: executor,
            local_main_thread_executor: local_executor,
        }
    }

    pub fn reset(&self) {
        self.discard_tasks();
    }

    fn discard_tasks(&self) {
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
}

impl ControlSurfaceMiddleware for HelperMiddleware {
    fn run(&mut self) {
        self.main_thread_executor.run();
        self.local_main_thread_executor.run();
    }
}
