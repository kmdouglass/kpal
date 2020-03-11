//! The KPAL plugin crate provides tools to write your own KPAL plugins.
//!
//! See the examples folder for ideas on how to implement the datatypes and methods defined in this
//! library.
mod constants;
mod errors;
mod ffi;
mod strings;

use std::{
    cell::{Ref, RefCell},
    cmp::PartialEq,
    error::Error,
    ffi::{CStr, CString, FromBytesWithNulError},
    fmt, slice,
};

use libc::{c_char, c_double, c_int, c_uchar, c_uint, size_t};
pub use multi_map::{multimap, MultiMap};

pub use {
    constants::*,
    errors::error_codes,
    errors::{PluginUninitializedError, ERRORS},
    ffi::*,
    strings::copy_string,
};

/// The set of functions that must be implemented by a plugin.
pub trait PluginAPI<E: Error + PluginError + 'static>
where
    Self: Sized,
{
    /// Returns a new instance of the plugin. No initialization of the hardware is performed.
    fn new() -> Result<Self, E>;

    /// Initialzes the plugin by performing any hardware initialization.
    fn init(&mut self) -> Result<(), E>;

    /// Returns the attributes of the plugin.
    fn attributes(&self) -> &Attributes<Self, E>;

    /// Returns the number of attributes of the plugin.
    fn attribute_count(&self) -> usize {
        self.attributes().borrow().iter().count()
    }

    /// Returns the attribute IDs.
    fn attribute_ids(&self) -> Vec<usize> {
        self.attributes()
            .borrow()
            .iter()
            .map(|(id, _)| *id)
            .collect()
    }

    /// Returns the name of an attribute.
    ///
    /// If the attribute that corresponds to the `id` does not exist, then an error is
    /// returned. Otherwise, the name is returned as a C-compatible `&CStr`.
    ///
    /// # Arguments
    ///
    /// * `id` - the numeric ID of the attribute
    fn attribute_name(&self, id: usize) -> Result<Ref<CString>, E> {
        log::debug!("Received request for the name of attribute: {}", id);
        let attributes = self.attributes().borrow();
        match attributes.get(&id) {
            Some(_) => Ok(Ref::map(attributes, |a| {
                &a.get(&id)
                    .expect("Attribute does not exist. This should never happen.")
                    .name
            })),
            None => Err(E::new(error_codes::ATTRIBUTE_DOES_NOT_EXIST)),
        }
    }

    /// Indicates whether an attribute may be set before initialization.
    ///
    /// # Arguments
    ///
    /// # `id` - the numeric ID of the attribute
    fn attribute_pre_init(&self, id: usize) -> Result<bool, E> {
        log::debug!(
            "Received request for attribute pre-initialzation status: {}",
            id
        );
        let attributes = self.attributes();
        let attributes = attributes.borrow();
        let attribute = attributes
            .get(&id)
            .ok_or_else(|| E::new(error_codes::ATTRIBUTE_DOES_NOT_EXIST))?;

        match attribute.callbacks_init {
            Callbacks::Update => Ok(true),
            _ => Ok(false),
        }
    }

    /// Returns the value of an attribute.
    ///
    /// If the attribute that corresponds to the `id` does not exist, then an error is
    /// returned. Otherwise, the value is returnd as a C-compatible tagged enum.
    ///
    /// # Arguments
    ///
    /// * `id` - the numeric ID of the attribute
    /// * `phase` - the lifecycle phase of the plugin that determines which callbacks to use
    fn attribute_value(&self, id: usize, phase: Phase) -> Result<Val, E> {
        log::debug!("Received request for the value of attribute: {}", id);
        let attributes = self.attributes();
        let mut attributes = attributes.borrow_mut();
        let attribute = attributes
            .get_mut(&id)
            .ok_or_else(|| E::new(error_codes::ATTRIBUTE_DOES_NOT_EXIST))?;

        let get = if phase == constants::INIT_PHASE {
            match attribute.callbacks_init {
                Callbacks::Constant => return Ok(attribute.value.as_val()),
                Callbacks::Update => return Ok(attribute.value.as_val()),
                Callbacks::Get(get) => get,
                Callbacks::GetAndSet(get, _) => get,
            }
        } else if phase == constants::RUN_PHASE {
            match attribute.callbacks_run {
                Callbacks::Constant => return Ok(attribute.value.as_val()),
                Callbacks::Update => return Ok(attribute.value.as_val()),
                Callbacks::Get(get) => get,
                Callbacks::GetAndSet(get, _) => get,
            }
        } else {
            return Err(E::new(error_codes::LIFECYCLE_PHASE_ERR));
        };

        let value = get(&self, &attribute.value).map_err(|err| {
            log::error!("Callback error {{ id: {:?}, error: {:?} }}", id, err);
            E::new(error_codes::CALLBACK_ERR)
        })?;

        // Update the attribute's cached value.
        attribute.value = value;

        Ok(attribute.value.as_val())
    }

    /// Sets the value of the attribute given by the id.
    ///
    /// If the attribute that corresponds to the `id` does not exist, or if the attribute cannot be
    /// set, then an error is returned.
    ///
    /// # Arguments
    ///
    /// * `id` - the numeric ID of the attribute
    /// * `val` - a reference to a Val instance containing the attribute's new value
    /// * `phase` - the lifecycle phase of the plugin that determines which callbacks to use
    fn attribute_set_value(&self, id: usize, val: &Val, phase: Phase) -> Result<(), E> {
        log::debug!("Received request to set the value of attribute: {}", id);
        let attributes = self.attributes();
        let mut attributes = attributes.borrow_mut();
        let attribute = attributes
            .get_mut(&id)
            .ok_or_else(|| E::new(error_codes::ATTRIBUTE_DOES_NOT_EXIST))?;

        let option_set = if phase == constants::INIT_PHASE {
            match attribute.callbacks_init {
                Callbacks::Update => None,
                Callbacks::GetAndSet(_, set) => Some(set),
                _ => return Err(E::new(error_codes::ATTRIBUTE_IS_NOT_SETTABLE)),
            }
        } else if phase == constants::RUN_PHASE {
            match attribute.callbacks_run {
                Callbacks::Update => None,
                Callbacks::GetAndSet(_, set) => Some(set),
                _ => return Err(E::new(error_codes::ATTRIBUTE_IS_NOT_SETTABLE)),
            }
        } else {
            return Err(E::new(error_codes::LIFECYCLE_PHASE_ERR));
        };

        // Call the set callback on the attribute's values.
        if let Some(set) = option_set {
            let result = set_helper(self, &attribute.value, val, set);

            result.map_err(|err| {
                log::error!("Callback error {{ id: {:?}, error: {:?} }}", id, err);
                E::new(error_codes::CALLBACK_ERR)
            })?;
        };

        // Update the attribute's cached value.
        attribute.value = val.to_value().map_err(|err| {
            log::error!(
                "Could not update plugin attribute's cached value: {{ id: {:?}, error: {:?} }}",
                id,
                err
            );
            E::new(error_codes::UPDATE_CACHED_VALUE_ERR)
        })?;

        Ok(())
    }
}

