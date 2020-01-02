//! Structures that provide error information to clients of the plugin library.

pub static ERRORS: [&[u8]; 11] = [
    // 0 PLUGIN_OK
    b"Plugin OK\0",
    // 1 UNDEFINED_ERR
    b"Undefined error\0",
    // 2 PLUGIN_INIT_ERR,
    b"Plugin failed to initialize\0",
    // 3 ATTRIBUTE_DOES_NOT_EXIST
    b"Attribute does not exist\0",
    // 4 ATTRIBUTE_TYPE_MISMATCH
    b"Attribute types do not match\0",
    // 5 ATTRIBUTE_IS_NOT_SETTABLE
    b"Attribute cannot be set\0",
    // 6 IO_ERR
    b"IO operation failed\0",
    // 7 NUMERIC_CONVERSION_ERR
    b"Could not convert numeric value into a different type\0",
    // 8 NULL_PTR_ERR
    b"The plugin encountered a null pointer\0",
    // 9 CALLBACK_ERR
    b"The plugin attribute's callback failed\0",
    // 10 UPDATE_CACHED_VALUE_ERR
    b"Could not update plugin attribute's cached value\0",
];

pub mod constants {
    //! Constants that indicate specific error codes that a plugin can return.
    use libc::c_int;

    pub const PLUGIN_OK: c_int = 0;
    pub const UNDEFINED_ERR: c_int = 1;
    pub const PLUGIN_INIT_ERR: c_int = 2;
    pub const ATTRIBUTE_DOES_NOT_EXIST: c_int = 3;
    pub const ATTRIBUTE_TYPE_MISMATCH: c_int = 4;
    pub const ATTRIBUTE_IS_NOT_SETTABLE: c_int = 5;
    pub const IO_ERR: c_int = 6;
    pub const NUMERIC_CONVERSION_ERR: c_int = 7;
    pub const NULL_PTR_ERR: c_int = 8;
    pub const CALLBACK_ERR: c_int = 9;
    pub const UPDATE_CACHED_VALUE_ERR: c_int = 10;
}
