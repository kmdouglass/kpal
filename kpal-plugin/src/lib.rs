//! The KPAL plugin crate provides tools to write your own KPAL plugins.
//!
//! See the examples folder for ideas on how to implement the datatypes and methods defined in this
//! library.
mod errors;
mod ffi;
mod strings;

use std::{
    cell::{Ref, RefCell},
    cmp::PartialEq,
    error::Error,
    ffi::CString,
    fmt,
};

use libc::{c_double, c_int, c_long, c_uchar, size_t};

pub use errors::constants;
pub use errors::ERRORS;
pub use ffi::*;
pub use strings::copy_string;

/// The set of functions that must be implemented by a plugin.
pub trait PluginAPI<E: Error + PluginError + 'static>
where
    Self: Sized,
{
    /// Initializes and returns a new instance of the plugin.
    fn new() -> Result<Self, E>;

    /// Returns the attributes of the plugin.
    fn attributes(&self) -> &Attributes<Self, E>;

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
        match attributes.get(id) {
            Some(_) => Ok(Ref::map(attributes, |a| &a[id].name)),
            None => Err(E::new(constants::ATTRIBUTE_DOES_NOT_EXIST)),
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
    fn attribute_value(&self, id: usize) -> Result<Value, E> {
        log::debug!("Received request for the value of attribute: {}", id);
        let attributes = self.attributes();
        let mut attributes = attributes.borrow_mut();
        let attribute = attributes
            .get_mut(id)
            .ok_or_else(|| E::new(constants::ATTRIBUTE_DOES_NOT_EXIST))?;

        let get = match attribute.callbacks {
            Callbacks::Constant => return Ok(attribute.value.clone()),
            Callbacks::Get(get) => get,
            Callbacks::GetAndSet(get, _) => get,
        };

        get(&self, &mut attribute.value)
    }

    /// Sets the value of the attribute given by the id.
    ///
    /// If the attribute that corresponds to the `id` does not exist, or if the attribute cannot be
    /// set, then an error is returned.
    ///
    /// # Arguments
    ///
    /// * `id` - the numeric ID of the attribute
    /// * `value` - a reference to a value
    fn attribute_set_value(&self, id: usize, value: &Value) -> Result<(), E> {
        log::debug!("Received request to set the value of attribute: {}", id);
        use Value::*;
        let attributes = self.attributes();
        let mut attributes = attributes.borrow_mut();
        let attribute = attributes
            .get_mut(id)
            .ok_or_else(|| E::new(constants::ATTRIBUTE_DOES_NOT_EXIST))?;

        let set = match attribute.callbacks {
            Callbacks::GetAndSet(_, set) => set,
            _ => return Err(E::new(constants::ATTRIBUTE_IS_NOT_SETTABLE)),
        };

        match (&attribute.value, &value) {
            (Int(_), Int(_)) | (Float(_), Float(_)) => set(&self, &mut attribute.value, value),
            _ => Err(E::new(constants::ATTRIBUTE_TYPE_MISMATCH)),
        }
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
#[derive(Clone, Debug)]
#[repr(C)]
pub struct VTable {
    /// Frees the memory associated with a plugin's data.
    pub plugin_free: extern "C" fn(*mut PluginData),

    /// Returns an error message associated with a Plugin error code.
    pub error_message: extern "C" fn(c_int) -> *const c_uchar,

    /// Writes the name of an attribute to a buffer that is provided by the caller.
    pub attribute_name: unsafe extern "C" fn(
        plugin_data: *const PluginData,
        id: size_t,
        buffer: *mut c_uchar,
        length: size_t,
    ) -> c_int,

    /// Writes the value of an attribute to a Value instance that is provided by the caller.
    pub attribute_value: unsafe extern "C" fn(
        plugin_data: *const PluginData,
        id: size_t,
        value: *mut Value,
    ) -> c_int,

    /// Sets the value of an attribute.
    pub set_attribute_value: unsafe extern "C" fn(
        plugin_data: *mut PluginData,
        id: size_t,
        value: *const Value,
    ) -> c_int,
}

/// The type signature of the function that returns a new plugin instance.
pub type KpalPluginInit = unsafe extern "C" fn(*mut Plugin) -> c_int;

/// The type signature of the function that initializes a library.
pub type KpalLibraryInit = unsafe extern "C" fn() -> c_int;

/// The type signature of the collection of attributes that is owned by the plugin.
pub type Attributes<T, E> = RefCell<Vec<Attribute<T, E>>>;

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

    /// The callback functions that are fired when the attribute is either read or set.
    pub callbacks: Callbacks<T, E>,
}

/// The value of an attribute.
///
/// Currently only integer and floating point (decimal) values are supported.
#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub enum Value {
    Int(c_long),
    Float(c_double),
}

/// Callback functions that communicate with the hardware when an attribute is read or set.
///
/// The purpose of a callback is to perform the actual communication with the hardware. If the
/// attribute is constant and never changes its original value, then the `Constant` variant should
/// be used. If the attribute's value changes without user input (e.g. a sensor reading) but cannot
/// be set, then use the `Get` variant. Otherwise, for attributes that can be both read and set,
/// use the `GetAndSet` variant.
#[repr(C)]
pub enum Callbacks<T, E: Error + PluginError> {
    Constant,
    Get(fn(plugin: &T, cached: &mut Value) -> Result<Value, E>),
    GetAndSet(
        fn(plugin: &T, cached: &mut Value) -> Result<Value, E>,
        fn(plugin: &T, cached: &mut Value, value: &Value) -> Result<(), E>,
    ),
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
        pub unsafe extern "C" fn kpal_plugin_init(plugin: *mut Plugin) -> c_int {
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
                error_message,
                attribute_name: attribute_name::<$plugin_type, $plugin_err_type>,
                attribute_value: attribute_value::<$plugin_type, $plugin_err_type>,
                set_attribute_value: set_attribute_value::<$plugin_type, $plugin_err_type>,
            };

            plugin.write(Plugin {
                plugin_data,
                vtable,
            });

            log::debug!("Initialized plugin: {:?}", plugin);
            PLUGIN_OK
        }
    };
}
