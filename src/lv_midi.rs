use crate::midi::MidiManager;
use crate::labview_interop::sync::LVUserEvent;
use crate::labview_interop::types::LVStatusCode;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uchar};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::mpsc::{self, Receiver};

// Global storage for MIDI managers (thread-safe)
static MIDI_MANAGERS: OnceLock<Mutex<HashMap<i32, MidiManager>>> = OnceLock::new();
static NEXT_HANDLE: OnceLock<Mutex<i32>> = OnceLock::new();

fn get_midi_managers() -> &'static Mutex<HashMap<i32, MidiManager>> {
    MIDI_MANAGERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_next_handle_mutex() -> &'static Mutex<i32> {
    NEXT_HANDLE.get_or_init(|| Mutex::new(1))
}

fn get_next_handle() -> i32 {
    let mut handle = get_next_handle_mutex().lock().unwrap();
    let current = *handle;
    *handle += 1;
    current
}

// ========== DEVICE DISCOVERY ==========

/// Get the number of MIDI input devices
#[no_mangle]
pub extern "C" fn midi_get_input_device_count() -> c_int {
    let manager = MidiManager::new();
    match manager.list_input_devices() {
        Ok(devices) => devices.len() as c_int,
        Err(_) => -1,
    }
}

/// Get the number of MIDI output devices
#[no_mangle]
pub extern "C" fn midi_get_output_device_count() -> c_int {
    let manager = MidiManager::new();
    match manager.list_output_devices() {
        Ok(devices) => devices.len() as c_int,
        Err(_) => -1,
    }
}

