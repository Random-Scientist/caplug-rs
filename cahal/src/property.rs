use std::ptr;

use crate::{OSResult, OSStatus};

/// This trait is marked unsafe because the [RawProperty::selector] function MUST always return the correct selector for its property data
pub unsafe trait RawProperty {
    /// Invariant: this function must always return the correct selector for this property
    fn selector(&self) -> u32;
    fn byte_size(&self) -> OSResult<u32>;
    fn is_mut(&self) -> bool;
    ///SAFETY: Implementation is responsible for ensuring the pointers passed in are valid
    unsafe fn set(&mut self, element: u32, data: *const [u8]) -> OSStatus;
    ///SAFETY: Implementation is responsible for ensuring the pointers passed in are valid
    unsafe fn get(&self, element: u32, data_out: *mut u8, data_len_out: *mut u32) -> OSStatus;
}

#[derive(Debug, Clone)]
pub struct TypedProperty<T, const SEL: u32, const MUTABLE_PROP: bool> {
    prop: T,
}
unsafe impl<T: Copy, const SEL: u32, const MUTABLE_PROP: bool> RawProperty
    for TypedProperty<T, SEL, MUTABLE_PROP>
{
    fn selector(&self) -> u32 {
        SEL
    }

    fn byte_size(&self) -> OSResult<u32> {
        std::mem::size_of::<T>().try_into()
    }

    fn is_mut(&self) -> bool {
        MUTABLE_PROP
    }

    unsafe fn set(&mut self, element: u32, data: *const u8, size: u32) -> OSStatus {
        assert_ne!(data, ptr::null());
        assert_eq!(size, self.byte_size());
        assert!(self.is_mut());
    }

    unsafe fn get(&self, element: u32, data_out: *mut u8, data_len_out: *mut u32) -> OSStatus {
        todo!()
    }
}
impl<T> From<Result<T, OSStatus>> for OSResult<T> {
    fn from(value: Result<T, OSStatus>) -> Self {
        match value {
            Err(s)
        }
    }
}
