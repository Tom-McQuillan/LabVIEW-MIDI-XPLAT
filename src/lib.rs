mod midi;

use midi::MidiManager;
use std::thread;
use std::time::Duration;

// This function demonstrates basic MIDI functionality
// You can test it with: cargo test -- --nocapture
#[cfg(test)]
mod tests {
    use super::*;

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
    #[ignore] // Run with: cargo test test_midi_io -- --ignored --nocapture
    fn test_midi_io() {
        let mut manager = MidiManager::new();
        
        // Try to connect to first available devices
        if let Ok(input_devices) = manager.list_input_devices() {
            if !input_devices.is_empty() {
                println!("Connecting to input device: {}", input_devices[0]);
                if let Err(e) = manager.connect_input(0) {
                    println!("Failed to connect input: {}", e);
                    return;
                }
            }
        }

        if let Ok(output_devices) = manager.list_output_devices() {
            if !output_devices.is_empty() {
                println!("Connecting to output device: {}", output_devices[0]);
                if let Err(e) = manager.connect_output(0) {
                    println!("Failed to connect output: {}", e);
                    return;
                }
            }
        }

        println!("Listening for MIDI messages for 10 seconds...");
        println!("Also sending test notes...");

        // Listen for messages and send test notes
        for i in 0..100 {
            // Check for incoming messages
            if let Some(message) = manager.receive_message() {
                println!("Received MIDI: {:?} (hex: {:02X?})", message, message);
            }

            // Send a test note every 20 iterations (about every 2 seconds)
            if i % 20 == 0 {
                let note = 60 + (i / 20) as u8; // C4, D4, E4, F4, G4
                let note_on = MidiManager::note_on(0, note, 100);
                
                if let Err(e) = manager.send_message(&note_on) {
                    println!("Failed to send note on: {}", e);
                } else {
                    println!("Sent Note On: Channel 0, Note {}, Velocity 100", note);
                }

                // Send note off after 500ms
                thread::sleep(Duration::from_millis(500));
                let note_off = MidiManager::note_off(0, note, 0);
                
                if let Err(e) = manager.send_message(&note_off) {
                    println!("Failed to send note off: {}", e);
                } else {
                    println!("Sent Note Off: Channel 0, Note {}", note);
                }
            }

            thread::sleep(Duration::from_millis(100));
        }

        println!("Test complete!");
    }
}

// Example function that could be called from LabVIEW (we'll expand this later)
pub fn get_midi_device_count() -> i32 {
    let manager = MidiManager::new();
    match manager.list_input_devices() {
        Ok(devices) => devices.len() as i32,
        Err(_) => -1,
    }
}