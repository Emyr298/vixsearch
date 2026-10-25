pub enum CollectionLifecycleEvent {
    Loaded(CollectionLifecycleEventLoaded),
    Created(CollectionLifecycleEventCreated),
    Deleted(CollectionLifecycleEventDeleted),
}

pub struct CollectionLifecycleEventLoaded {
    pub collection_ids: Vec<String>,
}

pub struct CollectionLifecycleEventCreated {
    pub collection_id: String,
}

pub struct CollectionLifecycleEventDeleted {
    pub collection_id: String,
}
