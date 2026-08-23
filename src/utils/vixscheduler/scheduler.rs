use std::{sync::Arc, thread, time::{Duration, Instant}};

pub struct Scheduler {
    task_fn: Arc<dyn Fn() + Send + Sync>,
    interval: Duration,
}

impl Scheduler {
    pub fn new(task_fn: impl Fn() + Send + Sync + 'static, interval: Duration) -> Self {
        Scheduler {
            task_fn: Arc::new(task_fn),
            interval,
        }
    }

    pub fn start(&self) {
        let interval = self.interval;
        let task_fn = Arc::clone(&self.task_fn);

        std::thread::spawn(move || {
            let mut next_tick = Instant::now() + interval;

            loop {
                task_fn();
                let now = Instant::now();
                if next_tick > now {
                    thread::sleep(interval);
                }
                next_tick += interval;
            }
        });
    }
}
