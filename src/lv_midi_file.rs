// Test program for MIDI file functionality
// Run with: cargo run --bin midi_file_test -- path/to/file.mid

use std::env;

// Import the Rust functions directly from our library
use crate::{
    midi_file::{load_midi_file, get_midi_file, close_midi_file, EventType},
    get_note_name,
};

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

    // Load the MIDI file using Rust function
    let file_handle = match load_midi_file(file_path) {
        Ok(handle) => {
            println!("✅ Successfully opened MIDI file with handle: {}", handle);
            handle
        }
        Err(e) => {
            println!("❌ Failed to open MIDI file: {}", e);
            return;
        }
    };

    // Get file information using Rust functions
    let files_guard = match get_midi_file(file_handle) {
        Some(guard) => guard,
        None => {
            println!("❌ Failed to get file reference");
            return;
        }
    };

    let midi_file = match files_guard.get(&file_handle) {
        Some(file) => file,
        None => {
            println!("❌ File handle not found");
            return;
        }
    };

    println!("\n📋 MIDI File Information:");
    println!("   Format: {}", midi_file.format);
    println!("   Tracks: {}", midi_file.tracks.len());
    println!("   Duration: {} ticks", midi_file.get_duration_ticks());
    
    match midi_file.timing {
        midly::Timing::Metrical(tpq) => {
            println!("   Timing: {} ticks per quarter note", tpq.as_int());
            let duration_ms = midi_file.ticks_to_ms(midi_file.get_duration_ticks(), 500000); // 120 BPM
            println!("   Duration: {:.2} seconds (at 120 BPM)", duration_ms / 1000.0);
        }
        midly::Timing::Timecode(fps, tpf) => {
            println!("   Timing: {:.2} FPS, {} ticks per frame", fps.as_f32(), tpf);
        }
    }

    // Analyze each track
    for (track_idx, track) in midi_file.tracks.iter().enumerate() {
        println!("\n🎼 Track {} Analysis:", track_idx + 1);
        
        println!("   Events: {}", track.events.len());
        println!("   Channels used: {:016b}", track.channel_mask);
        
        // Show track name if available
        if !track.name.is_empty() {
            println!("   Name: \"{}\"", track.name);
        }
        
        // Show instrument name if available
        if let Some(ref instrument) = track.instrument {
            println!("   Instrument: \"{}\"", instrument);
        }
        
        // Show first few events
        let events_to_show = std::cmp::min(10, track.events.len());
        if events_to_show > 0 {
            println!("   First {} events:", events_to_show);
            
            for (event_idx, abs_event) in track.events.iter().take(events_to_show).enumerate() {
                // Calculate UID as it would be generated in the C API
                let uid = generate_event_uid(file_handle, track_idx as i32, event_idx as i32);
                
                let type_name = match abs_event.event_type {
                    EventType::NoteOff => "Note Off",
                    EventType::NoteOn => "Note On", 
                    EventType::PolyphonicAftertouch => "Polyphonic Aftertouch",
                    EventType::ControlChange => "Control Change",
                    EventType::ProgramChange => "Program Change",
                    EventType::ChannelAftertouch => "Channel Aftertouch",
                    EventType::PitchBend => "Pitch Bend",
                    EventType::SystemExclusive => "System Exclusive",
                    EventType::MetaSequenceNumber => "Meta: Sequence Number",
                    EventType::MetaText => "Meta: Text",
                    EventType::MetaCopyright => "Meta: Copyright",
                    EventType::MetaTrackName => "Meta: Track Name",
                    EventType::MetaInstrumentName => "Meta: Instrument Name",
                    EventType::MetaLyric => "Meta: Lyric",
                    EventType::MetaMarker => "Meta: Marker",
                    EventType::MetaCuePoint => "Meta: Cue Point",
                    EventType::MetaChannelPrefix => "Meta: Channel Prefix",
                    EventType::MetaEndOfTrack => "Meta: End of Track",
                    EventType::MetaSetTempo => "Meta: Set Tempo",
                    EventType::MetaSmpteOffset => "Meta: SMPTE Offset",
                    EventType::MetaTimeSignature => "Meta: Time Signature",
                    EventType::MetaKeySignature => "Meta: Key Signature",
                    EventType::MetaSequencerSpecific => "Meta: Sequencer Specific",
                    EventType::Unknown => "Unknown",
                };
                
                print!("     [{}] UID:{:08X} T:{:6} {} Ch:{} ", 
                       event_idx, uid, abs_event.absolute_time, type_name, abs_event.channel + 1);
                
                // Show relevant data based on event type
                match abs_event.event_type {
                    EventType::NoteOff | EventType::NoteOn => {
                        let note_name = get_note_name(abs_event.data1);
                        println!("Note:{} ({}) Vel:{}", abs_event.data1, note_name, abs_event.data2);
                    }
                    EventType::ControlChange => {
                        let controller_name = get_controller_name(abs_event.data1);
                        println!("CC:{} ({}) Val:{}", abs_event.data1, controller_name, abs_event.data2);
                    }
                    EventType::ProgramChange => {
                        println!("Program:{}", abs_event.data1);
                    }
                    EventType::PitchBend => {
                        let bend_value = ((abs_event.data2 as u16) << 7) | (abs_event.data1 as u16);
                        println!("Bend:{} (center=8192)", bend_value);
                    }
                    _ => {
                        println!("Data1:{} Data2:{}", abs_event.data1, abs_event.data2);
                    }
                }
                
                // Show text if available
                if !abs_event.text.is_empty() {
                    println!("       Text: \"{}\"", abs_event.text);
                }
            }
            
            if track.events.len() > events_to_show {
                println!("     ... and {} more events", track.events.len() - events_to_show);
            }
        }
    }

    // Test UID functionality
    println!("\n🔧 Testing UID Functionality:");
    if !midi_file.tracks.is_empty() && !midi_file.tracks[0].events.is_empty() {
        let test_uid = generate_event_uid(file_handle, 0, 0);
        println!("   Generated UID for first event: 0x{:08X}", test_uid);
        
        // Decode the UID
        let decoded_file = ((test_uid >> 24) & 0xFF) as i32;
        let decoded_track = ((test_uid >> 16) & 0xFF) as i32;
        let decoded_event = (test_uid & 0xFFFF) as i32;
        
        println!("   Decoded UID: file={}, track={}, event={}", 
                decoded_file, decoded_track, decoded_event);
        
        // Verify it matches
        if decoded_file == file_handle && decoded_track == 0 && decoded_event == 0 {
            println!("   ✅ UID encoding/decoding works correctly!");
        } else {
            println!("   ❌ UID encoding/decoding failed!");
        }
    }

    // Close the file
    drop(files_guard); // Release the lock before closing
    if close_midi_file(file_handle) {
        println!("\n✅ Successfully closed MIDI file");
    } else {
        println!("\n❌ Failed to close MIDI file");
    }

    println!("\n🎼 MIDI File Analysis Complete! 🎼");
}

/// Generate UID using the same algorithm as the C API
fn generate_event_uid(file_handle: i32, track_index: i32, event_index: i32) -> u32 {
    let file_part = ((file_handle as u32) & 0xFF) << 24;
    let track_part = ((track_index as u32) & 0xFF) << 16;
    let event_part = (event_index as u32) & 0xFFFF;
    
    file_part | track_part | event_part
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
