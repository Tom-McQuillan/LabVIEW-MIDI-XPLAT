use midir::{MidiInput, MidiOutput, MidiInputConnection, MidiOutputConnection};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

pub struct MidiManager {
    input_connection: Option<MidiInputConnection<()>>,
    output_connection: Option<MidiOutputConnection>,
    message_receiver: Option<Receiver<Vec<u8>>>,
}

impl MidiManager {
    pub fn new() -> Self {
        MidiManager {
            input_connection: None,
            output_connection: None,
            message_receiver: None,
        }
    }

    // List all available MIDI input devices
    pub fn list_input_devices(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let midi_in = MidiInput::new("LabVIEW MIDI Input")?;
        let ports = midi_in.ports();
        let mut device_names = Vec::new();
        
        for port in &ports {
            if let Ok(name) = midi_in.port_name(port) {
                device_names.push(name);
            }
        }
        
        Ok(device_names)
    }

    // List all available MIDI output devices
    pub fn list_output_devices(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let midi_out = MidiOutput::new("LabVIEW MIDI Output")?;
        let ports = midi_out.ports();
        let mut device_names = Vec::new();
        
        for port in &ports {
            if let Ok(name) = midi_out.port_name(port) {
                device_names.push(name);
            }
        }
        
        Ok(device_names)
    }

    // Connect to a MIDI input device by index
    pub fn connect_input(&mut self, device_index: usize) -> Result<(), Box<dyn std::error::Error>> {
        let midi_in = MidiInput::new("LabVIEW MIDI Input")?;
        let ports = midi_in.ports();
        
        if device_index >= ports.len() {
            return Err("Device index out of range".into());
        }

        let port = &ports[device_index];
        let port_name = midi_in.port_name(port)?;
        
        // Create a channel to receive MIDI messages
        let (sender, receiver) = mpsc::channel();
        
        // Connect to the input port with a callback
        let connection = midi_in.connect(port, &port_name, 
            move |_timestamp, message, _| {
                // Send the MIDI message through the channel
                let _ = sender.send(message.to_vec());
            }, 
            ()
        )?;

        self.input_connection = Some(connection);
        self.message_receiver = Some(receiver);
        
        println!("Connected to MIDI input: {}", port_name);
        Ok(())
    }

    // Connect to a MIDI output device by index
    pub fn connect_output(&mut self, device_index: usize) -> Result<(), Box<dyn std::error::Error>> {
        let midi_out = MidiOutput::new("LabVIEW MIDI Output")?;
        let ports = midi_out.ports();
        
        if device_index >= ports.len() {
            return Err("Device index out of range".into());
        }

        let port = &ports[device_index];
        let port_name = midi_out.port_name(port)?;
        
        let connection = midi_out.connect(port, &port_name)?;
        self.output_connection = Some(connection);
        
        println!("Connected to MIDI output: {}", port_name);
        Ok(())
    }

    // Send a MIDI message
    pub fn send_message(&mut self, message: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut conn) = self.output_connection {
            conn.send(message)?;
            Ok(())
        } else {
            Err("No output device connected".into())
        }
    }

    // Check for incoming MIDI messages (non-blocking)
    pub fn receive_message(&self) -> Option<Vec<u8>> {
        if let Some(ref receiver) = self.message_receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }

    // Helper function to create common MIDI messages
    pub fn note_on(channel: u8, note: u8, velocity: u8) -> Vec<u8> {
        vec![0x90 | (channel & 0x0F), note & 0x7F, velocity & 0x7F]
    }

    pub fn note_off(channel: u8, note: u8, velocity: u8) -> Vec<u8> {
        vec![0x80 | (channel & 0x0F), note & 0x7F, velocity & 0x7F]
    }

    pub fn control_change(channel: u8, controller: u8, value: u8) -> Vec<u8> {
        vec![0xB0 | (channel & 0x0F), controller & 0x7F, value & 0x7F]
    }
}