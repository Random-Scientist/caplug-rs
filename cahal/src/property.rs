use core::slice;
use std::{any::Any, ffi::c_void, ptr};

use core_foundation::{
    base::{FromMutVoid, FromVoid},
    string::CFString,
};

use crate::{ret_assert, OSResult, OSStatus, OSStatusError, ResultExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PropertySelector(u32);

impl From<u32> for PropertySelector {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

pub trait RawProperty {
    /// Invariant: this function must always return the correct selector for this property or bad things will happen
    fn selector(&self) -> PropertySelector;
    /// Total size in bytes this type occupies
    fn byte_size(&self) -> OSResult<u32>;
    /// Whether to advertise this property as settable or not
    fn is_mut(&self) -> bool;
    /// Utility function for reading this property's value from Rust
    fn as_any(&self) -> &dyn Any;
    /// Utility function for mutating this property's value from Rust
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// Read a value of the type of the property of this implementation from the `data` pointer and write it to the internal storage
    unsafe fn set(&mut self, data: *const c_void, data_size: u32) -> OSStatus;
    /// Write a value stored in this implementation to the allocation at `data_out`
    unsafe fn get(
        &self,
        out_alloc_size: u32,
        data_out: *mut c_void,
        data_len_out: *mut u32,
    ) -> OSStatus;
}

#[derive(Debug, Clone)]
pub struct Prop<T, const SEL: u32, const MUTABLE_PROP: bool> {
    prop: T,
}

impl<T: Copy, const SEL: u32, const MUTABLE_PROP: bool> Prop<T, SEL, MUTABLE_PROP> {
    pub fn new(val: T) -> Self {
        Self { prop: val }
    }
}
//SAFETY: selector invariants and
impl<T: Copy + 'static, const SEL: u32, const MUTABLE_PROP: bool> RawProperty
    for Prop<T, SEL, MUTABLE_PROP>
{
    fn selector(&self) -> PropertySelector {
        SEL.into()
    }

    fn byte_size(&self) -> OSResult<u32> {
        std::mem::size_of::<T>()
            .try_into()
            .replace_err(OSStatusError::HW_BAD_PROPERTY_SIZE_ERR)
    }

    fn is_mut(&self) -> bool {
        MUTABLE_PROP
    }

    unsafe fn set(&mut self, data: *const c_void, data_size: u32) -> OSStatus {
        ret_assert!(data != ptr::null(), OSStatusError::HW_ILLEGAL_OPERATION_ERR);
        ret_assert!(
            data_size == self.byte_size()?,
            OSStatusError::HW_BAD_PROPERTY_SIZE_ERR
        );
        ret_assert!(
            data as usize % self.byte_size()? as usize == 0,
            OSStatusError::HW_BAD_OBJECT_ERR
        );
        ret_assert!(self.is_mut());

        let val = ptr::read(data as *const T);

        self.prop = val;
        Ok(())
    }

    unsafe fn get(
        &self,
        out_alloc_size: u32,
        data_out: *mut c_void,
        data_len_out: *mut u32,
    ) -> OSStatus {
        ret_assert!(
            data_out != ptr::null_mut() && data_len_out != ptr::null_mut(),
            OSStatusError::HW_ILLEGAL_OPERATION_ERR
        );
        ret_assert!(
            out_alloc_size >= self.byte_size()?,
            OSStatusError::HW_BAD_PROPERTY_SIZE_ERR
        );
        ret_assert!(
            data_out as usize % self.byte_size()? as usize == 0,
            OSStatusError::HW_BAD_OBJECT_ERR
        );
        ptr::write(data_out as *mut T, self.prop.clone());
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        &self.prop
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.prop
    }
}

#[derive(Debug, Clone)]
pub struct ArrayProp<T, const SEL: u32, const MUTABLE_PROP: bool> {
    props: Vec<T>,
}
impl<T, const SEL: u32, const MUTABLE_PROP: bool> ArrayProp<T, SEL, MUTABLE_PROP> {
    fn item_align(&self) -> OSResult<u32> {
        std::mem::size_of::<T>()
            .try_into()
            .replace_err(OSStatusError::HW_BAD_PROPERTY_SIZE_ERR)
    }
}

impl<T: Copy + 'static, const SEL: u32, const MUTABLE_PROP: bool> RawProperty
    for ArrayProp<T, SEL, MUTABLE_PROP>
{
    fn selector(&self) -> PropertySelector {
        SEL.into()
    }

    fn byte_size(&self) -> OSResult<u32> {
        (std::mem::size_of::<T>() * self.props.len())
            .try_into()
            .replace_err(OSStatusError::HW_BAD_PROPERTY_SIZE_ERR)
    }

    fn is_mut(&self) -> bool {
        MUTABLE_PROP
    }

    fn as_any(&self) -> &dyn Any {
        &self.props
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.props
    }

    unsafe fn set(&mut self, data: *const c_void, data_size: u32) -> OSStatus {
        ret_assert!(data != ptr::null(), OSStatusError::HW_ILLEGAL_OPERATION_ERR);
        ret_assert!(
            data_size == self.item_align()?,
            OSStatusError::HW_BAD_PROPERTY_SIZE_ERR
        );
        ret_assert!(
            data as usize % self.item_align()? as usize == 0,
            OSStatusError::HW_BAD_OBJECT_ERR
        );
        ret_assert!(self.is_mut());
        let r = slice::from_raw_parts(data as *const T, (data_size / self.item_align()?) as usize);
        self.props.clear();
        self.props.extend_from_slice(r);
        Ok(())
    }

    unsafe fn get(
        &self,
        out_alloc_size: u32,
        data_out: *mut c_void,
        data_len_out: *mut u32,
    ) -> OSStatus {
        ret_assert!(
            data_out != ptr::null_mut() && data_len_out != ptr::null_mut(),
            OSStatusError::HW_ILLEGAL_OPERATION_ERR
        );
        ret_assert!(
            out_alloc_size >= self.item_align()?,
            OSStatusError::HW_BAD_PROPERTY_SIZE_ERR
        );
        ret_assert!(
            data_out as usize % self.item_align()? as usize == 0,
            OSStatusError::HW_BAD_OBJECT_ERR
        );

        let s = slice::from_raw_parts_mut(
            data_out as *mut T,
            (out_alloc_size / self.item_align()?) as usize,
        );
        let to_copy = s.len().min(self.props.len());
        s.copy_from_slice(&self.props[0..to_copy]);

        *data_len_out = (to_copy * self.item_align()? as usize)
            .try_into()
            .replace_err(OSStatusError::HW_BAD_PROPERTY_SIZE_ERR)?;
        Ok(())
    }
}
