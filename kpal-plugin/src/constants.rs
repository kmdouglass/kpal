use libc::c_char;

use crate::ffi::Phase;

/// Indicates that an attribute may not be set before plugin initialization.
pub const ATTRIBUTE_PRE_INIT_FALSE: c_char = 0;

/// Indicates that an attribute may be set before plugin initialization.
pub const ATTRIBUTE_PRE_INIT_TRUE: c_char = 1;

/// Indicates that the init phase callbacks should be used when interacting with a plugin.
pub const INIT_PHASE: Phase = 0;

/// Indicates that the run phase callbacks should be used when interacting with a plugin.
pub const RUN_PHASE: Phase = 1;

/// Error messages associated with each error code.
pub static ERRORS: [&[u8]; 13] = [
    // 0 PLUGIN_OK
    b"Plugin OK\0",
    // 1 UNDEFINED_ERR
    b"Undefined error\0",
    // 2 PLUGIN_INIT_ERR
    b"Plugin failed to initialize\0",
    // 3 PLUGIN_UNINIT_ERR
    b"Plugin was wrongly assumed to be initialized",
    // 4 ATTRIBUTE_DOES_NOT_EXIST
    b"Attribute does not exist\0",
    // 5 ATTRIBUTE_TYPE_MISMATCH
    b"Attribute types do not match\0",
    // 6 ATTRIBUTE_IS_NOT_SETTABLE
    b"Attribute cannot be set\0",
    // 7 IO_ERR
    b"IO operation failed\0",
    // 8 CONVERSION_ERR
    b"Could not convert value into a different type\0",
    // 9 NULL_PTR_ERR
    b"The plugin encountered a null pointer\0",
    // 10 CALLBACK_ERR
    b"The plugin attribute's callback failed\0",
    // 11 UPDATE_CACHED_VALUE_ERR
    b"Could not update plugin attribute's cached value\0",
    // 12 LIFECYCLE_PHASE_ERR
    b"Unrecognized lifecycle phase\0",
];

pub mod error_codes {
    //! Constants that indicate specific error codes that a plugin can return.
    use libc::c_int;

    pub const PLUGIN_OK: c_int = 0;
    pub const UNDEFINED_ERR: c_int = 1;
    pub const PLUGIN_INIT_ERR: c_int = 2;
    pub const PLUGIN_UNINIT_ERR: c_int = 3;
    pub const ATTRIBUTE_DOES_NOT_EXIST: c_int = 4;
    pub const ATTRIBUTE_TYPE_MISMATCH: c_int = 5;
    pub const ATTRIBUTE_IS_NOT_SETTABLE: c_int = 6;
    pub const IO_ERR: c_int = 7;
    pub const CONVERSION_ERR: c_int = 8;
    pub const NULL_PTR_ERR: c_int = 9;
    pub const CALLBACK_ERR: c_int = 10;
    pub const UPDATE_CACHED_VALUE_ERR: c_int = 11;
    pub const LIFECYCLE_PHASE_ERR: c_int = 12;
}
