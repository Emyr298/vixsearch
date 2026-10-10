use std::sync::{Arc, RwLock};

use arc_swap::ArcSwap;

use crate::utils::{handler::handler::{Handler, HandlerGroupReader, HandlerGroupWriter}, observer::observer::{Observer, ObserverGroupReader, ObserverGroupWriter}};

pub struct HandlerGroup<EventT: 'static> {
    handlers: ArcSwap<Vec<Arc<dyn Handler<EventT>>>>,
}

impl<EventT> HandlerGroup<EventT> {
    pub fn new(handlers: Vec<Arc<dyn Handler<EventT>>>) -> (Arc<dyn HandlerGroupReader<EventT>>, Arc<dyn HandlerGroupWriter<EventT>>) {
        let arc = Arc::new(HandlerGroup {
            handlers: ArcSwap::from_pointee(handlers),
        });

        let reader: Arc<dyn HandlerGroupReader<EventT>> = arc.clone();
        let writer: Arc<dyn HandlerGroupWriter<EventT>> = arc;

        (reader, writer)
    }
}

impl<EventT> HandlerGroupReader<EventT> for HandlerGroup<EventT> {
    fn list(&self) -> Arc<Vec<Arc<dyn Handler<EventT>>>> {
        self.handlers.load_full()
    }
}

impl<EventT> HandlerGroupWriter<EventT> for HandlerGroup<EventT> {
    fn register(&self, handler: Arc<dyn Handler<EventT>>) {
        let handlers = self.handlers.rcu(|h| {
            let mut next_handlers = (**h).clone();
            next_handlers.push(handler.clone());
            next_handlers
        });
    }
}
