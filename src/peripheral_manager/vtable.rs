use std::io::Result;

use libc::{c_char, c_int, c_void, size_t};
use libloading::os::unix::Symbol as RawSymbol;
use libloading::{Library, Symbol};

// TODO Change c_void to an opaque pointer
type PeripheralFree = extern "C" fn(*mut c_void);
type PeripheralNew = extern "C" fn() -> *mut c_void;
type PropertyName = extern "C" fn(*const c_void, size_t) -> *const c_char;
type PropertySetValue = extern "C" fn(*const c_void, size_t, *const c_void) -> c_int;
type PropertyValue = extern "C" fn(*const c_void, size_t, *mut c_void) -> c_int;

pub struct VTable {
    pub peripheral_free: RawSymbol<PeripheralFree>,
    pub peripheral_new: RawSymbol<PeripheralNew>,
    pub property_name: RawSymbol<PropertyName>,
    pub property_set_value: RawSymbol<PropertySetValue>,
    pub property_value: RawSymbol<PropertyValue>,
}

impl VTable {
    pub unsafe fn new(library: &Library) -> Result<VTable> {
        let peripheral_free: Symbol<PeripheralFree> = library.get(b"peripheral_free\0")?;
        let peripheral_free = peripheral_free.into_raw();
        let peripheral_new: Symbol<PeripheralNew> = library.get(b"peripheral_new\0")?;
        let peripheral_new = peripheral_new.into_raw();
        let property_name: Symbol<PropertyName> = library.get(b"property_name\0")?;
        let property_name = property_name.into_raw();
        let property_value: Symbol<PropertyValue> = library.get(b"property_value\0")?;
        let property_value = property_value.into_raw();
        let property_set_value: Symbol<PropertySetValue> = library.get(b"property_set_value\0")?;
        let property_set_value = property_set_value.into_raw();

        let vtable = VTable {
            peripheral_free,
            peripheral_new,
            property_name,
            property_set_value,
            property_value,
        };

        Ok(vtable)
    }
}
