//! Synchronization with LabVIEW

use std::ffi::c_void;
use std::marker::PhantomData;

use crate::labview_interop::errors::Result;
use crate::labview_interop::labview::sync_api;
use crate::labview_interop::memory::MagicCookie;

type LVUserEventRef = MagicCookie;

/// Representation of a LabVIEW user event reference with type data.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct LVUserEvent<T> {
    reference: LVUserEventRef,
    _marker: PhantomData<T>,
}

impl<T> LVUserEvent<T> {
    /// Create a new user event from a raw reference
    pub fn from_raw(reference: u32) -> Self {
        Self {
            reference: MagicCookie::new(reference),
            _marker: PhantomData,
        }
    }

    /// Generate the user event with the provided data.
    pub fn post(&self, data: &mut T) -> Result<()> {
        let api = sync_api()?;
        let mg_err = unsafe {
            api.post_lv_user_event(self.reference, data as *mut T as *mut c_void)
        };
        mg_err.to_specific_result(())
    }
}

/// A LabVIEW occurrence which can be used to provide synchronization
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Occurrence(MagicCookie);

impl Occurrence {
    /// Create a new occurrence from a raw reference
    pub fn from_raw(reference: u32) -> Self {
        Self(MagicCookie::new(reference))
    }

    /// "set" generates the occurrence event which can be detected by LabVIEW.
    pub fn set(&self) -> Result<()> {
        let api = sync_api()?;
        let mg_err = unsafe { api.occur(self.0) };
        mg_err.to_specific_result(())
    }
}