/// Convenience function that calls a set callback only for valid (Value, Val) pairs.
///
/// # Arguments
///
/// * `plugin` - A reference to the plugin object that contains the plugins data
/// * `value` - A reference to an attribute's current value
/// * `val` - A reference to the attribute's desired, new value
/// * `set` - The callback function that will perform the value update
fn set_helper<T, E: Error + PluginError + 'static>(
    plugin: &T,
    value: &Value,
    val: &Val,
    set: fn(&T, &Value, &Val) -> Result<(), E>,
) -> Result<(), E> {
    let err = Err(E::new(error_codes::ATTRIBUTE_TYPE_MISMATCH));

    // The full range is listed so the compiler will remind you to update this function when a new
    // variant is added.
    match (value, val) {
        (Value::Int(_), Val::Int(_))
        | (Value::Double(_), Val::Double(_))
        | (Value::String(_), Val::String(_, _))
        | (Value::Uint(_), Val::Uint(_)) => set(plugin, value, val),
        // Invalid inputs
        (Value::Int(_), Val::Double(_)) => err,
        (Value::Int(_), Val::String(_, _)) => err,
        (Value::Int(_), Val::Uint(_)) => err,
        (Value::Double(_), Val::Int(_)) => err,
        (Value::Double(_), Val::String(_, _)) => err,
        (Value::Double(_), Val::Uint(_)) => err,
        (Value::String(_), Val::Int(_)) => err,
        (Value::String(_), Val::Double(_)) => err,
        (Value::String(_), Val::Uint(_)) => err,
        (Value::Uint(_), Val::Int(_)) => err,
        (Value::Uint(_), Val::Double(_)) => err,
        (Value::Uint(_), Val::String(_, _)) => err,
    }
}

