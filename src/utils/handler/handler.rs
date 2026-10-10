use std::{sync::Arc};

use arc_swap::ArcSwap;

use crate::utils::vixerr::Error;

pub trait Handler<EventT: 'static>: Send + Sync {
    fn handle(&self, event: &EventT) -> Result<(), Error>;
}

pub trait HandlerGroupReader<EventT: 'static>: Send + Sync {
    fn list(&self) -> Arc<Vec<Arc<dyn Handler<EventT>>>>;
}

pub trait HandlerGroupWriter<EventT: 'static>: Send + Sync {
    fn register(&self, handler: Arc<dyn Handler<EventT>>);
}
