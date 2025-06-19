#![allow(non_snake_case)]

mod midi;
mod lv_midi;
mod labview_interop;
mod user_event_test;

// Re-export LabVIEW MIDI functions publicly so the test binary can use them
pub use lv_midi::*;
pub use user_event_test::*;

// Helper function to convert MIDI note number to note name
pub fn get_note_name(note: u8) -> String {
    let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i32 - 1; // MIDI note 60 = C4
    let note_index = (note % 12) as usize;
    format!("{}{}", notes[note_index], octave)
}

// Helper function to get control change names
pub fn get_control_name(controller: u8) -> &'static str {
    match controller {
        1 => "Modulation",
        7 => "Volume",
        10 => "Pan",
        11 => "Expression",
        64 => "Sustain Pedal",
        65 => "Portamento",
        66 => "Sostenuto",
        67 => "Soft Pedal",
        _ => "Other"
    }
}

// Keep the tests for development
#[cfg(test)]
mod tests {
    use crate::midi::MidiManager;
    use crate::{get_note_name, get_control_name}; // Import the helper functions
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_midi_devices() {
        let manager = MidiManager::new();
        
        println!("=== MIDI Input Devices ===");
        match manager.list_input_devices() {
            Ok(devices) => {
                for (i, device) in devices.iter().enumerate() {
                    println!("{}: {}", i, device);
                }
                if devices.is_empty() {
                    println!("No MIDI input devices found");
                }
            }
            Err(e) => println!("Error listing input devices: {}", e),
        }

        println!("\n=== MIDI Output Devices ===");
        match manager.list_output_devices() {
            Ok(devices) => {
                for (i, device) in devices.iter().enumerate() {
                    println!("{}: {}", i, device);
                }
                if devices.is_empty() {
                    println!("No MIDI output devices found");
                }
            }
            Err(e) => println!("Error listing output devices: {}", e),
        }
    }

    #[test]
    #[ignore] // Run with: cargo test test_piano_listener -- --ignored --nocapture
    fn test_piano_listener() {
        let mut manager = MidiManager::new();
        
        // List available input devices first
        println!("Available MIDI input devices:");
        let input_devices = match manager.list_input_devices() {
            Ok(devices) => {
                for (i, device) in devices.iter().enumerate() {
                    println!("{}: {}", i, device);
                }
                devices
            }
            Err(e) => {
                println!("Error listing input devices: {}", e);
                return;
            }
        };

        if input_devices.is_empty() {
            println!("No MIDI input devices found! Make sure your piano is connected.");
            return;
        }

        // Connect to the first input device (usually your piano)
        println!("\nConnecting to: {}", input_devices[0]);
        if let Err(e) = manager.connect_input(0) {
            println!("Failed to connect to piano: {}", e);
            return;
        }

        println!("✅ Connected! Now play some keys on your piano...");
        println!("Listening for 30 seconds. Press Ctrl+C to stop early.\n");

        let start_time = std::time::Instant::now();
        let listen_duration = Duration::from_secs(30);

        while start_time.elapsed() < listen_duration {
            if let Some(message) = manager.receive_message() {
                // Filter out spam messages
                if message.len() == 1 {
                    match message[0] {
                        0xFE => continue, // Active Sensing - ignore
                        0xF8 => continue, // MIDI Clock - ignore
                        _ => {} // Process other single-byte messages
                    }
                }
                
                // Parse and display the MIDI message in a human-readable way
                if message.len() >= 3 {
                    let status = message[0];
                    let data1 = message[1];
                    let data2 = message[2];
                    
                    // Extract channel (lower 4 bits of status byte)
                    let channel = (status & 0x0F) + 1; // MIDI channels are 1-16, not 0-15
                    
                    match status & 0xF0 {
                        0x90 => {
                            // Note On
                            if data2 > 0 {
                                let note_name = get_note_name(data1);
                                println!("🎹 Note ON  - Channel: {}, Note: {} ({}), Velocity: {}", 
                                        channel, data1, note_name, data2);
                            } else {
                                // Note on with velocity 0 is actually note off
                                let note_name = get_note_name(data1);
                                println!("🎹 Note OFF - Channel: {}, Note: {} ({}) [via Note On velocity 0]", 
                                        channel, data1, note_name);
                            }
                        }
                        0x80 => {
                            // Note Off
                            let note_name = get_note_name(data1);
                            println!("🎹 Note OFF - Channel: {}, Note: {} ({}), Velocity: {}", 
                                    channel, data1, note_name, data2);
                        }
                        0xB0 => {
                            // Control Change (sustain pedal, modulation, etc.)
                            let control_name = get_control_name(data1);
                            println!("🎛️  Control  - Channel: {}, Controller: {} ({}), Value: {}", 
                                    channel, data1, control_name, data2);
                        }
                        0xE0 => {
                            // Pitch Bend
                            let bend_value = ((data2 as u16) << 7) | (data1 as u16);
                            println!("🎵 Pitch Bend - Channel: {}, Value: {} (center=8192)", 
                                    channel, bend_value);
                        }
                        0xC0 => {
                            // Program Change
                            println!("🎨 Program Change - Channel: {}, Program: {}", channel, data1);
                        }
                        _ => {
                            // Other MIDI message
                            println!("📡 MIDI Message - Raw: {:02X?}", message);
                        }
                    }
                } else {
                    // Short message or system message
                    println!("📡 System/Short Message - Raw: {:02X?}", message);
                }
            }
            
            // Small delay to prevent busy waiting
            thread::sleep(Duration::from_millis(10));
        }

        println!("\n🎼 Listening session complete!");
    }
}