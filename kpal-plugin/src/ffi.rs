//! Functions and types used by the foreign function interface to communicate with a plugin.
use std::boxed::Box;
use std::convert::TryInto;
use std::ptr::null;

use libc::{c_char, c_int, c_uchar, size_t};

use crate::error_codes::*;
use crate::{
    copy_string, PluginAPI, PluginData, PluginError, Val, ATTRIBUTE_PRE_INIT_FALSE,
    ATTRIBUTE_PRE_INIT_TRUE, ERRORS,
};

/// Determines which callbacks to use by indicating the current lifecycle phase of the plugin when
/// getting and setting attributes.
pub type Phase = c_int;

/// Frees the memory associated with the plugin's data.
///
/// This routine will be called automatically by the daemon and should not be called by any user
/// code.
///
/// # Arguments
///
/// * `plugin_data` - A pointer to a PluginData struct
pub extern "C" fn plugin_free(plugin_data: *mut PluginData) {
    if plugin_data.is_null() {
        return;
    }
    let plugin_data = plugin_data as *mut Box<PluginData>;
    unsafe {
        Box::from_raw(plugin_data);
    }
}

/// Initializes a plugin.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
///
/// # Arguments
///
/// * `plugin_data` - A pointer to a PluginData struct
pub unsafe extern "C" fn plugin_init<T: PluginAPI<E>, E: PluginError + 'static>(
    plugin_data: *mut PluginData,
) -> c_int {
    if plugin_data.is_null() {
        log::error!("plugin_data pointer is null");
        return NULL_PTR_ERR;
    };

    let plugin_data = plugin_data as *mut T;
    match (*plugin_data).init() {
        Ok(_) => {
            log::debug!("Successfully initialized plugin");
            PLUGIN_OK
        }
        Err(e) => {
            log::error!("Plugin failed to initialize: {}", e);
            e.error_code()
        }
    }
}

/// Returns an error message to the daemon given an error code.
///
/// If an undefined error code is provided, then this function will return a null pointer.
pub extern "C" fn error_message_ns(error_code: c_int) -> *const c_uchar {
    let error_code: size_t = match error_code.try_into() {
        Ok(error_code) => error_code,
        Err(_) => {
            log::error!("Unrecognized error code provided");
            return null();
        }
    };

    ERRORS.get(error_code).map_or(null(), |e| e.as_ptr())
}

/// Returns the number of attributes of the plugin.
///
/// This function returns the number of attributes rather than a status code. If this function is
/// provided with a null pointer as an argument, then zero will be returned.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
///
/// # Arguments
///
/// * `plugin_data` - A pointer to a PluginData struct
/// * `count` - A pointer to a size_t that will contain the number of attributes
pub unsafe extern "C" fn attribute_count<T: PluginAPI<E>, E: PluginError + 'static>(
    plugin_data: *const PluginData,
    count: *mut size_t,
) -> c_int {
    if plugin_data.is_null() {
        log::error!("plugin_data pointer is null");
        return NULL_PTR_ERR;
    };

    let plugin_data = plugin_data as *const T;
    *count = (*plugin_data).attribute_count();

    PLUGIN_OK
}

/// Writes the plugin's attribute IDs to a buffer that is provided by the caller.
///
/// This function returns a status code that indicates whether the operation succeeded and the
/// cause of any possible errors. It is recommended to check the status code returned by this
/// function before reading the contents of the buffer.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
///
/// # Arguments
/// * `plugin_data` - A pointer to a PluginData struct
/// * `buffer` - A pointer to a string of size_t's into which the attribute IDs will be written
/// * `length` - The length of the buffer
pub unsafe extern "C" fn attribute_ids<T: PluginAPI<E>, E: PluginError + 'static>(
    plugin_data: *const PluginData,
    buffer: *mut size_t,
    length: size_t,
) -> c_int {
    if plugin_data.is_null() {
        log::error!("plugin_data pointer is null");
        return NULL_PTR_ERR;
    }
    let plugin_data = plugin_data as *const T;
    let ids = (*plugin_data).attribute_ids();

    match copy_string(&ids, buffer, length) {
        Ok(_) => PLUGIN_OK,
        Err(_) => UNDEFINED_ERR,
    }
}

/// Writes the name of an attribute to a buffer that is provided by the caller.
///
/// This function returns a status code that indicates whether the operation succeeded and the
/// cause of any possible errors.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
///
/// # Arguments
///
/// * `plugin_data` - A pointer to a PluginData struct
/// * `id` - The id of the attribute
/// * `buffer` - A buffer of bytes into which the attribute's name will be written
/// * `length` - The length of the buffer
pub unsafe extern "C" fn attribute_name<T: PluginAPI<E>, E: PluginError + 'static>(
    plugin_data: *const PluginData,
    id: size_t,
    buffer: *mut c_uchar,
    length: size_t,
) -> c_int {
    if plugin_data.is_null() {
        log::error!("plugin_data pointer is null");
        return NULL_PTR_ERR;
    }
    let plugin_data = plugin_data as *const T;

    match (*plugin_data).attribute_name(id) {
        Ok(name) => copy_string(name.to_bytes_with_nul(), buffer, length)
            .map(|_| PLUGIN_OK)
            .unwrap_or_else(|_| UNDEFINED_ERR),
        Err(e) => e.error_code(),
    }
}