/// Get the name of a MIDI input device
#[no_mangle]
pub extern "C" fn midi_get_input_device_name(
    device_index: c_int,
    buffer: *mut c_char,
    buffer_size: c_int,
) -> c_int {
    if buffer.is_null() || buffer_size <= 0 {
        return -1;
    }

    let manager = MidiManager::new();
    match manager.list_input_devices() {
        Ok(devices) => {
            if device_index < 0 || device_index >= devices.len() as c_int {
                return -1;
            }
            
            let device_name = &devices[device_index as usize];
            let c_string = match CString::new(device_name.clone()) {
                Ok(s) => s,
                Err(_) => return -1,
            };
            
            let name_bytes = c_string.as_bytes_with_nul();
            if name_bytes.len() > buffer_size as usize {
                return -1;
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
        Err(_) => -1,
    }
}

/// Get the name of a MIDI output device
#[no_mangle]
pub extern "C" fn midi_get_output_device_name(
    device_index: c_int,
    buffer: *mut c_char,
    buffer_size: c_int,
) -> c_int {
    if buffer.is_null() || buffer_size <= 0 {
        return -1;
    }

    let manager = MidiManager::new();
    match manager.list_output_devices() {
        Ok(devices) => {
            if device_index < 0 || device_index >= devices.len() as c_int {
                return -1;
            }
            
            let device_name = &devices[device_index as usize];
            let c_string = match CString::new(device_name.clone()) {
                Ok(s) => s,
                Err(_) => return -1,
            };
            
            let name_bytes = c_string.as_bytes_with_nul();
            if name_bytes.len() > buffer_size as usize {
                return -1;
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
        Err(_) => -1,
    }
}

// ========== CONNECTION MANAGEMENT ==========

/// Create a new MIDI manager instance
#[no_mangle]
pub extern "C" fn midi_create_manager() -> c_int {
    let handle = get_next_handle();
    let manager = MidiManager::new();
    
    let mut managers = get_midi_managers().lock().unwrap();
    managers.insert(handle, manager);
    handle
}

/// Destroy a MIDI manager instance
#[no_mangle]
pub extern "C" fn midi_destroy_manager(handle: c_int) -> c_int {
    let mut managers = get_midi_managers().lock().unwrap();
    match managers.remove(&handle) {
        Some(_) => 0,
        None => -1,
    }
}

/// Connect to a MIDI input device
#[no_mangle]
pub extern "C" fn midi_connect_input(handle: c_int, device_index: c_int) -> c_int {
    let mut managers = get_midi_managers().lock().unwrap();
    match managers.get_mut(&handle) {
        Some(manager) => {
            match manager.connect_input(device_index as usize) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        None => -1,
    }
}

/// Connect to a MIDI output device
#[no_mangle]
pub extern "C" fn midi_connect_output(handle: c_int, device_index: c_int) -> c_int {
    let mut managers = get_midi_managers().lock().unwrap();
    match managers.get_mut(&handle) {
        Some(manager) => {
            match manager.connect_output(device_index as usize) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        None => -1,
    }
}

// ========== MIDI COMMUNICATION ==========

/// Send a MIDI message
#[no_mangle]
pub extern "C" fn midi_send_message(
    handle: c_int,
    message: *const c_uchar,
    message_length: c_int,
) -> c_int {
    if message.is_null() || message_length <= 0 {
        return -1;
    }

    let message_slice = unsafe {
        std::slice::from_raw_parts(message, message_length as usize)
    };

    let mut managers = get_midi_managers().lock().unwrap();
    match managers.get_mut(&handle) {
        Some(manager) => {
            match manager.send_message(message_slice) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        None => -1,
    }
}

/// Receive a MIDI message (non-blocking)
#[no_mangle]
pub extern "C" fn midi_receive_message(
    handle: c_int,
    buffer: *mut c_uchar,
    buffer_size: c_int,
    message_length: *mut c_int,
) -> c_int {
    if buffer.is_null() || message_length.is_null() || buffer_size <= 0 {
        return -1;
    }

    let managers = get_midi_managers().lock().unwrap();
    match managers.get(&handle) {
        Some(manager) => {
            match manager.receive_message() {
                Some(msg) => {
                    if msg.len() > buffer_size as usize {
                        return -1;
                    }
                    
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            msg.as_ptr(),
                            buffer,
                            msg.len(),
                        );
                        *message_length = msg.len() as c_int;
                    }
                    1
                }
                None => 0,
            }
        }
        None => -1,
    }
}

// ========== HELPER FUNCTIONS ==========

/// Create a Note On message
#[no_mangle]
pub extern "C" fn midi_create_note_on(
    channel: c_uchar,
    note: c_uchar,
    velocity: c_uchar,
    buffer: *mut c_uchar,
) -> c_int {
    if buffer.is_null() {
        return -1;
    }

    let message = MidiManager::note_on(channel, note, velocity);
    unsafe {
        std::ptr::copy_nonoverlapping(message.as_ptr(), buffer, 3);
    }
    3
}

/// Create a Note Off message
#[no_mangle]
pub extern "C" fn midi_create_note_off(
    channel: c_uchar,
    note: c_uchar,
    velocity: c_uchar,
    buffer: *mut c_uchar,
) -> c_int {
    if buffer.is_null() {
        return -1;
    }

    let message = MidiManager::note_off(channel, note, velocity);
    unsafe {
        std::ptr::copy_nonoverlapping(message.as_ptr(), buffer, 3);
    }
    3
}

/// Create a Control Change message
#[no_mangle]
pub extern "C" fn midi_create_control_change(
    channel: c_uchar,
    controller: c_uchar,
    value: c_uchar,
    buffer: *mut c_uchar,
) -> c_int {
    if buffer.is_null() {
        return -1;
    }

    let message = MidiManager::control_change(channel, controller, value);
    unsafe {
        std::ptr::copy_nonoverlapping(message.as_ptr(), buffer, 3);
    }
    3
}

// ========== MIDI MESSAGE PARSING ==========

/// Parse a MIDI message into its components
#[no_mangle]
pub extern "C" fn midi_parse_message(
    message: *const c_uchar,
    message_length: c_int,
    message_type: *mut c_uchar,
    channel: *mut c_uchar,
    note_or_controller: *mut c_uchar,
    velocity_or_value: *mut c_uchar,
) -> c_int {
    if message.is_null() || message_type.is_null() || channel.is_null() || 
       note_or_controller.is_null() || velocity_or_value.is_null() || message_length < 1 {
        return -1;
    }

    let message_slice = unsafe {
        std::slice::from_raw_parts(message, message_length as usize)
    };

    if message_slice.is_empty() {
        return -1;
    }

    let status_byte = message_slice[0];
    let midi_channel = status_byte & 0x0F;
    let msg_type = status_byte & 0xF0;
    
    unsafe {
        *channel = midi_channel;
        
        match msg_type {
            0x80 => {
                *message_type = 0;
                if message_length >= 3 {
                    *note_or_controller = message_slice[1];
                    *velocity_or_value = message_slice[2];
                } else {
                    *note_or_controller = 0;
                    *velocity_or_value = 0;
                }
            },
            0x90 => {
                if message_length >= 3 {
                    *note_or_controller = message_slice[1];
                    *velocity_or_value = message_slice[2];
                    
                    if message_slice[2] == 0 {
                        *message_type = 0;
                    } else {
                        *message_type = 1;
                    }
                } else {
                    *message_type = 1;
                    *note_or_controller = 0;
                    *velocity_or_value = 0;
                }
            },
            0xB0 => {
                *message_type = 2;
                if message_length >= 3 {
                    *note_or_controller = message_slice[1];
                    *velocity_or_value = message_slice[2];
                } else {
                    *note_or_controller = 0;
                    *velocity_or_value = 0;
                }
            },
            0xC0 => {
                *message_type = 3;
                if message_length >= 2 {
                    *note_or_controller = message_slice[1];
                    *velocity_or_value = 0;
                } else {
                    *note_or_controller = 0;
                    *velocity_or_value = 0;
                }
            },
            0xE0 => {
                *message_type = 4;
                if message_length >= 3 {
                    let lsb = message_slice[1] as u16;
                    let msb = message_slice[2] as u16;
                    let bend_value = (msb << 7) | lsb;
                    
                    *note_or_controller = (bend_value & 0xFF) as u8;
                    *velocity_or_value = ((bend_value >> 8) & 0xFF) as u8;
                } else {
                    *note_or_controller = 64;
                    *velocity_or_value = 64;
                }
            },
            _ => {
                *message_type = 255;
                *note_or_controller = 0;
                *velocity_or_value = 0;
            }
        }
    }
    
    0
}

// ========== LABVIEW USER EVENTS - FIXED IMPLEMENTATION ==========

/// MIDI data structure for LabVIEW User Events
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MidiEventData {
    pub message_type: i32,
    pub channel: i32,
    pub note_or_controller: i32,
    pub velocity_or_value: i32,
    pub raw_status: i32,
}

// Storage for event-based MIDI listeners
static EVENT_LISTENERS: OnceLock<Mutex<HashMap<i32, EventListener>>> = OnceLock::new();

struct EventListener {
    user_event: Option<LVUserEvent<MidiEventData>>,
    filter_array: Vec<u8>,
    device_index: Option<usize>,
    running: Arc<std::sync::atomic::AtomicBool>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    // Store the message receiver from the MIDI manager
    message_receiver: Option<Receiver<Vec<u8>>>,
}

impl EventListener {
    fn new() -> Self {
        EventListener {
            user_event: None,
            filter_array: Vec::new(),
            device_index: None,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            thread_handle: None,
            message_receiver: None,
        }
    }
}

fn get_event_listeners() -> &'static Mutex<HashMap<i32, EventListener>> {
    EVENT_LISTENERS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Create a new event-based MIDI listener for LabVIEW User Events
#[no_mangle]
pub extern "C" fn midi_create_event_listener() -> c_int {
    let handle = get_next_handle();
    let listener = EventListener::new();
    
    let mut listeners = get_event_listeners().lock().unwrap();
    listeners.insert(handle, listener);
    handle
}

/// Set the LabVIEW User Event reference (as a raw u32 value)
#[no_mangle]
pub extern "C" fn midi_set_user_event(
    handle: c_int,
    user_event_ref: u32,
) -> c_int {
    let mut listeners = get_event_listeners().lock().unwrap();
    match listeners.get_mut(&handle) {
        Some(listener) => {
            listener.user_event = Some(LVUserEvent::from_raw(user_event_ref));
            0
        }
        None => -1,
    }
}

/// Set the message filter array
#[no_mangle]
pub extern "C" fn midi_set_message_filter(
    handle: c_int,
    filter_array: *const c_uchar,
    array_size: c_int,
) -> c_int {
    let mut listeners = get_event_listeners().lock().unwrap();
    match listeners.get_mut(&handle) {
        Some(listener) => {
            if array_size == 0 {
                listener.filter_array.clear();
            } else if !filter_array.is_null() && array_size > 0 {
                let filter_slice = unsafe {
                    std::slice::from_raw_parts(filter_array, array_size as usize)
                };
                listener.filter_array = filter_slice.to_vec();
            } else {
                return -1;
            }
            0
        }
        None => -1,
    }
}

/// Connect the event listener to a MIDI input device
/// This creates a dedicated MIDI connection for the event listener
#[no_mangle]
pub extern "C" fn midi_connect_event_input(
    handle: c_int,
    device_index: c_int,
) -> c_int {
    let mut listeners = get_event_listeners().lock().unwrap();
    
    match listeners.get_mut(&handle) {
        Some(listener) => {
            // Create a dedicated MIDI manager for this event listener
            let mut dedicated_manager = MidiManager::new();
            match dedicated_manager.connect_input(device_index as usize) {
                Ok(_) => {
                    listener.device_index = Some(device_index as usize);
                    // We can't store the manager directly due to threading issues,
                    // but we store the device index for later use
                    0
                }
                Err(_) => -1,
            }
        }
        None => -1,
    }
}

/// Start event listening - FIXED IMPLEMENTATION
#[no_mangle]
pub extern "C" fn midi_start_event_listening(handle: c_int) -> c_int {
    let mut listeners = get_event_listeners().lock().unwrap();
    
    match listeners.get_mut(&handle) {
        Some(listener) => {
            if listener.user_event.is_none() {
                return -1;
            }

            let device_index = match listener.device_index {
                Some(idx) => idx,
                None => return -1,
            };

            if listener.running.load(std::sync::atomic::Ordering::Relaxed) {
                return -1;
            }

            listener.running.store(true, std::sync::atomic::Ordering::Relaxed);
            
            let running_flag = listener.running.clone();
            let user_event = listener.user_event.unwrap();
            let filter_array = listener.filter_array.clone();
            
            // FIXED: Create the MIDI manager and connection in the thread
            let thread_handle = std::thread::spawn(move || {
                // Create a fresh MIDI manager for this thread
                let mut midi_manager = MidiManager::new();
                
                // Connect to the MIDI device
                if let Err(e) = midi_manager.connect_input(device_index) {
                    eprintln!("Failed to connect to MIDI device in thread: {}", e);
                    return;
                }
                
                // Main listening loop
                while running_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    // Check for MIDI messages
                    if let Some(message) = midi_manager.receive_message() {
                        if !message.is_empty() {
                            let status_byte = message[0];
                            
                            // Apply filter if specified
                            if filter_array.is_empty() || filter_array.contains(&status_byte) {
                                // Parse the MIDI message
                                let channel = status_byte & 0x0F;
                                let msg_type = status_byte & 0xF0;
                                let data1 = if message.len() > 1 { message[1] } else { 0 };
                                let data2 = if message.len() > 2 { message[2] } else { 0 };
                                
                                let message_type = match msg_type {
                                    0x80 => 0, // Note Off
                                    0x90 => if data2 == 0 { 0 } else { 1 }, // Note On (velocity 0 = Note Off)
                                    0xB0 => 2, // Control Change
                                    0xC0 => 3, // Program Change
                                    0xE0 => 4, // Pitch Bend
                                    _ => 255,  // Unknown
                                };
                                
                                // Create event data
                                let mut event_data = MidiEventData {
                                    message_type: message_type as i32,
                                    channel: channel as i32,
                                    note_or_controller: data1 as i32,
                                    velocity_or_value: data2 as i32,
                                    raw_status: status_byte as i32,
                                };
                                
                                // Post the event to LabVIEW
                                if let Err(e) = user_event.post(&mut event_data) {
                                    eprintln!("Failed to post MIDI event to LabVIEW: {}", e);
                                }
                            }
                        }
                    }
                    
                    // Small delay to prevent busy waiting
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            });
            
            listener.thread_handle = Some(thread_handle);
            0
        }
        None => -1,
    }
}

/// Stop event listening
#[no_mangle]
pub extern "C" fn midi_stop_event_listening(handle: c_int) -> c_int {
    let mut listeners = get_event_listeners().lock().unwrap();
    
    match listeners.get_mut(&handle) {
        Some(listener) => {
            listener.running.store(false, std::sync::atomic::Ordering::Relaxed);
            
            if let Some(thread_handle) = listener.thread_handle.take() {
                drop(listeners);
                let _ = thread_handle.join();
            }
            
            0
        }
        None => 0,
    }
}

/// Destroy an event listener
#[no_mangle]
pub extern "C" fn midi_destroy_event_listener(handle: c_int) -> c_int {
    let _ = midi_stop_event_listening(handle);
    
    let mut listeners = get_event_listeners().lock().unwrap();
    listeners.remove(&handle);
    0
}

/// Get listener status for debugging
#[no_mangle]
pub extern "C" fn midi_get_listener_status(handle: c_int) -> c_int {
    let listeners = get_event_listeners().lock().unwrap();
    match listeners.get(&handle) {
        Some(listener) => {
            if listener.running.load(std::sync::atomic::Ordering::Relaxed) {
                1
            } else {
                0
            }
        }
        None => -1,
    }
}

/// Convenience function: Open MIDI input with LabVIEW User Event in one call
#[no_mangle]
pub extern "C" fn midi_open_with_user_event(
    device_id: c_int,
    user_event_ref: u32,
    filter_array: *const c_uchar,
    array_size: c_int,
) -> c_int {
    let handle = midi_create_event_listener();
    if handle == -1 {
        return -1;
    }
    
    if midi_set_user_event(handle, user_event_ref) != 0 {
        midi_destroy_event_listener(handle);
        return -1;
    }
    
    if array_size > 0 && !filter_array.is_null() {
        if midi_set_message_filter(handle, filter_array, array_size) != 0 {
            midi_destroy_event_listener(handle);
            return -1;
        }
    }
    
    if midi_connect_event_input(handle, device_id) != 0 {
        midi_destroy_event_listener(handle);
        return -1;
    }
    
    if midi_start_event_listening(handle) != 0 {
        midi_destroy_event_listener(handle);
        return -1;
    }
    
    handle
}

// ========== ADDITIONAL HELPER FUNCTIONS ==========

/// Convert MIDI note number to note name
#[no_mangle]
pub extern "C" fn midi_note_to_name(
    note: c_uchar,
    buffer: *mut c_char,
    buffer_size: c_int,
) -> c_int {
    if buffer.is_null() || buffer_size < 4 {
        return -1;
    }

    if note > 127 {
        return -1;
    }

    let notes = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let octave = (note / 12) as i32 - 1;
    let note_index = (note % 12) as usize;
    
    let note_name = format!("{}{}", notes[note_index], octave);
    
    let c_string = match CString::new(note_name) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let name_bytes = c_string.as_bytes_with_nul();
    if name_bytes.len() > buffer_size as usize {
        return -1;
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

/// Get message type name for debugging
#[no_mangle]
pub extern "C" fn midi_get_message_type_name(
    message_type: c_uchar,
    buffer: *mut c_char,
    buffer_size: c_int,
) -> c_int {
    if buffer.is_null() || buffer_size <= 0 {
        return -1;
    }

    let type_name = match message_type {
        0 => "Note Off",
        1 => "Note On",
        2 => "Control Change",
        3 => "Program Change", 
        4 => "Pitch Bend",
        255 => "Unknown",
        _ => "Invalid",
    };

    let c_string = match CString::new(type_name) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let name_bytes = c_string.as_bytes_with_nul();
    if name_bytes.len() > buffer_size as usize {
        return -1;
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

// Return the LabVIEW status code enum values for use in LabVIEW
#[no_mangle]
pub extern "C" fn lv_status_success() -> c_int {
    LVStatusCode::SUCCESS as c_int
}

#[no_mangle]
pub extern "C" fn lv_status_error() -> c_int {
    LVStatusCode::ARG_ERROR as c_int
}

#[no_mangle]
pub extern "C" fn midi_connect_with_user_event(
    device_index: c_int,
    user_event_ref: u32,
    filter_array: *const c_uchar,
    array_size: c_int,
) -> c_int {
    use std::sync::Arc;
    
    // Create filter vector
    let filter = if array_size > 0 && !filter_array.is_null() {
        let filter_slice = unsafe {
            std::slice::from_raw_parts(filter_array, array_size as usize)
        };
        Arc::new(filter_slice.to_vec())
    } else {
        Arc::new(Vec::new())
    };
    
    // Create User Event
    let user_event = Arc::new(LVUserEvent::<MidiEventData>::from_raw(user_event_ref));
    
    // Create MIDI manager
    let mut manager = MidiManager::new();
    
    // Create the callback that will be called directly by midir
    let callback = {
        let filter = filter.clone();
        let user_event = user_event.clone();
        
        move |message: Vec<u8>| {
            if !message.is_empty() {
                let status_byte = message[0];
                
                // Apply filter if specified
                if filter.is_empty() || filter.contains(&status_byte) {
                    // Parse the MIDI message
                    let channel = status_byte & 0x0F;
                    let msg_type = status_byte & 0xF0;
                    let data1 = if message.len() > 1 { message[1] } else { 0 };
                    let data2 = if message.len() > 2 { message[2] } else { 0 };
                    
                    let message_type = match msg_type {
                        0x80 => 0, // Note Off
                        0x90 => if data2 == 0 { 0 } else { 1 }, // Note On
                        0xB0 => 2, // Control Change
                        0xC0 => 3, // Program Change
                        0xE0 => 4, // Pitch Bend
                        _ => 255,  // Unknown
                    };
                    
                    // Create event data
                    let mut event_data = MidiEventData {
                        message_type: message_type as i32,
                        channel: channel as i32,
                        note_or_controller: data1 as i32,
                        velocity_or_value: data2 as i32,
                        raw_status: status_byte as i32,
                    };
                    
                    // Post the event to LabVIEW directly from midir's callback
                    if let Err(e) = user_event.post(&mut event_data) {
                        eprintln!("Failed to post MIDI event to LabVIEW: {}", e);
                    }
                }
            }
        }
    };
    
    // Connect with the callback
    match manager.connect_input_with_callback(device_index as usize, callback) {
        Ok(_) => {
            // Store the manager to keep the connection alive
            let handle = get_next_handle();
            let mut managers = get_midi_managers().lock().unwrap();
            managers.insert(handle, manager);
            handle
        }
        Err(_) => -1,
    }
}

/// Disconnect and cleanup a MIDI connection
#[no_mangle]
pub extern "C" fn midi_disconnect(handle: c_int) -> c_int {
    let mut managers = get_midi_managers().lock().unwrap();
    match managers.remove(&handle) {
        Some(_) => {
            // The MidiManager will be dropped here, which closes the connection
            println!("Disconnected MIDI handle {}", handle);
            0
        }
        None => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_counting() {
        let input_count = midi_get_input_device_count();
        assert!(input_count >= 0);
        
        let output_count = midi_get_output_device_count();
        assert!(output_count >= 0);
        
        println!("Found {} input devices, {} output devices", input_count, output_count);
    }

    #[test]
    fn test_manager_lifecycle() {
        let handle = midi_create_manager();
        assert!(handle > 0);
        
        let result = midi_destroy_manager(handle);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_event_listener_lifecycle() {
        let handle = midi_create_event_listener();
        assert!(handle > 0);
        
        let result = midi_destroy_event_listener(handle);
        assert_eq!(result, 0);
    }
}