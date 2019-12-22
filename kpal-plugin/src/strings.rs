//! Provides tools for working with strings in KPAL plugins.
use std::error::Error;
use std::fmt;
use std::slice;

/// Copies a C-compatible string to a buffer.
///
/// # Arguments
///
/// * `string` - A reference to an array of bytes to copy
/// * `buffer` - An array of bytes to receive the copy of the string
/// * `length` - The length of the buffer
///
/// # Safety
///
/// This function is unsafe because of its use of slice::from_raw_parts, which relies on the caller
/// to not exceed the length of the buffer when generating the slice.
pub unsafe fn copy_string(
    string: &[u8],
    buffer: *mut u8,
    length: usize,
) -> Result<(), BufferOverflowError> {
    let buffer = slice::from_raw_parts_mut(buffer, length);
    if string.len() > buffer.len() {
        return Err(BufferOverflowError {});
    }

    buffer[..string.len()].copy_from_slice(string);

    Ok(())
}

/// Raised when the length of a string exceeds the length of the buffer into which it is copied.
#[derive(Debug)]
pub struct BufferOverflowError {}

impl fmt::Display for BufferOverflowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BufferOverflowError")
    }
}

impl Error for BufferOverflowError {
    fn description(&self) -> &str {
        "provided buffer is too small to copy the full contents of the data"
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::{CStr, CString};

    use super::*;

    #[test]
    fn test_copy_string() {
        let string = CString::new("foo").expect("Could not create CString");
        let buffer: &mut [u8; 4] = &mut [0; 4];
        let buffer_p = buffer.as_mut_ptr();

        let bytes = string.to_bytes_with_nul();

        unsafe {
            match copy_string(&bytes, buffer_p, buffer.len()) {
                Ok(_) => (),
                Err(_e) => panic!("Failed to copy string to buffer"),
            }
        }

        let result = CStr::from_bytes_with_nul(buffer).expect("Could not convert buffer to Cstr");
        assert_eq!(string.as_c_str(), result)
    }

    #[test]
    fn test_copy_string_buffer_overflow() {
        let string = CString::new("foo").expect("Could not create CString");
        let buffer: &mut [u8; 3] = &mut [0; 3];
        let buffer_p = buffer.as_mut_ptr();

        let bytes = string.to_bytes_with_nul();

        unsafe {
            if copy_string(&bytes, buffer_p, buffer.len()).is_ok() {
                panic!("Failed to return an error due to a buffer overflow")
            };
        }
    }
}
