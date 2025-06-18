use crate::midi::MidiManager;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uchar};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// Global storage for MIDI managers (thread-safe)
// LabVIEW will get a handle (ID) to reference each manager
static MIDI_MANAGERS: OnceLock<Mutex<HashMap<i32, MidiManager>>> = OnceLock::new();
static NEXT_HANDLE: OnceLock<Mutex<i32>> = OnceLock::new();

fn get_midi_managers() -> &'static Mutex<HashMap<i32, MidiManager>> {
    MIDI_MANAGERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_next_handle_mutex() -> &'static Mutex<i32> {
    NEXT_HANDLE.get_or_init(|| Mutex::new(1))
}

// Helper function to get the next available handle
fn get_next_handle() -> i32 {
    let mut handle = get_next_handle_mutex().lock().unwrap();
    let current = *handle;
    *handle += 1;
    current
}

// ========== DEVICE DISCOVERY ==========

/// Get the number of MIDI input devices
/// Returns: Number of devices, or -1 on error
#[unsafe(no_mangle)]
pub extern "C" fn midi_get_input_device_count() -> c_int {
    let manager = MidiManager::new();
    match manager.list_input_devices() {
        Ok(devices) => devices.len() as c_int,
        Err(_) => -1,
    }
}

/// Get the number of MIDI output devices
/// Returns: Number of devices, or -1 on error
#[unsafe(no_mangle)]
pub extern "C" fn midi_get_output_device_count() -> c_int {
    let manager = MidiManager::new();
    match manager.list_output_devices() {
        Ok(devices) => devices.len() as c_int,
        Err(_) => -1,
    }
}

/// Get the name of a MIDI input device
/// device_index: Index of the device (0-based)
/// buffer: Buffer to write the device name into
/// buffer_size: Size of the buffer
/// Returns: 0 on success, -1 on error
#[unsafe(no_mangle)]
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
                return -1; // Buffer too small
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

/// Get the name of a MIDI output device (similar to input version)
#[unsafe(no_mangle)]
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
/// Returns: Handle to the manager, or -1 on error
#[unsafe(no_mangle)]
pub extern "C" fn midi_create_manager() -> c_int {
    let handle = get_next_handle();
    let manager = MidiManager::new();
    
    let mut managers = get_midi_managers().lock().unwrap();
    managers.insert(handle, manager);
    handle
}

/// Destroy a MIDI manager instance
/// handle: Handle returned by midi_create_manager
/// Returns: 0 on success, -1 on error
#[unsafe(no_mangle)]
pub extern "C" fn midi_destroy_manager(handle: c_int) -> c_int {
    let mut managers = get_midi_managers().lock().unwrap();
    match managers.remove(&handle) {
        Some(_) => 0,
        None => -1,
    }
}

/// Connect to a MIDI input device
/// handle: Manager handle
/// device_index: Index of the device to connect to
/// Returns: 0 on success, -1 on error
#[unsafe(no_mangle)]
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
/// handle: Manager handle
/// device_index: Index of the device to connect to
/// Returns: 0 on success, -1 on error
#[unsafe(no_mangle)]
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
/// handle: Manager handle
/// message: Pointer to MIDI message bytes
/// message_length: Length of the message
/// Returns: 0 on success, -1 on error
#[unsafe(no_mangle)]
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
/// handle: Manager handle
/// buffer: Buffer to write the message into
/// buffer_size: Size of the buffer
/// message_length: Pointer to write the actual message length
/// Returns: 1 if message received, 0 if no message, -1 on error
#[unsafe(no_mangle)]
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
                        return -1; // Buffer too small
                    }
                    
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            msg.as_ptr(),
                            buffer,
                            msg.len(),
                        );
                        *message_length = msg.len() as c_int;
                    }
                    1 // Message received
                }
                None => 0, // No message available
            }
        }
        None => -1, // Invalid handle
    }
}

// ========== HELPER FUNCTIONS ==========

