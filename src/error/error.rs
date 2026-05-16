use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse {name}: {value}")]
    Parse { name: String, value: String },

    #[error("system error: {message}")]
    System { message: String },
}
