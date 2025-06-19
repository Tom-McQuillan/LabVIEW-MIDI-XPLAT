//! Simplified memory module for MIDI project

use std::fmt::Debug;

/// Magic cookie type used for various reference types in the memory manager.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct MagicCookie(pub u32);

impl MagicCookie {
    pub fn new(value: u32) -> Self {
        MagicCookie(value)
    }
    
    pub fn as_raw(&self) -> u32 {
        self.0
    }
}