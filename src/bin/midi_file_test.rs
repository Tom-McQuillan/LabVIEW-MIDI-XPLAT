// Test program for MIDI file functionality
// Run with: cargo run --bin midi_file_test -- path/to/file.mid

use std::env;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};

// Import the functions from our library
extern "C" {
    fn midi_file_open(file_path: *const c_char, file_handle: *mut c_int) -> c_int;
    fn midi_file_close(file_handle: c_int) -> c_int;
    fn midi_file_get_info(file_handle: c_int, info: *mut MidiFileInfo) -> c_int;
    fn midi_file_get_track_info(file_handle: c_int, track_index: c_int, info: *mut TrackInfo) -> c_int;
    fn midi_file_get_track_name(file_handle: c_int, track_index: c_int, buffer: *mut c_char, buffer_size: c_int) -> c_int;
    fn midi_file_get_track_instrument(file_handle: c_int, track_index: c_int, buffer: *mut c_char, buffer_size: c_int) -> c_int;
    fn midi_file_get_event_count(file_handle: c_int, track_index: c_int) -> c_int;
    fn midi_file_get_event(file_handle: c_int, track_index: c_int, event_index: c_int, event: *mut MidiFileEvent) -> c_int;
    fn midi_file_get_event_text(file_handle: c_int, track_index: c_int, event_index: c_int, buffer: *mut c_char, buffer_size: c_int) -> c_int;
    fn midi_file_ticks_to_ms(file_handle: c_int, ticks: u32, tempo_us_per_quarter: u32) -> f64;
    fn midi_file_get_event_type_name(event_type: c_int, buffer: *mut c_char, buffer_size: c_int) -> c_int;
}

