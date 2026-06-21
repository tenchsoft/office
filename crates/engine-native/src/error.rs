use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeEngineError {
    Io(String),
    UnsupportedFormat(String),
    InvalidModel(String),
}

impl fmt::Display for NativeEngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => formatter.write_str(message),
            Self::UnsupportedFormat(message) => formatter.write_str(message),
            Self::InvalidModel(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for NativeEngineError {}

pub(crate) fn invalid_model(message: impl Into<String>) -> NativeEngineError {
    NativeEngineError::InvalidModel(message.into())
}
