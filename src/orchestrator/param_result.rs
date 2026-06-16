use crate::{collection, errcode, index::{self, IndexType, identifier}, shared::{self, ValueType}, vixerr::Error};

#[derive(Clone)]
pub struct CreateCollectionParam {
    pub name: String,
    pub fields: Vec<CreateCollectionParamField>,
}

impl CreateCollectionParam {
    pub fn validate(&self) -> Result<(), Error> {
        self.validate_name()?;
        self.fields.iter().map(|f| f.validate()).collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }

    fn validate_name(&self) -> Result<(), Error> {
        if self.name.len() == 0 {
            return Err(Error::new(errcode::PARSE_ERROR, "name is empty"));
        }

        if self.name.starts_with('_') {
            return Err(Error::new(errcode::PARSE_ERROR, "name cannot start with _"));
        }

        let valid_chars = self.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid_chars {
            return Err(Error::new(errcode::PARSE_ERROR, "name should contain only alphanumeric and underscore"));
        }

        Ok(())
    }
}

impl TryFrom<&CreateCollectionParam> for collection::CreateCollectionParam {
    type Error = Error;

    fn try_from(value: &CreateCollectionParam) -> Result<Self, Self::Error> {
        Ok(collection::CreateCollectionParam {
            name: value.name.clone(),
            fields: value
                .fields
                .iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Clone)]
pub struct CreateCollectionParamField {
    pub name: String,
    pub field_type: shared::ValueType,
    pub index_type: IndexType,
}

impl CreateCollectionParamField {
    pub fn identifier() -> CreateCollectionParamField {
        CreateCollectionParamField {
            name: "_id".to_string(),
            field_type: ValueType::String,
            index_type: identifier::INDEX_TYPE,
        }
    }

    fn validate(&self) -> Result<(), Error> {
        self.validate_name()
    }

    fn validate_name(&self) -> Result<(), Error> {
        if self.name.len() == 0 {
            return Err(Error::new(errcode::PARSE_ERROR, "name is empty"));
        }

        if self.name.starts_with('_') {
            return Err(Error::new(errcode::PARSE_ERROR, "name cannot start with _"));
        }

        let valid_chars = self.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid_chars {
            return Err(Error::new(errcode::PARSE_ERROR, "name should contain only alphanumeric and underscore"));
        }

        Ok(())
    }
}

impl CreateCollectionParamField {
    pub fn index_create_param(&self, collection: &str) -> index::CreateParam {
        index::CreateParam {
            collection: collection.to_string(),
            field: self.name.clone(),
            field_type: self.field_type.clone(),
            index_type: self.index_type,
        }
    }
}

impl TryFrom<&CreateCollectionParamField> for collection::Field {
    type Error = Error;

    fn try_from(value: &CreateCollectionParamField) -> Result<Self, Self::Error> {
        Ok(collection::Field {
            name: value.name.clone(),
            field_type: value.field_type.clone(),
        })
    }
}