// Replicate the structures from our FFI module
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MidiFileInfo {
    pub format: c_int,
    pub track_count: c_int,
    pub timing_type: c_int,
    pub ticks_per_quarter: c_int,
    pub fps: f32,
    pub ticks_per_frame: c_int,
    pub duration_ticks: u32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub track_index: c_int,
    pub event_count: c_int,
    pub channel_mask: c_int,
    pub has_name: c_int,
    pub has_instrument: c_int,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MidiFileEvent {
    pub absolute_time: u32,
    pub event_type: c_int,
    pub channel: u8,
    pub data1: u8,
    pub data2: u8,
    pub has_text: c_int,
}

fn main() {
    println!("🎵 MIDI File Test Program 🎵");
    println!("=============================");

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <midi-file-path>", args[0]);
        println!("Example: {} test.mid", args[0]);
        return;
    }

    let file_path = &args[1];
    println!("Loading MIDI file: {}", file_path);

    // Convert path to C string
    let c_path = match CString::new(file_path.as_str()) {
        Ok(path) => path,
        Err(e) => {
            println!("❌ Error converting path to C string: {}", e);
            return;
        }
    };

    // Open the MIDI file
    let mut file_handle: c_int = 0;
    let result = unsafe {
        midi_file_open(c_path.as_ptr(), &mut file_handle)
    };

    if result != 0 {
        println!("❌ Failed to open MIDI file. Error code: {}", result);
        match result {
            -1 => println!("   Null pointer error"),
            -2 => println!("   Invalid UTF-8 in file path"),
            -3 => println!("   File loading error (file not found or invalid MIDI file)"),
            _ => println!("   Unknown error"),
        }
        return;
    }

    println!("✅ Successfully opened MIDI file with handle: {}", file_handle);

    // Get file information
    let mut file_info = MidiFileInfo {
        format: 0,
        track_count: 0,
        timing_type: 0,
        ticks_per_quarter: 0,
        fps: 0.0,
        ticks_per_frame: 0,
        duration_ticks: 0,
    };

    let result = unsafe {
        midi_file_get_info(file_handle, &mut file_info)
    };

    if result != 0 {
        println!("❌ Failed to get file info. Error code: {}", result);
    } else {
        println!("\n📋 MIDI File Information:");
        println!("   Format: {}", file_info.format);
        println!("   Tracks: {}", file_info.track_count);
        println!("   Duration: {} ticks", file_info.duration_ticks);
        
        if file_info.timing_type == 0 {
            println!("   Timing: {} ticks per quarter note", file_info.ticks_per_quarter);
            let duration_ms = unsafe {
                midi_file_ticks_to_ms(file_handle, file_info.duration_ticks, 500000) // 120 BPM
            };
            println!("   Duration: {:.2} seconds (at 120 BPM)", duration_ms / 1000.0);
        } else {
            println!("   Timing: {:.2} FPS, {} ticks per frame", file_info.fps, file_info.ticks_per_frame);
        }
    }

    // Analyze each track
    for track_idx in 0..file_info.track_count {
        println!("\n🎼 Track {} Analysis:", track_idx + 1);
        
        // Get track info
        let mut track_info = TrackInfo {
            track_index: 0,
            event_count: 0,
            channel_mask: 0,
            has_name: 0,
            has_instrument: 0,
        };
        
        let result = unsafe {
            midi_file_get_track_info(file_handle, track_idx, &mut track_info)
        };
        
        if result != 0 {
            println!("   ❌ Failed to get track info. Error code: {}", result);
            continue;
        }
        
        println!("   Events: {}", track_info.event_count);
        println!("   Channels used: {:016b}", track_info.channel_mask);
        
        // Get track name if available
        if track_info.has_name != 0 {
            let mut name_buffer = vec![0u8; 256];
            let result = unsafe {
                midi_file_get_track_name(
                    file_handle,
                    track_idx,
                    name_buffer.as_mut_ptr() as *mut c_char,
                    256,
                )
            };
            
            if result == 0 {
                if let Ok(name) = std::ffi::CStr::from_bytes_until_nul(&name_buffer) {
                    if let Ok(name_str) = name.to_str() {
                        println!("   Name: \"{}\"", name_str);
                    }
                }
            }
        }
        
        // Get instrument name if available
        if track_info.has_instrument != 0 {
            let mut instr_buffer = vec![0u8; 256];
            let result = unsafe {
                midi_file_get_track_instrument(
                    file_handle,
                    track_idx,
                    instr_buffer.as_mut_ptr() as *mut c_char,
                    256,
                )
            };
            
            if result == 0 {
                if let Ok(instr) = std::ffi::CStr::from_bytes_until_nul(&instr_buffer) {
                    if let Ok(instr_str) = instr.to_str() {
                        println!("   Instrument: \"{}\"", instr_str);
                    }
                }
            }
        }
        
        // Show first few events
        let events_to_show = std::cmp::min(10, track_info.event_count);
        if events_to_show > 0 {
            println!("   First {} events:", events_to_show);
            
            for event_idx in 0..events_to_show {
                let mut event = MidiFileEvent {
                    absolute_time: 0,
                    event_type: 0,
                    channel: 0,
                    data1: 0,
                    data2: 0,
                    has_text: 0,
                };
                
                let result = unsafe {
                    midi_file_get_event(file_handle, track_idx, event_idx, &mut event)
                };
                
                if result != 0 {
                    println!("     ❌ Failed to get event {}. Error code: {}", event_idx, result);
                    continue;
                }
                
                // Get event type name
                let mut type_name_buffer = vec![0u8; 64];
                let _result = unsafe {
                    midi_file_get_event_type_name(
                        event.event_type,
                        type_name_buffer.as_mut_ptr() as *mut c_char,
                        64,
                    )
                };
                
                let type_name = std::ffi::CStr::from_bytes_until_nul(&type_name_buffer)
                    .ok()
                    .and_then(|c| c.to_str().ok())
                    .unwrap_or("Unknown");
                
                print!("     [{}] T:{:6} {} Ch:{} ", 
                       event_idx, event.absolute_time, type_name, event.channel + 1);
                
                // Show relevant data based on event type
                match event.event_type {
                    0 | 1 => { // Note Off/On
                        let note_name = get_note_name(event.data1);
                        println!("Note:{} ({}) Vel:{}", event.data1, note_name, event.data2);
                    }
                    3 => { // Control Change
                        let controller_name = get_controller_name(event.data1);
                        println!("CC:{} ({}) Val:{}", event.data1, controller_name, event.data2);
                    }
                    4 => { // Program Change
                        println!("Program:{}", event.data1);
                    }
                    6 => { // Pitch Bend
                        let bend_value = ((event.data2 as u16) << 7) | (event.data1 as u16);
                        println!("Bend:{} (center=8192)", bend_value);
                    }
                    _ => {
                        println!("Data1:{} Data2:{}", event.data1, event.data2);
                    }
                }
                
                // Show text if available
                if event.has_text != 0 {
                    let mut text_buffer = vec![0u8; 256];
                    let result = unsafe {
                        midi_file_get_event_text(
                            file_handle,
                            track_idx,
                            event_idx,
                            text_buffer.as_mut_ptr() as *mut c_char,
                            256,
                        )
                    };
                    
                    if result == 0 {
                        if let Ok(text) = std::ffi::CStr::from_bytes_until_nul(&text_buffer) {
                            if let Ok(text_str) = text.to_str() {
                                println!("       Text: \"{}\"", text_str);
                            }
                        }
                    }
                }
            }
            
            if track_info.event_count > events_to_show {
                println!("     ... and {} more events", track_info.event_count - events_to_show);
            }
        }
    }

    // Close the file
    let result = unsafe {
        midi_file_close(file_handle)
    };

    if result == 0 {
        println!("\n✅ Successfully closed MIDI file");
    } else {
        println!("\n❌ Failed to close MIDI file. Error code: {}", result);
    }

    println!("\n🎼 MIDI File Analysis Complete! 🎼");
}

// Helper function to convert MIDI note number to note name
fn get_note_name(note: u8) -> String {
    let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i32 - 1; // MIDI note 60 = C4
    let note_index = (note % 12) as usize;
    format!("{}{}", notes[note_index], octave)
}

// Helper function to get controller names
fn get_controller_name(controller: u8) -> &'static str {
    match controller {
        1 => "Modulation",
        7 => "Volume",
        10 => "Pan",
        11 => "Expression",
        64 => "Sustain",
        65 => "Portamento",
        66 => "Sostenuto",
        67 => "Soft Pedal",
        _ => "Other"
    }
}