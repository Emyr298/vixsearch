use crate::document::ValueType;

pub struct CreatePortParam {
    pub id: String,
    pub internal_id: String,
    pub fields: Vec<CreatePortParamField>,
}

pub struct CreatePortParamField {
    pub name: String,
    pub field_type: ValueType,
}

pub struct GetAllPortResult {
    pub collections: Vec<GetAllPortResultCollection>,
}

pub struct GetAllPortResultCollection {
    pub id: String,
    pub internal_id: String,
    pub fields: Vec<GetAllPortResultCollectionField>,
}

pub struct GetAllPortResultCollectionField {
    pub name: String,
    pub field_type: ValueType,
}