/// The set of functions that must be implemented by a plugin library's main error type.
pub trait PluginError: std::error::Error {
    /// Initializes and returns a new instace of the error type.
    ///
    /// # Arguments
    ///
    /// * `error_code` - One of the integer error codes recognized by KPAL.
    fn new(error_code: c_int) -> Self;

    /// Returns the error code of the instance.
    fn error_code(&self) -> c_int;
}

/// A Plugin combines the data that determines its state and with its functionality.
///
/// This struct holds a raw pointer to a data struct that is created by the plugin library. In
/// addition, it contains the vtable of function pointers defined by the C API and implemented
/// within the plugin library.
///
/// # Safety
///
/// The plugin implements the `Send` trait because after creation the plugin is moved into the
/// thread that is dedicated to the plugin that it manages. Once it is moved, it will only ever be
/// owned and used by this single thread by design.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct Plugin {
    /// A pointer to a struct containing the state of the plugin.
    pub plugin_data: *mut PluginData,

    /// The table of function pointers that define part of the plugin API.
    pub vtable: VTable,
}

impl Drop for Plugin {
    /// Frees the memory allocated to the plugin data.
    fn drop(&mut self) {
        (self.vtable.plugin_free)(self.plugin_data);
    }
}

unsafe impl Send for Plugin {}

/// An opaque struct that contains the state of an individual plugin.
///
/// The daemon does not actually work directly with structs provided by a plugin library. Instead,
/// they are hidden behind pointers to opaque structs of this type. The kpal-plugin FFI code takes
/// care of casting the pointers back into the appropriate type inside the library code.
///
/// # Notes
///
/// In Rust, an opaque struct is defined as a struct with a field that is a zero-length array of
/// unsigned 8-bit integers. It is used to hide the plugin's state, forcing all interactions
/// with the data through the functions in the vtable instead.
#[derive(Debug)]
#[repr(C)]
pub struct PluginData {
    _private: [u8; 0],
}

/// A table of function pointers that comprise the plugin API for the foreign function interface.
///
/// By default, functions in the VTable return a number that represents a status code that maps
/// onto a particular reason for an error. All functions should use the same mapping between status
/// code and reason.
///
/// Functions that return values that do not represent status codes have names that end in the
/// characters '_ns' that stand for "no status."
#[derive(Clone, Debug)]
#[repr(C)]
pub struct VTable {
    /// Frees the memory associated with a plugin's data.
    pub plugin_free: extern "C" fn(*mut PluginData),

    /// Initializes a plugin.
    ///
    /// This method is distinct from the `kpal_plugin_new` FFI call in that it actually
    /// communicates with the hardware, whereas `kpal_plugin_new` is used merely to create the
    /// plugin data structures.
    pub plugin_init: unsafe extern "C" fn(*mut PluginData) -> c_int,

    /// Returns an error message associated with a Plugin error code.
    pub error_message_ns: extern "C" fn(c_int) -> *const c_uchar,

    /// Returns the number of attributes of the plugin.
    pub attribute_count:
        unsafe extern "C" fn(plugin_data: *const PluginData, count: *mut size_t) -> c_int,

    /// Returns the attribute IDs in a buffer provided by the caller.
    pub attribute_ids:
        unsafe extern "C" fn(plugin_data: *const PluginData, ids: *mut size_t, size_t) -> c_int,

    /// Writes the name of an attribute to a buffer that is provided by the caller.
    pub attribute_name: unsafe extern "C" fn(
        plugin_data: *const PluginData,
        id: size_t,
        buffer: *mut c_uchar,
        length: size_t,
    ) -> c_int,

