use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::utils::vixerr::Error;

pub trait Observer<EventT: 'static>: Send + Sync {
    fn observe(&self, event: &EventT);
}

pub trait ObserverGroupReader<EventT: 'static>: Send + Sync {
    fn list(&self) -> Arc<Vec<Arc<dyn Observer<EventT>>>>;
}

pub trait ObserverGroupWriter<EventT: 'static>: Send + Sync {
    fn register(&self, observer: Arc<dyn Observer<EventT>>);
}
