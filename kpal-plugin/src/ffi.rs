//! Functions used by the foreign function interface to communicate with a plugin.
use std::boxed::Box;
use std::convert::TryInto;
use std::ptr::null;

use libc::{c_int, c_uchar, size_t};

use crate::constants::*;
use crate::{copy_string, PluginAPI, PluginData, PluginError, Val, ERRORS};

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

/// Returns an error message to the daemon given an error code.
///
/// If an undefined error code is provided, then this function will return a null pointer.
pub extern "C" fn error_message(error_code: c_int) -> *const c_uchar {
    let error_code: size_t = match error_code.try_into() {
        Ok(error_code) => error_code,
        Err(_) => {
            log::error!("Unrecognized error code provided");
            return null();
        }
    };

    ERRORS.get(error_code).map_or(null(), |e| e.as_ptr())
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
pub unsafe extern "C" fn attribute_value<T: PluginAPI<E>, E: PluginError + 'static>(
    plugin_data: *const PluginData,
    id: size_t,
    value: *mut Val,
) -> c_int {
    if plugin_data.is_null() {
        log::error!("plugin_data pointer is null");
        return NULL_PTR_ERR;
    }
    let plugin_data = plugin_data as *const T;

    match (*plugin_data).attribute_value(id) {
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
pub unsafe extern "C" fn set_attribute_value<T: PluginAPI<E>, E: PluginError + 'static>(
    plugin_data: *mut PluginData,
    id: size_t,
    value: *const Val,
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

    match (*plugin_data).attribute_set_value(id, &*value) {
        Ok(_) => {
            log::debug!("Set attribute {} to {:?}", id, *value);
            PLUGIN_OK
        }
        Err(e) => e.error_code(),
    }
}
