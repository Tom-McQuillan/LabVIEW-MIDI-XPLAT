//! Error handling for LabVIEW interop

use std::fmt;

pub type Result<T> = std::result::Result<T, LVInteropError>;

#[derive(Debug, Clone)]
pub enum LVInteropError {
    InternalError(InternalError),
}

#[derive(Debug, Clone)]
pub enum InternalError {
    NoLabviewApi(String),
    InvalidHandle,
    HandleCreationFailed,
}

impl fmt::Display for LVInteropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LVInteropError::InternalError(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternalError::NoLabviewApi(s) => write!(f, "LabVIEW API not available: {}", s),
            InternalError::InvalidHandle => write!(f, "Invalid handle"),
            InternalError::HandleCreationFailed => write!(f, "Handle creation failed"),
        }
    }
}

impl std::error::Error for LVInteropError {}
impl std::error::Error for InternalError {}

impl From<InternalError> for LVInteropError {
    fn from(e: InternalError) -> Self {
        LVInteropError::InternalError(e)
    }
}