use crate::{collection::port_param_result::{CreatePortParam, CreatePortParamField}, document::ValueType, errcode::PARSE_ERROR, utils::vixerr::Error};

pub struct CreateParam {
    pub id: String,
    pub fields: Vec<CreateParamField>,
}

impl CreateParam {
    pub fn validate(&self) -> Result<(), Error> {
        if self.id.len() == 0 {
            return Error::code(PARSE_ERROR)
                .message("id is empty")
                .throw();
        }

        if self.id.starts_with('_') {
            return Error::code(PARSE_ERROR)
                .message("id cannot start with _")
                .throw();
        }

        let valid_chars = self.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid_chars {
            return Error::code(PARSE_ERROR)
                .message("id should contain only alphanumeric and underscore")
                .throw();
        }

        self.fields.iter()
            .map(|f| f.validate())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }

    pub fn create_port_param(&self) -> CreatePortParam {
        CreatePortParam {
            id: self.id.to_string(),
            fields: self.fields.iter()
                .map(|field| field.create_port_param_field())
                .collect(),
        }
    }

//     pub fn into_create_storage_param(self, id: String) -> CreateStorageParam {
//         CreateStorageParam {
//             id,
//             name: self.name,
//             fields: self.fields,
//         }
//     }
}

#[derive(Clone)]
pub struct CreateParamField {
    pub name: String,
    pub field_type: ValueType,
}

impl CreateParamField {
    pub fn validate(&self) -> Result<(), Error> {
        if self.name.len() == 0 {
            return Error::code(PARSE_ERROR)
                .message("field name is empty")
                .throw();
        }

        if self.name.starts_with('_') {
            return Error::code(PARSE_ERROR)
                .message("field name cannot start with _")
                .throw();
        }

        let valid_chars = self.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid_chars {
            return Error::code(PARSE_ERROR)
                .message("field name should contain only alphanumeric and underscore")
                .throw();
        }

        Ok(())
    }

    pub fn create_port_param_field(&self) -> CreatePortParamField {
        CreatePortParamField {
            name: self.name.to_string(),
            field_type: self.field_type.clone(),
        }
    }
}
