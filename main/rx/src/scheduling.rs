use crate::run_loop_scheduler::RxTask;
use crossbeam_channel::Receiver;

pub struct SchedulingMiddleware {
    logger: slog::Logger,
    // This is for scheduling rxRust observables.
    // TODO-medium Remove, I ran into deadlocks with this thing.
    main_thread_rx_task_receiver: Receiver<RxTask>,
    bulk_size: usize,
}

impl SchedulingMiddleware {
    pub fn new(
        logger: slog::Logger,
        main_thread_rx_task_receiver: Receiver<RxTask>,
        bulk_size: usize,
    ) -> SchedulingMiddleware {
        SchedulingMiddleware {
            logger,
            main_thread_rx_task_receiver,
            bulk_size,
        }
    }

    pub fn reset(&self) {
        self.discard_tasks();
    }

    pub fn run(&mut self) {
        // Execute observables
        for task in self
            .main_thread_rx_task_receiver
            .try_iter()
            .take(self.bulk_size)
        {
            task();
        }
    }

    fn discard_tasks(&self) {
        let task_count = self.main_thread_rx_task_receiver.try_iter().count();
        if task_count > 0 {
            slog::warn!(self.logger, "Discarded main thread rx tasks on reactivation";
                "task_count" => task_count,
            );
        }
    }
}
