use crate::collection::entity::Field;

pub struct CreateCollectionParam {
    pub name: String,
    pub fields: Vec<Field>,
}