/// Create a Note On message
/// channel: MIDI channel (0-15)
/// note: Note number (0-127)
/// velocity: Velocity (0-127)
/// buffer: Buffer to write the message (must be at least 3 bytes)
/// Returns: Message length (3) on success, -1 on error
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int, c_uchar};

    #[test]
    fn test_device_counting() {
        // Test input device counting
        let input_count = midi_get_input_device_count();
        assert!(input_count >= 0, "Input device count should be non-negative");
        
        // Test output device counting
        let output_count = midi_get_output_device_count();
        assert!(output_count >= 0, "Output device count should be non-negative");
        
        println!("Found {} input devices, {} output devices", input_count, output_count);
    }

    #[test]
    fn test_device_name_retrieval() {
        let input_count = midi_get_input_device_count();
        
        if input_count > 0 {
            // Test getting first device name
            let mut buffer = [0u8; 256];
            let result = midi_get_input_device_name(
                0,
                buffer.as_mut_ptr() as *mut c_char,
                buffer.len() as c_int,
            );
            
            assert_eq!(result, 0, "Should successfully get device name");
            
            // Safely convert to string without taking ownership
            let name = unsafe { 
                let c_str = std::ffi::CStr::from_ptr(buffer.as_ptr() as *const c_char);
                c_str.to_string_lossy().to_string()
            };
            
            println!("First input device: {}", name);
            assert!(!name.is_empty(), "Device name should not be empty");
        }
        
        // Test invalid device index
        let mut buffer = [0u8; 256];
        let result = midi_get_input_device_name(
            99999, // Invalid index
            buffer.as_mut_ptr() as *mut c_char,
            buffer.len() as c_int,
        );
        assert_eq!(result, -1, "Should return error for invalid device index");
    }

    #[test]
    fn test_device_name_edge_cases() {
        // Test null buffer
        let result = midi_get_input_device_name(0, std::ptr::null_mut(), 256);
        assert_eq!(result, -1, "Should return error for null buffer");
        
        // Test zero buffer size
        let mut buffer = [0u8; 256];
        let result = midi_get_input_device_name(
            0,
            buffer.as_mut_ptr() as *mut c_char,
            0,
        );
        assert_eq!(result, -1, "Should return error for zero buffer size");
        
        // Test negative buffer size
        let result = midi_get_input_device_name(
            0,
            buffer.as_mut_ptr() as *mut c_char,
            -1,
        );
        assert_eq!(result, -1, "Should return error for negative buffer size");
    }

    #[test]
    fn test_manager_lifecycle() {
        // Test creating manager
        let handle = midi_create_manager();
        assert!(handle > 0, "Manager handle should be positive");
        
        // Test creating multiple managers
        let handle2 = midi_create_manager();
        assert!(handle2 > 0, "Second manager handle should be positive");
        assert_ne!(handle, handle2, "Manager handles should be unique");
        
        // Test destroying managers
        let result1 = midi_destroy_manager(handle);
        assert_eq!(result1, 0, "Should successfully destroy first manager");
        
        let result2 = midi_destroy_manager(handle2);
        assert_eq!(result2, 0, "Should successfully destroy second manager");
        
        // Test destroying non-existent manager
        let result3 = midi_destroy_manager(handle);
        assert_eq!(result3, -1, "Should return error for already destroyed manager");
    }

    #[test]
    fn test_connection_functions() {
        let handle = midi_create_manager();
        assert!(handle > 0, "Should create manager successfully");
        
        // Test connecting to invalid device (should fail gracefully)
        let result = midi_connect_input(handle, 99999);
        assert_eq!(result, -1, "Should return error for invalid device index");
        
        let result = midi_connect_output(handle, 99999);
        assert_eq!(result, -1, "Should return error for invalid device index");
        
        // Test connecting with invalid handle
        let result = midi_connect_input(99999, 0);
        assert_eq!(result, -1, "Should return error for invalid handle");
        
        // Clean up
        midi_destroy_manager(handle);
    }

    #[test]
    fn test_message_creation() {
        let mut buffer = [0u8; 3];
        
        // Test Note On creation
        let length = midi_create_note_on(0, 60, 100, buffer.as_mut_ptr());
        assert_eq!(length, 3, "Note On should be 3 bytes");
        assert_eq!(buffer[0], 0x90, "Should be Note On status byte");
        assert_eq!(buffer[1], 60, "Should have correct note number");
        assert_eq!(buffer[2], 100, "Should have correct velocity");
        
        // Test Note Off creation
        buffer = [0u8; 3];
        let length = midi_create_note_off(1, 64, 0, buffer.as_mut_ptr());
        assert_eq!(length, 3, "Note Off should be 3 bytes");
        assert_eq!(buffer[0], 0x81, "Should be Note Off status byte for channel 1");
        assert_eq!(buffer[1], 64, "Should have correct note number");
        assert_eq!(buffer[2], 0, "Should have correct velocity");
        
        // Test Control Change creation
        buffer = [0u8; 3];
        let length = midi_create_control_change(2, 7, 127, buffer.as_mut_ptr());
        assert_eq!(length, 3, "Control Change should be 3 bytes");
        assert_eq!(buffer[0], 0xB2, "Should be Control Change status byte for channel 2");
        assert_eq!(buffer[1], 7, "Should have correct controller number");
        assert_eq!(buffer[2], 127, "Should have correct value");
    }

    #[test]
    fn test_message_creation_edge_cases() {
        // Test null buffer
        let length = midi_create_note_on(0, 60, 100, std::ptr::null_mut());
        assert_eq!(length, -1, "Should return error for null buffer");
        
        // Test channel bounds (should handle channel > 15)
        let mut buffer = [0u8; 3];
        let length = midi_create_note_on(16, 60, 100, buffer.as_mut_ptr());
        assert_eq!(length, 3, "Should still create message");
        assert_eq!(buffer[0] & 0x0F, 0, "Should wrap channel to 0");
        
        // Test note bounds (should handle note > 127)
        buffer = [0u8; 3];
        let length = midi_create_note_on(0, 200, 100, buffer.as_mut_ptr());
        assert_eq!(length, 3, "Should still create message");
        assert_eq!(buffer[1] & 0x7F, 72, "Should mask note to 7 bits (200 & 0x7F = 72)");
        
        // Test velocity bounds
        buffer = [0u8; 3];
        let length = midi_create_note_on(0, 60, 200, buffer.as_mut_ptr());
        assert_eq!(length, 3, "Should still create message");
        assert_eq!(buffer[2] & 0x7F, 72, "Should mask velocity to 7 bits (200 & 0x7F = 72)");
    }

    #[test]
    fn test_send_message_safety() {
        let handle = midi_create_manager();
        assert!(handle > 0, "Should create manager successfully");
        
        // Test sending with null message
        let result = midi_send_message(handle, std::ptr::null(), 3);
        assert_eq!(result, -1, "Should return error for null message");
        
        // Test sending with zero length
        let message = [0x90, 60, 100];
        let result = midi_send_message(handle, message.as_ptr(), 0);
        assert_eq!(result, -1, "Should return error for zero length");
        
        // Test sending with negative length
        let result = midi_send_message(handle, message.as_ptr(), -1);
        assert_eq!(result, -1, "Should return error for negative length");
        
        // Test sending with invalid handle
        let result = midi_send_message(99999, message.as_ptr(), 3);
        assert_eq!(result, -1, "Should return error for invalid handle");
        
        // Clean up
        midi_destroy_manager(handle);
    }

    #[test]
    fn test_receive_message_safety() {
        let handle = midi_create_manager();
        assert!(handle > 0, "Should create manager successfully");
        
        let mut buffer = [0u8; 256];
        let mut message_length: c_int = 0;
        
        // Test receiving with null buffer
        let result = midi_receive_message(
            handle,
            std::ptr::null_mut(),
            256,
            &mut message_length,
        );
        assert_eq!(result, -1, "Should return error for null buffer");
        
        // Test receiving with null message_length pointer
        let result = midi_receive_message(
            handle,
            buffer.as_mut_ptr(),
            256,
            std::ptr::null_mut(),
        );
        assert_eq!(result, -1, "Should return error for null message_length pointer");
        
        // Test receiving with zero buffer size
        let result = midi_receive_message(
            handle,
            buffer.as_mut_ptr(),
            0,
            &mut message_length,
        );
        assert_eq!(result, -1, "Should return error for zero buffer size");
        
        // Test receiving with invalid handle
        let result = midi_receive_message(
            99999,
            buffer.as_mut_ptr(),
            256,
            &mut message_length,
        );
        assert_eq!(result, -1, "Should return error for invalid handle");
        
        // Test receiving with no connection (should return 0 - no message)
        let result = midi_receive_message(
            handle,
            buffer.as_mut_ptr(),
            256,
            &mut message_length,
        );
        assert_eq!(result, 0, "Should return 0 for no message available");
        
        // Clean up
        midi_destroy_manager(handle);
    }

    #[test]
    fn test_concurrent_managers() {
        // Test that multiple managers can coexist
        let handles: Vec<c_int> = (0..5).map(|_| midi_create_manager()).collect();
        
        // All handles should be valid and unique
        for &handle in &handles {
            assert!(handle > 0, "Handle should be positive");
        }
        
        // Check uniqueness
        for i in 0..handles.len() {
            for j in i+1..handles.len() {
                assert_ne!(handles[i], handles[j], "Handles should be unique");
            }
        }
        
        // Clean up all managers
        for handle in handles {
            let result = midi_destroy_manager(handle);
            assert_eq!(result, 0, "Should successfully destroy manager");
        }
    }

    #[test]
    #[ignore] // Run with: cargo test test_full_workflow -- --ignored --nocapture
    fn test_full_workflow() {
        // This test requires actual MIDI devices
        println!("=== Full Workflow Test ===");
        
        // 1. Check device availability
        let input_count = midi_get_input_device_count();
        let output_count = midi_get_output_device_count();
        
        println!("Available devices: {} inputs, {} outputs", input_count, output_count);
        
        if input_count == 0 && output_count == 0 {
            println!("No MIDI devices available - skipping workflow test");
            return;
        }
        
        // 2. Create manager
        let handle = midi_create_manager();
        assert!(handle > 0, "Should create manager");
        
        // 3. Try to connect to first available device
        if input_count > 0 {
            let result = midi_connect_input(handle, 0);
            if result == 0 {
                println!("✅ Successfully connected to input device");
            } else {
                println!("❌ Failed to connect to input device");
            }
        }
        
        if output_count > 0 {
            let result = midi_connect_output(handle, 0);
            if result == 0 {
                println!("✅ Successfully connected to output device");
                
                // 4. Try sending a test message
                let mut note_on_buffer = [0u8; 3];
                midi_create_note_on(0, 60, 100, note_on_buffer.as_mut_ptr());
                
                let send_result = midi_send_message(handle, note_on_buffer.as_ptr(), 3);
                if send_result == 0 {
                    println!("✅ Successfully sent Note On message");
                } else {
                    println!("❌ Failed to send Note On message");
                }
            } else {
                println!("❌ Failed to connect to output device");
            }
        }
        
        // 5. Clean up
        let destroy_result = midi_destroy_manager(handle);
        assert_eq!(destroy_result, 0, "Should destroy manager successfully");
        
        println!("✅ Workflow test completed");
    }
}