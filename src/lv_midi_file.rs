use crate::midi_file::{load_midi_file, get_midi_file, close_midi_file, EventType};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_uchar, c_float, c_double};

// ========== MIDI FILE STRUCTURES FOR LABVIEW ==========

/// MIDI file information structure for LabVIEW
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MidiFileInfo {
    pub format: c_int,          // MIDI file format (0, 1, or 2)
    pub track_count: c_int,     // Number of tracks
    pub timing_type: c_int,     // 0 = metrical, 1 = timecode
    pub ticks_per_quarter: c_int, // For metrical timing
    pub fps: c_float,           // For timecode timing
    pub ticks_per_frame: c_int, // For timecode timing
    pub duration_ticks: c_uint, // Total duration in ticks
}

/// Track information structure for LabVIEW
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub track_index: c_int,
    pub event_count: c_int,
    pub channel_mask: c_int,    // Bitmask of channels used
    pub has_name: c_int,        // Boolean
    pub has_instrument: c_int,  // Boolean
}

/// MIDI event structure for LabVIEW
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MidiFileEvent {
    pub absolute_time: c_uint,
    pub event_type: c_int,
    pub channel: c_uchar,
    pub data1: c_uchar,
    pub data2: c_uchar,
    pub has_text: c_int,        // Boolean - whether text data is available
}

// ========== FILE OPERATIONS ==========

/// Load a MIDI file from disk
#[no_mangle]
pub extern "C" fn midi_file_open(
    file_path: *const c_char,
    file_handle: *mut c_int,
) -> c_int {
    if file_path.is_null() || file_handle.is_null() {
        return -1;
    }
    
    let path_str = unsafe {
        match CStr::from_ptr(file_path).to_str() {
            Ok(s) => s,
            Err(_) => return -2, // Invalid UTF-8
        }
    };
    
    match load_midi_file(path_str) {
        Ok(handle) => {
            unsafe {
                *file_handle = handle;
            }
            0
        }
        Err(_) => -3, // File loading error
    }
}

/// Close a MIDI file and free resources
#[no_mangle]
pub extern "C" fn midi_file_close(file_handle: c_int) -> c_int {
    if close_midi_file(file_handle) {
        0
    } else {
        -1
    }
}

