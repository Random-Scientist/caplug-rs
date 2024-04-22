use std::{collections::HashMap, ptr};

use crate::{OSResult, OSStatus, OSStatusError, ResultExt};

#[derive(Debug, Clone, Copy)]
pub struct PropertySelector(u32);

impl From<u32> for PropertySelector {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// This trait is marked unsafe because the [RawProperty::selector] function MUST always return the correct selector for its property data
pub unsafe trait RawProperty {
    /// Invariant: this function must always return the correct selector for this property
    fn selector(&self) -> u32;
    fn byte_size(&self) -> OSResult<u32>;
    fn is_mut(&self) -> bool;
    ///SAFETY: Implementation is responsible for ensuring the pointers passed in are valid
    unsafe fn set(&mut self, element: u32, data: *const u8, data_size: u32) -> OSStatus;
    ///SAFETY: Implementation is responsible for ensuring the pointers passed in are valid
    unsafe fn get(&self, element: u32, data_out: *mut u8, data_len_out: *mut u32) -> OSStatus;
}

pub trait AudioObject {
    fn properties(&self) -> HashMap<PropertySelector, Box<dyn RawProperty>>;
}

/// Note that T must be FFI-safe in order for this structure to be sound
#[derive(Debug, Clone)]
#[repr(C)]
pub struct SingularTypedProperty<T, const SEL: u32, const MUTABLE_PROP: bool> {
    prop: T,
}
impl<T: Copy, const SEL: u32, const MUTABLE_PROP: bool>
    SingularTypedProperty<T, SEL, MUTABLE_PROP>
{
    /// **SAFETY:** the caller must ensure that T is a valid type with the C ABI of the corresponding selector
    pub unsafe fn new(val: T) -> Self {
        Self { prop: val }
    }
}
unsafe impl<T: Copy, const SEL: u32, const MUTABLE_PROP: bool> RawProperty
    for SingularTypedProperty<T, SEL, MUTABLE_PROP>
{
    fn selector(&self) -> u32 {
        SEL
    }

    fn byte_size(&self) -> OSResult<u32> {
        std::mem::size_of::<T>()
            .try_into()
            .replace_err(OSStatusError::HW_BAD_PROPERTY_SIZE_ERR)
    }

    fn is_mut(&self) -> bool {
        MUTABLE_PROP
    }

    unsafe fn set(&mut self, _element: u32, data: *const u8, data_size: u32) -> OSStatus {
        assert_ne!(data, ptr::null());
        assert_eq!(data_size, self.byte_size()?);
        assert!(self.is_mut());
        todo!()
    }

    unsafe fn get(&self, _element: u32, _data_out: *mut u8, _data_len_out: *mut u32) -> OSStatus {
        todo!()
    }
}
