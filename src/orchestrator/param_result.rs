use crate::{collection, shared, vixerr::Error};

pub struct CreateCollectionParam {
    pub name: String,
    pub fields: Vec<CreateCollectionParamField>,
}

impl TryFrom<CreateCollectionParam> for collection::CreateCollectionParam {
    type Error = Error;

    fn try_from(value: CreateCollectionParam) -> Result<Self, Self::Error> {
        Ok(collection::CreateCollectionParam {
            name: value.name,
            fields: value
                .fields
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

pub struct CreateCollectionParamField {
    pub name: String,
    pub field_type: shared::ValueType,
}

impl TryFrom<CreateCollectionParamField> for collection::Field {
    type Error = Error;

    fn try_from(value: CreateCollectionParamField) -> Result<Self, Self::Error> {
        Ok(collection::Field {
            name: value.name,
            field_type: value.field_type,
        })
    }
}
