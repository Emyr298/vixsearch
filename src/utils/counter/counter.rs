use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

pub trait Counter: Send + Sync {
    fn next(&self) -> u64;
}

pub trait CounterLoader: Send + Sync {
    fn load(&self, initial_value: u64);
}

pub struct CounterImpl {
    value: AtomicU64,
}

impl CounterImpl {
    pub fn new(initial_value: u64) -> (Arc<dyn Counter>, Arc<dyn CounterLoader>) {
        let arc = Arc::new(Self {
            value: AtomicU64::new(initial_value),
        });

        let counter_arc: Arc<dyn Counter> = arc.clone();
        let loader_arc: Arc<dyn CounterLoader> = arc.clone();

        (counter_arc, loader_arc)
    }
}

impl CounterLoader for CounterImpl {
    fn load(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed)
    }
}

impl Counter for CounterImpl {
    fn next(&self) -> u64 {
        self.value.fetch_add(1, Ordering::Relaxed)
    }
}
