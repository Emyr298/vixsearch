use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{errcode::{self, PARSE_ERROR}, vixerr::Error};


// pub fn get_id(document: &Document) -> Result<String, Error> {
//     let Some(id_value) = document.get(IDENTIFIER_FIELD) else {
//         return Err(Error::new(errcode::FATAL_ERROR, "id not in document"));
//     };

//     let Value::String(id) = id_value else {
//         return Err(Error::new(errcode::FATAL_ERROR, "id must be a string"));
//     };

//     Ok(id.clone())
// }
