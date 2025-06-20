use midly::{Smf, Timing, TrackEventKind, MidiMessage, MetaMessage};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::path::Path;
use std::fs;

// Global storage for MIDI files (thread-safe)
static MIDI_FILES: OnceLock<Mutex<HashMap<i32, MidiFile>>> = OnceLock::new();
static NEXT_FILE_HANDLE: OnceLock<Mutex<i32>> = OnceLock::new();

fn get_midi_files() -> &'static Mutex<HashMap<i32, MidiFile>> {
    MIDI_FILES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_next_file_handle_mutex() -> &'static Mutex<i32> {
    NEXT_FILE_HANDLE.get_or_init(|| Mutex::new(1))
}

fn get_next_file_handle() -> i32 {
    let mut handle = get_next_file_handle_mutex().lock().unwrap();
    let current = *handle;
    *handle += 1;
    current
}

/// Represents a loaded MIDI file with processed track data
pub struct MidiFile {
    pub smf: Smf<'static>,
    pub tracks: Vec<TrackData>,
    pub timing: Timing,
    pub format: u16,
}

/// Processed track data with absolute timing
#[derive(Debug, Clone)]
pub struct TrackData {
    pub events: Vec<AbsoluteEvent>,
    pub name: String,
    pub instrument: Option<String>,
    pub channel_mask: u16, // Bitmask of channels used in this track
}

/// MIDI event with absolute timing
#[derive(Debug, Clone)]
pub struct AbsoluteEvent {
    pub absolute_time: u32,
    pub event_type: EventType,
    pub channel: u8,
    pub data1: u8,
    pub data2: u8,
    pub text: String, // For meta events
}

/// Event type enumeration for easier processing
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    NoteOff,
    NoteOn,
    PolyphonicAftertouch,
    ControlChange,
    ProgramChange,
    ChannelAftertouch,
    PitchBend,
    SystemExclusive,
    MetaSequenceNumber,
    MetaText,
    MetaCopyright,
    MetaTrackName,
    MetaInstrumentName,
    MetaLyric,
    MetaMarker,
    MetaCuePoint,
    MetaChannelPrefix,
    MetaEndOfTrack,
    MetaSetTempo,
    MetaSmpteOffset,
    MetaTimeSignature,
    MetaKeySignature,
    MetaSequencerSpecific,
    Unknown,
}

impl MidiFile {
    /// Create a new MidiFile from raw MIDI data
    pub fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse the MIDI file
        let smf = Smf::parse(data)?;
        
        // Convert to owned data
        let owned_smf = smf.make_static();
        
        let timing = owned_smf.header.timing;
        let format = match owned_smf.header.format {
            midly::Format::SingleTrack => 0,
            midly::Format::Parallel => 1,
            midly::Format::Sequential => 2,
        };
        
        // Process tracks
        let mut tracks = Vec::new();
        for (track_idx, track) in owned_smf.tracks.iter().enumerate() {
            let track_data = Self::process_track(track, track_idx, timing)?;
            tracks.push(track_data);
        }
        
