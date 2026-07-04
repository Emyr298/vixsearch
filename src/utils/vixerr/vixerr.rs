use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    pub code: String,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error>>
}

impl Error {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            source: Option::None,
        }
    }

    pub fn code(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: "".to_string(),
            source: Option::None,
        }
    }

    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        self
    }

    pub fn wrap(mut self, source: impl std::error::Error + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    pub fn throw<T>(self) -> Result<T, Self> {
        Err(self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