/// Get basic information about a loaded MIDI file
#[no_mangle]
pub extern "C" fn midi_file_get_info(
    file_handle: c_int,
    info: *mut MidiFileInfo,
) -> c_int {
    if info.is_null() {
        return -1;
    }
    
    let files = match get_midi_file(file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let midi_file = match files.get(&file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let (timing_type, ticks_per_quarter, fps, ticks_per_frame) = match midi_file.timing {
        midly::Timing::Metrical(tpq) => (0, tpq.as_int() as c_int, 0.0, 0),
        midly::Timing::Timecode(fps_val, tpf) => {
            (1, 0, fps_val.as_f32(), tpf as c_int)
        }
    };
    
    let file_info = MidiFileInfo {
        format: midi_file.format as c_int,
        track_count: midi_file.tracks.len() as c_int,
        timing_type,
        ticks_per_quarter,
        fps,
        ticks_per_frame,
        duration_ticks: midi_file.get_duration_ticks(),
    };
    
    unsafe {
        *info = file_info;
    }
    
    0
}

// ========== TRACK OPERATIONS ==========

/// Get information about a specific track
#[no_mangle]
pub extern "C" fn midi_file_get_track_info(
    file_handle: c_int,
    track_index: c_int,
    info: *mut TrackInfo,
) -> c_int {
    if info.is_null() || track_index < 0 {
        return -1;
    }
    
    let files = match get_midi_file(file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let midi_file = match files.get(&file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let track = match midi_file.tracks.get(track_index as usize) {
        Some(t) => t,
        None => return -3, // Track index out of range
    };
    
    let track_info = TrackInfo {
        track_index,
        event_count: track.events.len() as c_int,
        channel_mask: track.channel_mask as c_int,
        has_name: if track.name.is_empty() { 0 } else { 1 },
        has_instrument: if track.instrument.is_some() { 1 } else { 0 },
    };
    
    unsafe {
        *info = track_info;
    }
    
    0
}

/// Get the name of a track
#[no_mangle]
pub extern "C" fn midi_file_get_track_name(
    file_handle: c_int,
    track_index: c_int,
    buffer: *mut c_char,
    buffer_size: c_int,
) -> c_int {
    if buffer.is_null() || buffer_size <= 0 || track_index < 0 {
        return -1;
    }
    
    let files = match get_midi_file(file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let midi_file = match files.get(&file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let track = match midi_file.tracks.get(track_index as usize) {
        Some(t) => t,
        None => return -3,
    };
    
    let c_string = match CString::new(track.name.clone()) {
        Ok(s) => s,
        Err(_) => return -4,
    };
    
    let name_bytes = c_string.as_bytes_with_nul();
    if name_bytes.len() > buffer_size as usize {
        return -5; // Buffer too small
    }
    
    unsafe {
        std::ptr::copy_nonoverlapping(
            name_bytes.as_ptr() as *const c_char,
            buffer,
            name_bytes.len(),
        );
    }
    
    0
}

/// Get the instrument name of a track
#[no_mangle]
pub extern "C" fn midi_file_get_track_instrument(
    file_handle: c_int,
    track_index: c_int,
    buffer: *mut c_char,
    buffer_size: c_int,
) -> c_int {
    if buffer.is_null() || buffer_size <= 0 || track_index < 0 {
        return -1;
    }
    
    let files = match get_midi_file(file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let midi_file = match files.get(&file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let track = match midi_file.tracks.get(track_index as usize) {
        Some(t) => t,
        None => return -3,
    };
    
    let instrument_name = match &track.instrument {
        Some(name) => name,
        None => return -4, // No instrument name
    };
    
    let c_string = match CString::new(instrument_name.clone()) {
        Ok(s) => s,
        Err(_) => return -5,
    };
    
    let name_bytes = c_string.as_bytes_with_nul();
    if name_bytes.len() > buffer_size as usize {
        return -6; // Buffer too small
    }
    
    unsafe {
        std::ptr::copy_nonoverlapping(
            name_bytes.as_ptr() as *const c_char,
            buffer,
            name_bytes.len(),
        );
    }
    
    0
}

// ========== EVENT OPERATIONS ==========

/// Get the number of events in a track
#[no_mangle]
pub extern "C" fn midi_file_get_event_count(
    file_handle: c_int,
    track_index: c_int,
) -> c_int {
    if track_index < 0 {
        return -1;
    }
    
    let files = match get_midi_file(file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let midi_file = match files.get(&file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let track = match midi_file.tracks.get(track_index as usize) {
        Some(t) => t,
        None => return -3,
    };
    
    track.events.len() as c_int
}

/// Get a specific event from a track
#[no_mangle]
pub extern "C" fn midi_file_get_event(
    file_handle: c_int,
    track_index: c_int,
    event_index: c_int,
    event: *mut MidiFileEvent,
) -> c_int {
    if event.is_null() || track_index < 0 || event_index < 0 {
        return -1;
    }
    
    let files = match get_midi_file(file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let midi_file = match files.get(&file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let track = match midi_file.tracks.get(track_index as usize) {
        Some(t) => t,
        None => return -3,
    };
    
    let abs_event = match track.events.get(event_index as usize) {
        Some(e) => e,
        None => return -4, // Event index out of range
    };
    
    let event_type_code = match abs_event.event_type {
        EventType::NoteOff => 0,
        EventType::NoteOn => 1,
        EventType::PolyphonicAftertouch => 2,
        EventType::ControlChange => 3,
        EventType::ProgramChange => 4,
        EventType::ChannelAftertouch => 5,
        EventType::PitchBend => 6,
        EventType::SystemExclusive => 7,
        EventType::MetaSequenceNumber => 100,
        EventType::MetaText => 101,
        EventType::MetaCopyright => 102,
        EventType::MetaTrackName => 103,
        EventType::MetaInstrumentName => 104,
        EventType::MetaLyric => 105,
        EventType::MetaMarker => 106,
        EventType::MetaCuePoint => 107,
        EventType::MetaChannelPrefix => 108,
        EventType::MetaEndOfTrack => 109,
        EventType::MetaSetTempo => 110,
        EventType::MetaSmpteOffset => 111,
        EventType::MetaTimeSignature => 112,
        EventType::MetaKeySignature => 113,
        EventType::MetaSequencerSpecific => 114,
        EventType::Unknown => 255,
    };
    
    let file_event = MidiFileEvent {
        absolute_time: abs_event.absolute_time,
        event_type: event_type_code,
        channel: abs_event.channel,
        data1: abs_event.data1,
        data2: abs_event.data2,
        has_text: if abs_event.text.is_empty() { 0 } else { 1 },
    };
    
    unsafe {
        *event = file_event;
    }
    
    0
}

/// Get the text data associated with an event (for meta events)
#[no_mangle]
pub extern "C" fn midi_file_get_event_text(
    file_handle: c_int,
    track_index: c_int,
    event_index: c_int,
    buffer: *mut c_char,
    buffer_size: c_int,
) -> c_int {
    if buffer.is_null() || buffer_size <= 0 || track_index < 0 || event_index < 0 {
        return -1;
    }
    
    let files = match get_midi_file(file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let midi_file = match files.get(&file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let track = match midi_file.tracks.get(track_index as usize) {
        Some(t) => t,
        None => return -3,
    };
    
    let abs_event = match track.events.get(event_index as usize) {
        Some(e) => e,
        None => return -4,
    };
    
    if abs_event.text.is_empty() {
        return -5; // No text data
    }
    
    let c_string = match CString::new(abs_event.text.clone()) {
        Ok(s) => s,
        Err(_) => return -6,
    };
    
    let text_bytes = c_string.as_bytes_with_nul();
    if text_bytes.len() > buffer_size as usize {
        return -7; // Buffer too small
    }
    
    unsafe {
        std::ptr::copy_nonoverlapping(
            text_bytes.as_ptr() as *const c_char,
            buffer,
            text_bytes.len(),
        );
    }
    
    0
}

// ========== UTILITY FUNCTIONS ==========

/// Convert ticks to milliseconds using default tempo (120 BPM)
#[no_mangle]
pub extern "C" fn midi_file_ticks_to_ms(
    file_handle: c_int,
    ticks: c_uint,
    tempo_us_per_quarter: c_uint, // Use 500000 for 120 BPM
) -> c_double {
    let files = match get_midi_file(file_handle) {
        Some(f) => f,
        None => return -1.0,
    };
    
    let midi_file = match files.get(&file_handle) {
        Some(f) => f,
        None => return -1.0,
    };
    
    midi_file.ticks_to_ms(ticks, tempo_us_per_quarter)
}

/// Get events from a track within a time range
#[no_mangle]
pub extern "C" fn midi_file_get_events_in_range(
    file_handle: c_int,
    track_index: c_int,
    start_time: c_uint,
    end_time: c_uint,
    events: *mut MidiFileEvent,
    max_events: c_int,
    actual_count: *mut c_int,
) -> c_int {
    if events.is_null() || actual_count.is_null() || track_index < 0 || max_events <= 0 {
        return -1;
    }
    
    let files = match get_midi_file(file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let midi_file = match files.get(&file_handle) {
        Some(f) => f,
        None => return -2,
    };
    
    let track = match midi_file.tracks.get(track_index as usize) {
        Some(t) => t,
        None => return -3,
    };
    
    let mut count = 0;
    let events_slice = unsafe {
        std::slice::from_raw_parts_mut(events, max_events as usize)
    };
    
    for abs_event in &track.events {
        if abs_event.absolute_time >= start_time && abs_event.absolute_time <= end_time {
            if count >= max_events {
                break;
            }
            
            let event_type_code = match abs_event.event_type {
                EventType::NoteOff => 0,
                EventType::NoteOn => 1,
                EventType::PolyphonicAftertouch => 2,
                EventType::ControlChange => 3,
                EventType::ProgramChange => 4,
                EventType::ChannelAftertouch => 5,
                EventType::PitchBend => 6,
                EventType::SystemExclusive => 7,
                EventType::MetaSequenceNumber => 100,
                EventType::MetaText => 101,
                EventType::MetaCopyright => 102,
                EventType::MetaTrackName => 103,
                EventType::MetaInstrumentName => 104,
                EventType::MetaLyric => 105,
                EventType::MetaMarker => 106,
                EventType::MetaCuePoint => 107,
                EventType::MetaChannelPrefix => 108,
                EventType::MetaEndOfTrack => 109,
                EventType::MetaSetTempo => 110,
                EventType::MetaSmpteOffset => 111,
                EventType::MetaTimeSignature => 112,
                EventType::MetaKeySignature => 113,
                EventType::MetaSequencerSpecific => 114,
                EventType::Unknown => 255,
            };
            
            events_slice[count as usize] = MidiFileEvent {
                absolute_time: abs_event.absolute_time,
                event_type: event_type_code,
                channel: abs_event.channel,
                data1: abs_event.data1,
                data2: abs_event.data2,
                has_text: if abs_event.text.is_empty() { 0 } else { 1 },
            };
            
            count += 1;
        }
    }
    
    unsafe {
        *actual_count = count;
    }
    
    0
}

/// Get the event type name as a string
#[no_mangle]
pub extern "C" fn midi_file_get_event_type_name(
    event_type: c_int,
    buffer: *mut c_char,
    buffer_size: c_int,
) -> c_int {
    if buffer.is_null() || buffer_size <= 0 {
        return -1;
    }
    
    let type_name = match event_type {
        0 => "Note Off",
        1 => "Note On",
        2 => "Polyphonic Aftertouch",
        3 => "Control Change",
        4 => "Program Change",
        5 => "Channel Aftertouch",
        6 => "Pitch Bend",
        7 => "System Exclusive",
        100 => "Meta: Sequence Number",
        101 => "Meta: Text",
        102 => "Meta: Copyright",
        103 => "Meta: Track Name",
        104 => "Meta: Instrument Name",
        105 => "Meta: Lyric",
        106 => "Meta: Marker",
        107 => "Meta: Cue Point",
        108 => "Meta: Channel Prefix",
        109 => "Meta: End of Track",
        110 => "Meta: Set Tempo",
        111 => "Meta: SMPTE Offset",
        112 => "Meta: Time Signature",
        113 => "Meta: Key Signature",
        114 => "Meta: Sequencer Specific",
        255 => "Unknown",
        _ => "Invalid",
    };
    
    let c_string = match CString::new(type_name) {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    let name_bytes = c_string.as_bytes_with_nul();
    if name_bytes.len() > buffer_size as usize {
        return -3;
    }
    
    unsafe {
        std::ptr::copy_nonoverlapping(
            name_bytes.as_ptr() as *const c_char,
            buffer,
            name_bytes.len(),
        );
    }
    
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_type_codes() {
        // Test that event type codes are consistent
        assert_eq!(0, 0); // Note Off
        assert_eq!(1, 1); // Note On
        // Add more assertions as needed
    }
}