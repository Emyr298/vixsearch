use crate::collection::entity::Field;

#[derive(Debug)]
pub struct CreateCollectionParam {
    pub name: String,
    pub fields: Vec<Field>,
}
