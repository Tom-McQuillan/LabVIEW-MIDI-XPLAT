// Simple test program to check MIDI functionality
// Run with: cargo run --bin midi_test

use std::thread;
use std::time::Duration;

// Import midir directly since we can't easily import from our own lib
use midir::{MidiInput, MidiOutput};

fn main() {
    println!("🎵 MIDI Test Program 🎵");
    println!("======================");

    // Test device discovery using midir directly
    println!("\n1. Testing device discovery...");
    
    // Test input devices
    match MidiInput::new("Test MIDI Input") {
        Ok(midi_in) => {
            let ports = midi_in.ports();
            println!("   Input devices: {}", ports.len());
            
            println!("\n2. Available MIDI input devices:");
            for (i, port) in ports.iter().enumerate() {
                match midi_in.port_name(port) {
                    Ok(name) => println!("   [{}]: {}", i, name),
                    Err(_) => println!("   [{}]: <Unknown>", i),
                }
            }
            
            if ports.is_empty() {
                println!("❌ No MIDI input devices found!");
                println!("   Make sure your MIDI device is connected.");
            } else {
                println!("   ✅ Found {} MIDI input device(s)", ports.len());
                
                // Test connecting to the first device
                if let Some(first_port) = ports.first() {
                    println!("\n3. Testing connection to first device...");
                    
                    match midi_in.port_name(first_port) {
                        Ok(port_name) => {
                            println!("   Attempting to connect to: {}", port_name);
                            
                            // Create a receiver for MIDI messages
                            let (tx, rx) = std::sync::mpsc::channel();
                            
                            // Connect with a callback
                            match midi_in.connect(first_port, "test-connection", 
                                move |_timestamp, message, _| {
                                    let _ = tx.send(message.to_vec());
                                }, 
                                ()
                            ) {
                                Ok(_connection) => {
                                    println!("   ✅ Connected successfully!");
                                    println!("   🎹 Listening for MIDI messages for 10 seconds...");
                                    println!("   Play some notes or move controllers on your MIDI device!");
                                    
                                    let mut message_count = 0;
                                    let start_time = std::time::Instant::now();
                                    
                                    while start_time.elapsed() < Duration::from_secs(10) {
                                        if let Ok(message) = rx.try_recv() {
                                            message_count += 1;
                                            
                                            // Filter out common spam messages
                                            if message.len() == 1 {
                                                match message[0] {
                                                    0xFE => continue, // Active Sensing
                                                    0xF8 => continue, // MIDI Clock
                                                    _ => {}
                                                }
                                            }
                                            
                                            print!("   📨 MIDI Message #{}: ", message_count);
                                            
                                            // Parse and display the message
                                            if message.len() >= 3 {
                                                let status = message[0];
                                                let data1 = message[1];
                                                let data2 = message[2];
                                                let channel = (status & 0x0F) + 1;
                                                
                                                match status & 0xF0 {
                                                    0x90 => {
                                                        if data2 > 0 {
                                                            let note_name = get_note_name(data1);
                                                            println!("Note ON  - Channel: {}, Note: {} ({}), Velocity: {}", 
                                                                    channel, data1, note_name, data2);
                                                        } else {
                                                            let note_name = get_note_name(data1);
                                                            println!("Note OFF - Channel: {}, Note: {} ({})", 
                                                                    channel, data1, note_name);
                                                        }
                                                    }
                                                    0x80 => {
                                                        let note_name = get_note_name(data1);
                                                        println!("Note OFF - Channel: {}, Note: {} ({}), Velocity: {}", 
                                                                channel, data1, note_name, data2);
                                                    }
                                                    0xB0 => {
                                                        let control_name = get_control_name(data1);
                                                        println!("Control  - Channel: {}, Controller: {} ({}), Value: {}", 
                                                                channel, data1, control_name, data2);
                                                    }
                                                    0xE0 => {
                                                        let bend_value = ((data2 as u16) << 7) | (data1 as u16);
                                                        println!("Pitch Bend - Channel: {}, Value: {}", channel, bend_value);
                                                    }
                                                    0xC0 => {
                                                        println!("Program Change - Channel: {}, Program: {}", channel, data1);
                                                    }
                                                    _ => {
                                                        println!("Raw: {:02X?}", message);
                                                    }
                                                }
                                            } else {
                                                println!("Raw: {:02X?}", message);
                                            }
                                        }
                                        
                                        thread::sleep(Duration::from_millis(10));
                                    }
                                    
                                    if message_count > 0 {
                                        println!("   ✅ Received {} MIDI messages!", message_count);
                                    } else {
                                        println!("   ❌ No MIDI messages received.");
                                        println!("   Try playing notes or moving controllers on your MIDI device.");
                                    }
                                    
                                    // Connection automatically drops here
                                }
                                Err(e) => {
                                    println!("   ❌ Failed to connect: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("   ❌ Failed to get port name: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to create MIDI input: {}", e);
        }
    }
    
    // Test output devices
    println!("\n4. Testing MIDI output devices...");
    match MidiOutput::new("Test MIDI Output") {
        Ok(midi_out) => {
            let ports = midi_out.ports();
            println!("   Output devices: {}", ports.len());
            
            for (i, port) in ports.iter().enumerate() {
                match midi_out.port_name(port) {
                    Ok(name) => println!("   [{}]: {}", i, name),
                    Err(_) => println!("   [{}]: <Unknown>", i),
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to create MIDI output: {}", e);
        }
    }

    println!("\n🎵 Test Complete! 🎵");
    println!("This test uses midir directly to verify your MIDI setup works.");
    println!("If you see MIDI messages above, your library should work too!");
}

// Helper function to convert MIDI note number to note name
fn get_note_name(note: u8) -> String {
    let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i32 - 1; // MIDI note 60 = C4
    let note_index = (note % 12) as usize;
    format!("{}{}", notes[note_index], octave)
}

// Helper function to get control change names
fn get_control_name(controller: u8) -> &'static str {
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