/// Indicates whether an attribute may be set before initialization.
///
/// This function accepts a pointer to a c_char. If the char is ATTRIBUTE_PRE_INIT_FALSE after the
/// function returns and it returns a value of PLUGIN_OK, then the attribute that corresponds to
/// the provided ID may not be set before plugin initialization. If the char is any value other
/// than 0 and the function returns PLUGIN_OK, then the plugin may be set before initialization.
///
/// If the function does not return PLUGIN_OK, then the value stored at pre_init will not be
/// modified.
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
///
/// # Arguments
///
/// * `plugin_data` - A pointer to a PluginData struct
/// * `id` - The id of the attribute
/// * `pre_init` - A value that determines whether the attribute's value may be set before the
/// plugin is initialized
pub unsafe extern "C" fn attribute_pre_init<T: PluginAPI<E>, E: PluginError + 'static>(
    plugin_data: *const PluginData,
    id: size_t,
    pre_init: *mut c_char,
) -> c_int {
    if plugin_data.is_null() {
        log::error!("plugin_data pointer is null");
        return NULL_PTR_ERR;
    }
    if pre_init.is_null() {
        log::error!("pre_init pointer is null");
        return NULL_PTR_ERR;
    }
    let plugin_data = plugin_data as *const T;

    match (*plugin_data).attribute_pre_init(id) {
        Ok(pre_init_resp) => {
            log::debug!(
                "Response for pre-init status of attribute {}: {}",
                id,
                pre_init_resp
            );
            if pre_init_resp {
                *pre_init = ATTRIBUTE_PRE_INIT_TRUE;
            } else {
                *pre_init = ATTRIBUTE_PRE_INIT_FALSE;
            };
            PLUGIN_OK
        }
        Err(e) => e.error_code(),
    }
}

/// Writes the value of an attribute to a Value instance that is provided by the caller.
///
/// This function returns a status code that indicates whether the operation succeeded and the
/// cause of any possible errors.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
///
/// # Arguments
///
/// * `plugin_data` - A pointer to a PluginData struct
/// * `id` - The id of the attribute
/// * `value` - A pointer to a Value enum. The enum is provided by this function's caller.
/// * `phase` - The phase of the plugin lifecycle. This determines what callbacks to use to read
/// the attribute value.
pub unsafe extern "C" fn attribute_value<T: PluginAPI<E>, E: PluginError + 'static>(
    plugin_data: *const PluginData,
    id: size_t,
    value: *mut Val,
    phase: Phase,
) -> c_int {
    if plugin_data.is_null() {
        log::error!("plugin_data pointer is null");
        return NULL_PTR_ERR;
    }
    let plugin_data = plugin_data as *const T;

    match (*plugin_data).attribute_value(id, phase) {
        Ok(new_value) => {
            log::debug!(
                "Response for the value of attribute {}: {:?}",
                id,
                new_value
            );
            *value = new_value
        }
        Err(e) => return e.error_code(),
    };

    PLUGIN_OK
}

/// Sets the value of an attribute.
///
/// This function returns a status code that indicates whether the operation succeeded and the
/// cause of any possible errors.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
///
/// # Arguments
///
/// * `plugin_data` - A pointer to a PluginData struct
/// * `id` - The id of the attribute
/// * `value` - A pointer to a Val enum. The enum is provided by this function's caller and will be
/// copied.
/// * `phase` - The phase of the plugin lifecycle. This determines what callbacks to use to read
/// the attribute value.
pub unsafe extern "C" fn set_attribute_value<T: PluginAPI<E>, E: PluginError + 'static>(
    plugin_data: *mut PluginData,
    id: size_t,
    value: *const Val,
    phase: Phase,
) -> c_int {
    if plugin_data.is_null() {
        log::error!("plugin_data pointer is null");
        return NULL_PTR_ERR;
    }
    if value.is_null() {
        log::error!("value pointer is null");
        return NULL_PTR_ERR;
    }
    let plugin_data = plugin_data as *mut T;

    match (*plugin_data).attribute_set_value(id, &*value, phase) {
        Ok(_) => {
            log::debug!("Set attribute {} to {:?}", id, *value);
            PLUGIN_OK
        }
        Err(e) => e.error_code(),
    }
}
