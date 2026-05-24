use crate::collection::entity::Field;

#[derive(Debug)]
pub struct CreateCollectionParam {
    pub name: String,
    pub fields: Vec<Field>,
}

impl CreateCollectionParam {
    pub fn into_create_storage_param(self, id: String) -> CreateStorageParam {
        CreateStorageParam {
            id,
            name: self.name,
            fields: self.fields,
        }
    }
}

#[derive(Debug)]
pub struct CreateStorageParam {
    pub id: String,
    pub name: String,
    pub fields: Vec<Field>,
}
