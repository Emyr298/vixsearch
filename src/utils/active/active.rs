use std::{ops::Deref, sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}}};

struct Inner<T> {
    inner: T,
    is_deactivating: AtomicBool,
    access_count: AtomicUsize,
}

pub struct ActiveGuard<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Deref for ActiveGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner.inner
    }
}

impl<T> Drop for ActiveGuard<T> {
    fn drop(&mut self) {
        self.inner.access_count.fetch_sub(1, Ordering::AcqRel);
    }
}

pub struct ActiveHandle<T> {
    inner: Arc<Inner<T>>,
}

impl<T> ActiveHandle<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(Inner {
                inner,
                is_deactivating: AtomicBool::new(false),
                access_count: AtomicUsize::new(0),
            }),
        }
    }

    pub fn get(&self) -> Option<ActiveGuard<T>> {
        if self.inner.is_deactivating.load(Ordering::Acquire) {
            return None;
        }

        self.inner.access_count.fetch_add(1, Ordering::AcqRel);

        if self.inner.is_deactivating.load(Ordering::Acquire) {
            self.inner.access_count.fetch_sub(1, Ordering::AcqRel);
            return None;
        }

        Some(ActiveGuard { inner: self.inner.clone() })
    }

    pub fn mark_for_deactivation(&self) {
        self.inner.is_deactivating.store(true, Ordering::Release);
    }

    pub fn is_deactivable(&self) -> bool {
        self.inner.is_deactivating.load(Ordering::Acquire)
            && self.inner.access_count.load(Ordering::Acquire) == 0
    }
}