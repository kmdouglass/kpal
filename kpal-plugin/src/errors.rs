//! Structures that provide error information to clients of the plugin library.

pub static ERRORS: [&[u8]; 4] = [
    // 0 PLUGIN_OK
    b"Plugin OK\0",
    // 1 UNDEFINED_ERR
    b"Undefined error\0",
    // 2 ATTRIBUTE_DOES_NOT_EXIST
    b"Attribute does not exist\0",
    // 3 ATTRIBUTE_TYPE_MISMATCH
    b"Attribute types do not match\0",
];
