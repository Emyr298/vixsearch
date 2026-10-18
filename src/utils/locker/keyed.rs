use dashmap::DashMap;
use std::sync::{Arc, Mutex};

pub struct KeyedLockManager {
    lock_by_key: DashMap<String, Arc<Mutex<()>>>,
}

impl KeyedLockManager {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            lock_by_key: DashMap::new()
        })
    }

    pub fn with_lock<ResultT, Error>(
        &self,
        key: &str,
        task_fn: Box<dyn FnOnce() -> Result<ResultT, Error> + Send + 'static>,
    ) -> Result<ResultT, Error> {
        let _cleanup = Cleanup { manager: self, key }; // drops after _guard and lock

        let lock = self.lock_by_key.entry(key.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();

        let _guard = lock.lock().unwrap_or_else(|e| e.into_inner());

        task_fn()
    }
}

struct Cleanup<'a> {
    manager: &'a KeyedLockManager,
    key: &'a str,
}

impl Drop for Cleanup<'_> {
    fn drop(&mut self) {
        self.manager.lock_by_key.remove_if(self.key, |_, v| Arc::strong_count(v) == 1);
    }
}
