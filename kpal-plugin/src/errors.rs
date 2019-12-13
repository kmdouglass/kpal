//! Structures that provide error information to clients of the plugin library.

pub static ERRORS: [&[u8]; 8] = [
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
    // 5 IO_ERR
    b"IO operation failed\0",
    // 6 NUMERIC_CONVERSION_ERR
    b"Could not convert numeric value into a different type\0",
    // 7 NULL_PTR_ERR
    b"The plugin encountered a null pointer\0",
];

pub mod constants {
    use libc::c_int;

    pub const PLUGIN_OK: c_int = 0;
    pub const UNDEFINED_ERR: c_int = 1;
    pub const PLUGIN_INIT_ERR: c_int = 2;
    pub const ATTRIBUTE_DOES_NOT_EXIST: c_int = 3;
    pub const ATTRIBUTE_TYPE_MISMATCH: c_int = 4;
    pub const IO_ERR: c_int = 5;
    pub const NUMERIC_CONVERSION_ERR: c_int = 6;
    pub const NULL_PTR_ERR: c_int = 7;
}
