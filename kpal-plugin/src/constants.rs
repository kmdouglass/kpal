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
