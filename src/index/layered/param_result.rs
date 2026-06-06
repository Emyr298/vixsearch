use crate::shared;

pub struct InsertParam {
    pub collection: String,
    pub id: String,
    pub field: String,
    pub value: shared::Value,
}

pub struct FlushParam {
    pub collection: String,
    pub field: String,
}