    /// Indicates whether an attribute may be set before initialization.
    pub attribute_pre_init: unsafe extern "C" fn(
        plugin_data: *const PluginData,
        id: size_t,
        pre_init: *mut c_char,
    ) -> c_int,

    /// Writes the value of an attribute to a Value instance that is provided by the caller.
    pub attribute_value: unsafe extern "C" fn(
        plugin_data: *const PluginData,
        id: size_t,
        value: *mut Val,
        phase: Phase,
    ) -> c_int,

    /// Sets the value of an attribute.
    pub set_attribute_value: unsafe extern "C" fn(
        plugin_data: *mut PluginData,
        id: size_t,
        value: *const Val,
        phase: Phase,
    ) -> c_int,
}

/// The type signature of the function that returns a new plugin instance.
pub type KpalPluginInit = unsafe extern "C" fn(*mut Plugin) -> c_int;

/// The type signature of the function that initializes a library.
pub type KpalLibraryInit = unsafe extern "C" fn() -> c_int;

/// The type signature of the collection of attributes that is owned by the plugin.
pub type Attributes<T, E> = RefCell<MultiMap<usize, &'static str, Attribute<T, E>>>;

/// A single piece of information that partly determines the state of a plugin.
#[derive(Debug)]
#[repr(C)]
pub struct Attribute<T, E: Error + PluginError> {
    /// The name of the attribute.
    pub name: CString,

    /// The value of the attribute.
    ///
    /// This field may be used to cache values retrieved from the hardware. This is the initial
    /// value of non-constant attributes.
    pub value: Value,

    /// The callback functions that are fired when the attribute is either read or set during the
    /// init phase of the plugin.
    pub callbacks_init: Callbacks<T, E>,

    /// The callback functions that are fired when the attribute is either read or set during the
    /// run phase of the plugin.
    pub callbacks_run: Callbacks<T, E>,
}

/// An owned value of an attribute.
///
/// Unlike the `Val` enum, these are intended to be owned by an instance of a PluginData struct and
/// do not pass through the FFI.
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub enum Value {
    Int(c_int),
    Double(c_double),
    String(CString),
    Uint(c_uint),
}

impl Value {
    /// Returns a reference type to a Value.
    ///
    /// as_val creates a new Val instance from a Value. Value variants that contain datatypes that
    /// implement Copy are copied into the new Val instance. For complex datatypes that are not
    /// Copy, pointers to the data are embedded inside the Val instance instead.
    ///
    /// This method is used to generate datatypes that represent attribute values and that may pass
    /// through the FFI.
    pub fn as_val(&self) -> Val {
        match self {
            Value::Int(value) => Val::Int(*value),
            Value::Double(value) => Val::Double(*value),
            Value::String(value) => {
                let slice = value.as_bytes_with_nul();
                Val::String(slice.as_ptr(), slice.len())
            }
            Value::Uint(value) => Val::Uint(*value),
        }
    }
}

/// A wrapper type for transporting Values through the plugin API.
///
/// Unlike the `Value` enum, this type is intended to be sent through the FFI. Because of this, the
/// enum variants can only contain C-compatible datatypes.
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub enum Val {
    Int(c_int),
    Double(c_double),
    String(*const c_uchar, size_t),
    Uint(c_uint),
}

impl Val {
    /// Clones the data inside a Val into a new Value type.
    ///
    /// This method is used to convert Vals, which pass through the FFI, into owned Value
    /// datatypes. Wrapped data that is not Copy is necessarily cloned when the new Value instance
    /// is created.
    pub fn to_value(&self) -> Result<Value, ValueConversionError> {
        match self {
            Val::Int(value) => Ok(Value::Int(*value)),
            Val::Double(value) => Ok(Value::Double(*value)),
            Val::String(p_value, length) => {
                let slice = unsafe { slice::from_raw_parts(*p_value, *length) };
                let c_string = CStr::from_bytes_with_nul(slice)?.to_owned();
                Ok(Value::String(c_string))
            }
            Val::Uint(value) => Ok(Value::Uint(*value)),
        }
    }
}

