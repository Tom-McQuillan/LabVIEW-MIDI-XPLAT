//! The LabVIEW Interop module for MIDI project
//! Simplified version focused on User Events

pub mod errors;
#[cfg(feature = "link")]
mod labview;
pub mod memory;
#[cfg(feature = "sync")]
pub mod sync;
pub mod types;