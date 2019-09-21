use std::error::Error;
use std::fmt;
use std::slice;

pub unsafe fn copy_string(
    string: &[u8],
    buffer: *mut u8,
    length: usize,
) -> Result<(), BufferOverflowError> {
    let mut buffer = slice::from_raw_parts_mut(buffer, length);
    if string.len() > buffer.len() {
        return Err(BufferOverflowError {});
    }

    &buffer[..string.len()].copy_from_slice(string);

    Ok(())
}

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
                Err(_) => panic!("Failed to copy string to buffer"),
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
            match copy_string(&bytes, buffer_p, buffer.len()) {
                Ok(_) => panic!("Failed to return an error due to a buffer overflow"),
                _ => (),
            }
        }
    }
}