/// Callback functions that communicate with the hardware when an attribute is read or set.
///
/// The purpose of a callback is two-fold: it performs the actual communication with the hardware
/// and/or it modifies the plugin's cached attribute data.
///
/// If the attribute is constant and never changes its original value, then the `Constant` variant
/// should be used. If the attribute's value changes without user input (e.g. a sensor reading) but
/// cannot be set, then use the `Get` variant. Otherwise, for attributes that can be both read and
/// set, use the `GetAndSet` variant.
///
/// The Update variant is used to set only the cached value of attributes. Attributes that are
/// Update always return their cached value when the attribute's value is read.
#[repr(C)]
pub enum Callbacks<T, E: Error + PluginError> {
    Constant,
    Get(fn(plugin: &T, cached: &Value) -> Result<Value, E>),
    GetAndSet(
        fn(plugin: &T, cached: &Value) -> Result<Value, E>,
        fn(plugin: &T, cached: &Value, value: &Val) -> Result<(), E>,
    ),
    Update,
}

impl<T, E: Error + PluginError> fmt::Debug for Callbacks<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Callbacks::*;
        match *self {
            Constant => write!(f, "Constant"),
            Get(get) => write!(f, "Get Callback: {:x}", get as usize),
            GetAndSet(get, set) => write!(
                f,
                "Get Callback: {:x}, Set Callback: {:x}",
                get as usize, set as usize
            ),
            Update => write!(f, "Update"),
        }
    }
}

/// Creates the required symbols for a plugin library.
///
/// Any plugin library must call this macro exactly once to generate the symbols that are required
/// by the daemon.
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $plugin_err_type:ty) => {
        /// Initializes the library.                                                                                                                                                                                      
        ///                                                                                                                                                                                                               
        /// This function is called only once by the daemon. It is called when a library is first
        /// loaded into memory.
        #[no_mangle]
        pub extern "C" fn kpal_library_init() -> c_int {
            env_logger::init();
            PLUGIN_OK
        }

        /// Returns a new Plugin instance containing the plugin data and the function vtable.
        ///
        /// The plugin is used by the daemon to communicate with it. It contains an opaque pointer
        /// to the plugin data and a vtable. The vtable is a struct of function pointers to the
        /// methods in the plugin API.
        ///
        /// # Safety
        ///
        /// This function is unsafe because it dereferences a null pointer and assigns data to a
        /// variable of the type `MaybeUnit`.
        #[no_mangle]
        pub unsafe extern "C" fn kpal_plugin_new(plugin: *mut Plugin) -> c_int {
            let plugin_data = match <$plugin_type>::new() {
                Ok(plugin_data) => plugin_data,
                Err(e) => {
                    log::error!("Failed to initialize the plugin: {:?}", e);
                    return PLUGIN_INIT_ERR;
                }
            };

            let plugin_data: Box<$plugin_type> = Box::new(plugin_data);
            let plugin_data = Box::into_raw(plugin_data) as *mut PluginData;

            let vtable = VTable {
                plugin_free,
                plugin_init: plugin_init::<$plugin_type, $plugin_err_type>,
                error_message_ns,
                attribute_count: attribute_count::<$plugin_type, $plugin_err_type>,
                attribute_ids: attribute_ids::<$plugin_type, $plugin_err_type>,
                attribute_name: attribute_name::<$plugin_type, $plugin_err_type>,
                attribute_pre_init: attribute_pre_init::<$plugin_type, $plugin_err_type>,
                attribute_value: attribute_value::<$plugin_type, $plugin_err_type>,
                set_attribute_value: set_attribute_value::<$plugin_type, $plugin_err_type>,
            };

            plugin.write(Plugin {
                plugin_data,
                vtable,
            });

            log::debug!("Created new plugin: {:?}", plugin);
            PLUGIN_OK
        }
    };
}

/// An error type that represents a failure to convert a Val to a Value.
#[derive(Debug)]
pub struct ValueConversionError {
    side: FromBytesWithNulError,
}

impl Error for ValueConversionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.side)
    }
}

impl fmt::Display for ValueConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PluginError: {:?}", self)
    }
}

impl From<FromBytesWithNulError> for ValueConversionError {
    fn from(error: FromBytesWithNulError) -> Self {
        ValueConversionError { side: error }
    }
}
