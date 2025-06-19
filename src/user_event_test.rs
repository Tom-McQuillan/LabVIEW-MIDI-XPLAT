//! Test functions for LabVIEW User Event integration
//! These functions can be called from LabVIEW to test the User Event functionality

use crate::labview_interop::sync::LVUserEvent;
use crate::labview_interop::types::LVStatusCode;
use crate::lv_midi::MidiEventData;
use std::os::raw::c_int;

/// Test function: Generate a test MIDI event
/// This function can be called from LabVIEW to test if User Events are working
#[no_mangle]
pub extern "C" fn test_generate_midi_event(user_event_ref: u32) -> c_int {
    let user_event: LVUserEvent<MidiEventData> = LVUserEvent::from_raw(user_event_ref);
    
    // Create a test MIDI event (Note On, Middle C, Channel 1)
    let mut test_event = MidiEventData {
        message_type: 1,      // Note On
        channel: 0,          // Channel 1 (0-based)
        note_or_controller: 60, // Middle C
        velocity_or_value: 127, // Maximum velocity
        raw_status: 0x90,    // Note On, Channel 1
    };
    
    match user_event.post(&mut test_event) {
        Ok(_) => LVStatusCode::SUCCESS as c_int,
        Err(_) => LVStatusCode::ARG_ERROR as c_int,
    }
}

/// Test function: Generate multiple test events
#[no_mangle]
pub extern "C" fn test_generate_chord_events(user_event_ref: u32) -> c_int {
    let user_event: LVUserEvent<MidiEventData> = LVUserEvent::from_raw(user_event_ref);
    
    // Generate a C Major chord (C-E-G)
    let chord_notes = [60, 64, 67]; // C4, E4, G4
    
    for &note in &chord_notes {
        let mut event = MidiEventData {
            message_type: 1,
            channel: 0,
            note_or_controller: note,
            velocity_or_value: 100,
            raw_status: 0x90,
        };
        
        if let Err(_) = user_event.post(&mut event) {
            return LVStatusCode::ARG_ERROR as c_int;
        }
        
        // Small delay between notes
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    
    LVStatusCode::SUCCESS as c_int
}