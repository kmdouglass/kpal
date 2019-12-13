//! Structures that provide error information to clients of the plugin library.

pub static ERRORS: [&[u8]; 7] = [
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
];
