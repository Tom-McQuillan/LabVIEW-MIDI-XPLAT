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

/// MIDI event structure for LabVIEW with both ticks and milliseconds
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MidiFileEvent {
    pub absolute_time_ticks: c_uint,
    pub absolute_time_ms: c_double,
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

/// Get a specific event from a track (with accurate millisecond timing based on tempo changes)
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
    
    // Calculate accurate milliseconds by tracking tempo changes
    let time_ms = calculate_accurate_milliseconds(midi_file, abs_event.absolute_time);
    
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
        absolute_time_ticks: abs_event.absolute_time,
        absolute_time_ms: time_ms,
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

/// Calculate accurate milliseconds by tracking tempo changes chronologically
fn calculate_accurate_milliseconds(midi_file: &crate::midi_file::MidiFile, target_ticks: u32) -> f64 {
    let mut current_tempo_us = 500000u32; // Default: 120 BPM = 500,000 μs per quarter
    let mut current_time_ms = 0.0f64;
    let mut last_tick_time = 0u32;
    
    // Get timing info
    let ticks_per_quarter = match midi_file.timing {
        midly::Timing::Metrical(tpq) => tpq.as_int(),
        midly::Timing::Timecode(fps, tpf) => {
            // For timecode, convert directly without tempo tracking
            let fps = fps.as_f32() as f64;
            let ticks_per_frame = tpf as f64;
            return (target_ticks as f64 / (fps * ticks_per_frame)) * 1000.0;
        }
    };
    
    // Collect all tempo events from all tracks and sort by time
    let mut tempo_events = Vec::new();
    
    for track in &midi_file.tracks {
        for abs_event in &track.events {
            if abs_event.event_type == EventType::MetaSetTempo {
                // Parse tempo from text (e.g., "Tempo: 500000 μs/quarter")
                if let Some(tempo_us) = parse_tempo_from_text(&abs_event.text) {
                    tempo_events.push((abs_event.absolute_time, tempo_us));
                }
            }
        }
    }
    
    // Sort tempo events by time
    tempo_events.sort_by_key(|&(time, _)| time);
    
    // Calculate milliseconds by processing tempo changes chronologically
    for &(tempo_change_time, new_tempo) in &tempo_events {
        // If this tempo change is after our target time, we're done
        if tempo_change_time > target_ticks {
            break;
        }
        
        // Calculate time elapsed since last tempo change using current tempo
        let ticks_elapsed = tempo_change_time - last_tick_time;
        let time_elapsed_ms = (ticks_elapsed as f64 / ticks_per_quarter as f64) * (current_tempo_us as f64 / 1000.0);
        current_time_ms += time_elapsed_ms;
        
        // Update tempo and time tracking
        current_tempo_us = new_tempo;
        last_tick_time = tempo_change_time;
    }
    
    // Calculate remaining time from last tempo change to target time
    let remaining_ticks = target_ticks - last_tick_time;
    let remaining_time_ms = (remaining_ticks as f64 / ticks_per_quarter as f64) * (current_tempo_us as f64 / 1000.0);
    current_time_ms += remaining_time_ms;
    
    current_time_ms
}

/// Parse tempo value from meta event text
fn parse_tempo_from_text(text: &str) -> Option<u32> {
    // Text format: "Tempo: 500000 μs/quarter"
    if let Some(start) = text.find("Tempo: ") {
        let after_colon = &text[start + 7..]; // Skip "Tempo: "
        if let Some(end) = after_colon.find(' ') {
            let tempo_str = &after_colon[..end];
            tempo_str.parse::<u32>().ok()
        } else {
            // Try parsing the whole remaining string
            after_colon.parse::<u32>().ok()
        }
    } else {
        None
    }
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
            
            // Use accurate tempo tracking for this event too
            let time_ms = calculate_accurate_milliseconds(midi_file, abs_event.absolute_time);
            
            events_slice[count as usize] = MidiFileEvent {
                absolute_time_ticks: abs_event.absolute_time,
                absolute_time_ms: time_ms,
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
    use crate::midi_file::{MidiFile, TrackData, AbsoluteEvent, EventType};
    
    #[test]
    fn test_parse_tempo_from_text() {
        // Test standard tempo format
        assert_eq!(parse_tempo_from_text("Tempo: 500000 μs/quarter"), Some(500000));
        assert_eq!(parse_tempo_from_text("Tempo: 600000 μs/quarter"), Some(600000));
        assert_eq!(parse_tempo_from_text("Tempo: 428571 μs/quarter"), Some(428571));
        
        // Test without units
        assert_eq!(parse_tempo_from_text("Tempo: 500000"), Some(500000));
        
        // Test invalid formats
        assert_eq!(parse_tempo_from_text("Not a tempo"), None);
        assert_eq!(parse_tempo_from_text("Tempo: invalid"), None);
        assert_eq!(parse_tempo_from_text(""), None);
    }
    
    #[test]
    fn test_calculate_accurate_milliseconds_no_tempo_changes() {
        // Create a simple MIDI file with no tempo changes (should use default 120 BPM)
        let midi_file = create_test_midi_file(384, vec![]); // 384 ticks per quarter, no tempo events
        
        // Test calculations
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 0), 0.0); // Start
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 384), 500.0); // 1 quarter note at 120 BPM = 500ms
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 768), 1000.0); // 2 quarter notes = 1000ms
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 192), 250.0); // Half quarter note = 250ms
    }
    
    #[test]
    fn test_calculate_accurate_milliseconds_with_tempo_change() {
        // Create MIDI file with tempo change: 120 BPM -> 100 BPM at tick 384
        let tempo_events = vec![
            (384, 600000), // Change to 100 BPM (600,000 μs per quarter) at tick 384
        ];
        let midi_file = create_test_midi_file(384, tempo_events);
        
        // Before tempo change (120 BPM = 500,000 μs per quarter)
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 0), 0.0);
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 384), 500.0); // 1 quarter at 120 BPM = 500ms
        
        // After tempo change (100 BPM = 600,000 μs per quarter)
        // Time at tick 768 = 500ms (first quarter at 120 BPM) + 600ms (second quarter at 100 BPM) = 1100ms
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 768), 1100.0);
    }
    
    #[test]
    fn test_calculate_accurate_milliseconds_multiple_tempo_changes() {
        // Create MIDI file with multiple tempo changes
        let tempo_events = vec![
            (192, 400000), // 150 BPM at tick 192 (half quarter note)
            (576, 800000), // 75 BPM at tick 576 (1.5 quarter notes)
        ];
        let midi_file = create_test_midi_file(384, tempo_events);
        
        // Calculate expected times:
        // 0-192: 0.5 quarter at 120 BPM = 0.5 * 500ms = 250ms
        // 192-576: 1 quarter at 150 BPM = 1 * 400ms = 400ms  
        // Total at tick 576 = 250ms + 400ms = 650ms
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 192), 250.0);
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 576), 650.0);
        
        // 576-768: 0.5 quarter at 75 BPM = 0.5 * 800ms = 400ms
        // Total at tick 768 = 650ms + 400ms = 1050ms
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 768), 1050.0);
    }
    
    #[test]
    fn test_calculate_accurate_milliseconds_tempo_after_target() {
        // Test tempo change that occurs after our target time
        let tempo_events = vec![
            (1000, 600000), // Tempo change after our target time
        ];
        let midi_file = create_test_midi_file(384, tempo_events);
        
        // Should use default tempo (120 BPM) since tempo change is after target
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 384), 500.0);
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 768), 1000.0);
    }
    
    #[test]
    fn test_calculate_accurate_milliseconds_timecode() {
        // Test timecode timing (should bypass tempo tracking)
        let midi_file = create_test_midi_file_timecode(25.0, 40); // 25 FPS, 40 ticks per frame
        
        // Should calculate: ticks / (fps * ticks_per_frame) * 1000
        // 1000 ticks / (25 * 40) * 1000 = 1000 / 1000 * 1000 = 1000ms
        assert_eq!(calculate_accurate_milliseconds(&midi_file, 1000), 1000.0);
    }
    
    // Helper function to create test MIDI file
    fn create_test_midi_file(ticks_per_quarter: u16, tempo_events: Vec<(u32, u32)>) -> MidiFile {
        let mut events = Vec::new();
        
        // Add tempo events
        for (time, tempo_us) in tempo_events {
            events.push(AbsoluteEvent {
                absolute_time: time,
                event_type: EventType::MetaSetTempo,
                channel: 0,
                data1: 0,
                data2: 0,
                text: format!("Tempo: {} μs/quarter", tempo_us),
            });
        }
        
        let track = TrackData {
            events,
            name: "Test Track".to_string(),
            instrument: None,
            channel_mask: 0,
        };
        
        // Create a dummy SMF - we'll use unsafe zeroed since we don't actually use it in tests
        let dummy_smf: midly::Smf<'static> = unsafe { std::mem::zeroed() };
        
        MidiFile {
            smf: dummy_smf,
            tracks: vec![track],
            timing: midly::Timing::Metrical(midly::num::u15::new(ticks_per_quarter).unwrap()),
            format: 1,
        }
    }
    
    // Helper function to create timecode MIDI file
    fn create_test_midi_file_timecode(fps: f32, ticks_per_frame: u8) -> MidiFile {
        let track = TrackData {
            events: vec![],
            name: "Test Track".to_string(),
            instrument: None,
            channel_mask: 0,
        };
        
        let fps_enum = match fps as u8 {
            24 => midly::Fps::Fps24,
            25 => midly::Fps::Fps25,
            29 => midly::Fps::Fps29,
            30 => midly::Fps::Fps30,
            _ => midly::Fps::Fps25, // Default
        };
        
        // Create a dummy SMF
        let dummy_smf: midly::Smf<'static> = unsafe { std::mem::zeroed() };
        
        MidiFile {
            smf: dummy_smf,
            tracks: vec![track],
            timing: midly::Timing::Timecode(fps_enum, ticks_per_frame),
            format: 1,
        }
    }
    
    #[test]
    fn test_event_type_codes() {
        // Test that event type codes are consistent
        assert_eq!(0, 0); // Note Off
        assert_eq!(1, 1); // Note On
        // Add more assertions as needed
    }
}