use std::fmt::Display;

pub struct ErrorBuilder {
    code: String,
    message: Option<String>,
}

impl ErrorBuilder {
    pub fn code(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: None,
        }
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn wrap(
        self,
        cause: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Error {
        Error {
            code: self.code,
            message: self.message,
            cause: cause.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn error(self, cause: impl Into<String>) -> Error {
        Error {
            code: self.code,
            message: self.message,
            cause: cause.into(),
            source: None,
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub code: String,
    pub message: Option<String>,
    pub cause: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl Error {
    pub fn code(code: impl Into<String>) -> ErrorBuilder {
        ErrorBuilder::code(code)
    }

    pub fn throw<T>(self) -> Result<T, Self> {
        Err(self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cause)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

pub fn full_cause(e: &dyn std::error::Error) -> String {
    let mut out = e.to_string();
    let mut cur = std::error::Error::source(e);

    while let Some(e) = cur {
        out.push_str(&format!(" -> {e}"));
        cur = std::error::Error::source(e);
    }
    out
}
