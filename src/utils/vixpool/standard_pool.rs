use std::{sync::{Arc, atomic::{AtomicUsize, Ordering}}, thread::yield_now};

use threadpool::ThreadPool;

use crate::utils::{vixerr::Error, vixpool::{errors::POOL_QUEUE_FULL, pool::Pool}};

pub struct StandardPool {
    pool: ThreadPool,
    is_blocking: bool,
    queue_size_opt: Option<usize>,
    queue_counter: Arc<AtomicUsize>,
}

pub fn new_standard_pool(thread_size: usize, queue_size: Option<usize>, is_blocking: bool) -> Arc<dyn Pool> {
    Arc::new(StandardPool::new(thread_size, queue_size, is_blocking))
}

impl StandardPool {
    pub fn new(thread_size: usize, queue_size: Option<usize>, is_blocking: bool) -> Self {
        let pool = ThreadPool::new(thread_size);

        Self {
            pool,
            is_blocking,
            queue_size_opt: queue_size,
            queue_counter: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Pool for StandardPool {
    fn submit(&self, func: Box<dyn FnOnce() + Send + 'static>) -> Result<(), Error> {
        if let Some(queue_size) = self.queue_size_opt {
            loop {
                let queued = self.queue_counter.load(Ordering::Acquire);
                if queued >= queue_size {
                    if !self.is_blocking {
                        return Error::code(POOL_QUEUE_FULL)
                            .message(format!("pool queue is full, size: {}", queue_size))
                            .throw();
                    } else {
                        yield_now();
                        continue;
                    }
                }

                let acquire = self.queue_counter
                    .compare_exchange(
                        queued, 
                        queued + 1, 
                        Ordering::AcqRel, 
                        Ordering::Acquire)
                    .is_ok();

                if acquire {
                    break;
                }
            }
        }

        let is_finite = self.queue_size_opt.is_some();
        let counter = Arc::clone(&self.queue_counter);
        self.pool.execute(move || {
            func();

            if is_finite {
                counter.fetch_sub(1, Ordering::Release);
            }
        });
        Ok(())
    }
}
