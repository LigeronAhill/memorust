use std::{fmt::Display, num::TryFromIntError, string::FromUtf8Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotAnArray,
    Incomplete,
    Custom(String),
}
impl core::error::Error for Error {}
pub type Result<T> = core::result::Result<T, Error>;
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Custom(s) => write!(f, "Custom error: '{s}'"),
            Self::Incomplete => write!(f, "Not enough data"),
            Self::NotAnArray => write!(f, "Frame type is not an array!"),
        }
    }
}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Custom(value.to_string())
    }
}
impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Custom(value)
    }
}
impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Error {
        "protocol error; invalid frame format".into()
    }
}
impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}
