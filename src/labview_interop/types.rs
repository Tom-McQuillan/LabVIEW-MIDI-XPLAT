//! LabVIEW data types

use crate::labview_interop::errors::Result;

/// LabVIEW status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
#[allow(non_camel_case_types)]
pub enum LVStatusCode {
    SUCCESS = 0,
    ZONE_ERROR = -4,
    FULL_ERROR = -2,
    ARG_ERROR = 1,
}

impl LVStatusCode {
    pub fn to_specific_result<T>(self, value: T) -> Result<T> {
        match self {
            LVStatusCode::SUCCESS => Ok(value),
            _ => Err(crate::labview_interop::errors::InternalError::InvalidHandle.into()),
        }
    }
}