// Simple test program for MIDI file functionality
// Run with: cargo run --bin simple_midi_test -- path/to/file.mid

use std::env;
use std::fs;

fn main() {
    println!("🎵 Simple MIDI File Test 🎵");
    println!("============================");

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <midi-file-path>", args[0]);
        println!("Example: {} test.mid", args[0]);
        return;
    }

    let file_path = &args[1];
    println!("Loading MIDI file: {}", file_path);

    // Try to read the file
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            println!("❌ Error reading file: {}", e);
            return;
        }
    };

    println!("✅ File read successfully, {} bytes", data.len());

    // Try to parse with midly
    match midly::Smf::parse(&data) {
        Ok(smf) => {
            println!("✅ MIDI file parsed successfully!");
            
            let format = match smf.header.format {
                midly::Format::SingleTrack => 0,
                midly::Format::Parallel => 1,
                midly::Format::Sequential => 2,
            };
            
            println!("📋 File Information:");
            println!("   Format: {}", format);
            println!("   Tracks: {}", smf.tracks.len());
            
            match smf.header.timing {
                midly::Timing::Metrical(tpq) => {
                    println!("   Timing: {} ticks per quarter note", tpq.as_int());
                }
                midly::Timing::Timecode(fps, tpf) => {
                    println!("   Timing: {} FPS, {} ticks per frame", fps.as_f32(), tpf);
                }
            }

            // Analyze each track
            for (track_idx, track) in smf.tracks.iter().enumerate() {
                println!("\n🎼 Track {}: {} events", track_idx + 1, track.len());
                
                let mut absolute_time = 0u32;
                let mut note_events = 0;
                let mut meta_events = 0;
                let mut other_events = 0;
                
                for event in track.iter().take(10) { // Show first 10 events
                    absolute_time = absolute_time.saturating_add(event.delta.as_int());
                    
                    match &event.kind {
                        midly::TrackEventKind::Midi { channel, message } => {
                            note_events += 1;
                            match message {
                                midly::MidiMessage::NoteOn { key, vel } => {
                                    if *vel > 0 {
                                        println!("   [{}] Note ON  - Ch:{} Note:{} Vel:{}", 
                                                absolute_time, channel.as_int() + 1, key.as_int(), vel.as_int());
                                    } else {
                                        println!("   [{}] Note OFF - Ch:{} Note:{}", 
                                                absolute_time, channel.as_int() + 1, key.as_int());
                                    }
                                }
                                midly::MidiMessage::NoteOff { key, vel } => {
                                    println!("   [{}] Note OFF - Ch:{} Note:{} Vel:{}", 
                                            absolute_time, channel.as_int() + 1, key.as_int(), vel.as_int());
                                }
                                midly::MidiMessage::Controller { controller, value } => {
                                    println!("   [{}] Control - Ch:{} CC:{} Val:{}", 
                                            absolute_time, channel.as_int() + 1, controller.as_int(), value.as_int());
                                }
                                midly::MidiMessage::ProgramChange { program } => {
                                    println!("   [{}] Program - Ch:{} Prog:{}", 
                                            absolute_time, channel.as_int() + 1, program.as_int());
                                }
                                _ => {
                                    other_events += 1;
                                    println!("   [{}] MIDI Event - Ch:{}", absolute_time, channel.as_int() + 1);
                                }
                            }
                        }
                        midly::TrackEventKind::Meta(meta) => {
                            meta_events += 1;
                            match meta {
                                midly::MetaMessage::TrackName(name) => {
                                    println!("   [{}] Track Name: \"{}\"", 
                                            absolute_time, String::from_utf8_lossy(name));
                                }
                                midly::MetaMessage::InstrumentName(name) => {
                                    println!("   [{}] Instrument: \"{}\"", 
                                            absolute_time, String::from_utf8_lossy(name));
                                }
                                midly::MetaMessage::Tempo(tempo) => {
                                    println!("   [{}] Tempo: {} μs/quarter", 
                                            absolute_time, tempo.as_int());
                                }
                                midly::MetaMessage::TimeSignature(num, den, clocks, _) => {
                                    println!("   [{}] Time Sig: {}/{}", 
                                            absolute_time, num, 1 << den);
                                }
                                midly::MetaMessage::KeySignature(key, minor) => {
                                    println!("   [{}] Key Sig: {} {}", 
                                            absolute_time, key, if *minor { "minor" } else { "major" });
                                }
                                midly::MetaMessage::EndOfTrack => {
                                    println!("   [{}] End of Track", absolute_time);
                                }
                                _ => {
                                    println!("   [{}] Meta Event", absolute_time);
                                }
                            }
                        }
                        _ => {
                            other_events += 1;
                            println!("   [{}] Other Event", absolute_time);
                        }
                    }
                }
                
                if track.len() > 10 {
                    println!("   ... and {} more events", track.len() - 10);
                }
                
                println!("   Summary: {} note/MIDI events, {} meta events, {} other events", 
                        note_events, meta_events, other_events);
            }
        }
        Err(e) => {
            println!("❌ Error parsing MIDI file: {}", e);
        }
    }

    println!("\n🎼 Test Complete! 🎼");
}