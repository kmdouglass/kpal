use std::ffi::CString;

/// An Error contains information about the most recent error from an API call.
///
/// An Error has two states: triggered and untriggered. The `triggered` field will contain the
/// value `true` when the last KPAL API call resulted in an error. The message associated with the
/// error is contained in the `msg` field.
pub struct Error {
    msg: CString,
    triggered: bool,
}

impl Error {
    /// Initializes a new Error struct.
    pub fn new() -> Error {
        Error {
            msg: CString::new("").expect("Error: CString::new()"),
            triggered: false,
        }
    }

    /// Returns the most recent error message from a KPAL API call.
    ///
    /// Calling this method will reset the state of the Error struct by changing the `triggered`
    /// field to `false`.
    pub fn query(&mut self) -> Option<&CString> {
        if self.triggered {
            self.triggered = false;
            Some(&self.msg)
        } else {
            None
        }
    }

    /// Sets the state of the Error to `triggered` and sets the error message.
    ///
    /// # Arguments
    ///
    /// * `msg` - A `CString` that contains the error message.
    pub fn set(&mut self, msg: CString) {
        self.triggered = true;
        self.msg = msg;
    }
}
