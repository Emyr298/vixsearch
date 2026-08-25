use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;

use crate::utils::observer::observer::{Observer, ObserverGroupReader, ObserverGroupWriter};

pub struct ObserverGroup<EventT: 'static> {
    observers: ArcSwap<Vec<Arc<dyn Observer<EventT>>>>,
}

impl<EventT> ObserverGroup<EventT> {
    pub fn new(observers: Vec<Arc<dyn Observer<EventT>>>) -> (Arc<dyn ObserverGroupReader<EventT>>, Arc<dyn ObserverGroupWriter<EventT>>) {
        let arc = Arc::new(ObserverGroup {
            observers: ArcSwap::from_pointee(observers),
        });

        let reader: Arc<dyn ObserverGroupReader<EventT>> = arc.clone();
        let writer: Arc<dyn ObserverGroupWriter<EventT>> = arc;

        (reader, writer)
    }
}

impl<EventT> ObserverGroupReader<EventT> for ObserverGroup<EventT> {
    fn list(&self) -> Arc<Vec<Arc<dyn Observer<EventT>>>> {
        self.observers.load_full()
    }
}

impl<EventT> ObserverGroupWriter<EventT> for ObserverGroup<EventT> {
    fn register(&self, observer: Arc<dyn Observer<EventT>>) {
        let observers = self.observers.rcu(|o| {
            let mut next_observers = (**o).clone();
            next_observers.push(observer.clone());
            next_observers
        });
    }
}