        Ok(MidiFile {
            smf: owned_smf,
            tracks,
            timing,
            format,
        })
    }
    
    /// Process a single track to extract events with absolute timing
    fn process_track(
        track: &midly::Track<'_>,
        track_idx: usize,
        timing: Timing,
    ) -> Result<TrackData, Box<dyn std::error::Error>> {
        let mut events = Vec::new();
        let mut absolute_time = 0u32;
        let mut track_name = format!("Track {}", track_idx + 1);
        let mut instrument_name = None;
        let mut channel_mask = 0u16;
        
        for event in track.iter() {
            // Update absolute time
            absolute_time = absolute_time.saturating_add(event.delta.as_int());
            
            match &event.kind {
                TrackEventKind::Midi { channel, message } => {
                    // Set channel bit in mask
                    channel_mask |= 1 << channel.as_int();
                    
                    let (event_type, data1, data2) = match message {
                        MidiMessage::NoteOff { key, vel } => {
                            (EventType::NoteOff, key.as_int(), vel.as_int())
                        }
                        MidiMessage::NoteOn { key, vel } => {
                            if vel.as_int() == 0 {
                                (EventType::NoteOff, key.as_int(), vel.as_int())
                            } else {
                                (EventType::NoteOn, key.as_int(), vel.as_int())
                            }
                        }
                        MidiMessage::Aftertouch { key, vel } => {
                            (EventType::PolyphonicAftertouch, key.as_int(), vel.as_int())
                        }
                        MidiMessage::Controller { controller, value } => {
                            (EventType::ControlChange, controller.as_int(), value.as_int())
                        }
                        MidiMessage::ProgramChange { program } => {
                            (EventType::ProgramChange, program.as_int(), 0)
                        }
                        MidiMessage::ChannelAftertouch { vel } => {
                            (EventType::ChannelAftertouch, vel.as_int(), 0)
                        }
                        MidiMessage::PitchBend { bend } => {
                            let bend_value = bend.as_int();
                            (EventType::PitchBend, (bend_value & 0x7F) as u8, ((bend_value >> 7) & 0x7F) as u8)
                        }
                    };
                    
                    events.push(AbsoluteEvent {
                        absolute_time,
                        event_type,
                        channel: channel.as_int(),
                        data1,
                        data2,
                        text: String::new(),
                    });
                }
                TrackEventKind::SysEx(data) => {
                    events.push(AbsoluteEvent {
                        absolute_time,
                        event_type: EventType::SystemExclusive,
                        channel: 0,
                        data1: 0,
                        data2: 0,
                        text: format!("SysEx: {} bytes", data.len()),
                    });
                }
                TrackEventKind::Meta(meta) => {
                    let (event_type, text) = match meta {
                        MetaMessage::TrackName(name) => {
                            let name_str = String::from_utf8_lossy(name);
                            track_name = name_str.to_string();
                            (EventType::MetaTrackName, name_str.to_string())
                        }
                        MetaMessage::InstrumentName(name) => {
                            let name_str = String::from_utf8_lossy(name);
                            instrument_name = Some(name_str.to_string());
                            (EventType::MetaInstrumentName, name_str.to_string())
                        }
                        MetaMessage::Text(text) => {
                            (EventType::MetaText, String::from_utf8_lossy(text).to_string())
                        }
                        MetaMessage::Copyright(text) => {
                            (EventType::MetaCopyright, String::from_utf8_lossy(text).to_string())
                        }
                        MetaMessage::Lyric(text) => {
                            (EventType::MetaLyric, String::from_utf8_lossy(text).to_string())
                        }
                        MetaMessage::Marker(text) => {
                            (EventType::MetaMarker, String::from_utf8_lossy(text).to_string())
                        }
                        MetaMessage::CuePoint(text) => {
                            (EventType::MetaCuePoint, String::from_utf8_lossy(text).to_string())
                        }
                        MetaMessage::Tempo(tempo) => {
                            (EventType::MetaSetTempo, format!("Tempo: {} μs/quarter", tempo.as_int()))
                        }
                        MetaMessage::TimeSignature(numerator, denominator, clocks_per_click, _) => {
                            (EventType::MetaTimeSignature, 
                             format!("Time Sig: {}/{} ({})", numerator, 1 << denominator, clocks_per_click))
                        }
                        MetaMessage::KeySignature(key, is_minor) => {
                            (EventType::MetaKeySignature, 
                             format!("Key Sig: {} {}", key, if *is_minor { "minor" } else { "major" }))
                        }
                        MetaMessage::EndOfTrack => {
                            (EventType::MetaEndOfTrack, "End of Track".to_string())
                        }
                        _ => (EventType::Unknown, "Unknown Meta Event".to_string()),
                    };
                    
                    events.push(AbsoluteEvent {
                        absolute_time,
                        event_type,
                        channel: 0,
                        data1: 0,
                        data2: 0,
                        text,
                    });
                }
                TrackEventKind::Escape(_) => {
                    events.push(AbsoluteEvent {
                        absolute_time,
                        event_type: EventType::SystemExclusive,
                        channel: 0,
                        data1: 0,
                        data2: 0,
                        text: "Escape Sequence".to_string(),
                    });
                }
            }
        }
        
        Ok(TrackData {
            events,
            name: track_name,
            instrument: instrument_name,
            channel_mask,
        })
    }
    
    /// Get the duration of the file in ticks
    pub fn get_duration_ticks(&self) -> u32 {
        self.tracks.iter()
            .filter_map(|track| track.events.last())
            .map(|event| event.absolute_time)
            .max()
            .unwrap_or(0)
    }
    
    /// Convert ticks to milliseconds (approximate)
    pub fn ticks_to_ms(&self, ticks: u32, tempo_us_per_quarter: u32) -> f64 {
        match self.timing {
            Timing::Metrical(ticks_per_quarter) => {
                let ticks_per_quarter = ticks_per_quarter.as_int() as f64;
                let tempo_ms_per_quarter = tempo_us_per_quarter as f64 / 1000.0;
                (ticks as f64 / ticks_per_quarter) * tempo_ms_per_quarter
            }
            Timing::Timecode(fps, ticks_per_frame) => {
                let fps = fps.as_f32() as f64;
                let ticks_per_frame = ticks_per_frame as f64;
                (ticks as f64 / (fps * ticks_per_frame)) * 1000.0
            }
        }
    }
}

/// Load a MIDI file from disk
pub fn load_midi_file<P: AsRef<Path>>(path: P) -> Result<i32, Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    let midi_file = MidiFile::from_bytes(&data)?;
    
    let handle = get_next_file_handle();
    let mut files = get_midi_files().lock().unwrap();
    files.insert(handle, midi_file);
    
    Ok(handle)
}

/// Get a reference to a loaded MIDI file
pub fn get_midi_file(_handle: i32) -> Option<std::sync::MutexGuard<'static, HashMap<i32, MidiFile>>> {
    get_midi_files().lock().ok()
}

/// Close a MIDI file and free its resources
pub fn close_midi_file(handle: i32) -> bool {
    let mut files = get_midi_files().lock().unwrap();
    files.remove(&handle).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_midi_file_lifecycle() {
        // This test would need a sample MIDI file
        // For now, just test the basic structure
        assert_eq!(get_next_file_handle(), 1);
        assert_eq!(get_next_file_handle(), 2);
    }
}