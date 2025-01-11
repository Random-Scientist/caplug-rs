use core::slice;
use std::{
    any::Any,
    ffi::c_void,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr,
};

use crate::os_err::{OSStatus, OSStatusError, ResultExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PropertySelector(u32);

impl From<u32> for PropertySelector {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<PropertySelector> for u32 {
    fn from(value: PropertySelector) -> Self {
        value.0
    }
}

pub trait RawProperty {
    /// Invariant: this function must always return the correct selector for this property or bad things will happen
    fn selector(&self) -> PropertySelector;
    /// Total size in bytes this type occupies
    fn byte_size(&self) -> u32;
    /// Whether to advertise this property as settable or not
    fn is_mut(&self) -> bool;
    /// Utility function for reading this property's value from Rust
    fn as_any(&self) -> &dyn Any;
    /// Utility function for mutating this property's value from Rust
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// Read a value of the type of the property of this implementation from the `data` pointer and write it to the internal storage
    /// # Safety
    /// data must point to a valid, initialized value or array of values with the same type as this property, with data size being a multiple of the size of that property
    unsafe fn set(&mut self, data: *const c_void, data_size: u32) -> OSStatus;
    /// Write a value stored in this instance to the allocation at `data_out`
    /// # Safety
    /// see discussion under [`RawProperty::set`]
    unsafe fn get(
        &self,
        out_alloc_size: u32,
        data_out: *mut c_void,
        data_len_out: *mut u32,
    ) -> OSStatus;
}

macro_rules! ret_assert {
    ($cond:expr, $err:expr) => {
        if !($cond) {
            ::log::error!(
                "assertion {} == true failed in {}:{}",
                ::std::stringify!($cond),
                ::std::file!(),
                ::std::line!()
            );
            return Err($err);
        }
    };
    ($cond:expr) => {
        if !($cond) {
            ::log::error!(
                "assertion {} == true failed in {}:{}",
                ::std::stringify!($cond),
                ::std::file!(),
                ::std::line!()
            );
            return Err(OSStatusError::HW_UNSPECIFIED_ERR);
        }
    };
}

#[derive(Debug, Clone)]
/// A convenient wrapper for Copy types that implements [RawProperty] for them, given the correct selector and mutability in the const generic parameters
pub struct Prop<T, const SEL: u32, const MUTABLE_PROP: bool>(pub T);

impl<T, const SEL: u32, const MUTABLE_PROP: bool> Prop<T, SEL, MUTABLE_PROP> {
    const SIZE: u32 = const {
        let size = std::mem::size_of::<T>();
        assert!(size <= u32::MAX as usize);
        size as u32
    };
    pub fn new(val: T) -> Self {
        Self(val)
    }
}

//SAFETY: selector invariants and
impl<T: Clone + 'static, const SEL: u32, const MUTABLE_PROP: bool> RawProperty
    for Prop<T, SEL, MUTABLE_PROP>
{
    fn selector(&self) -> PropertySelector {
        SEL.into()
    }

    fn byte_size(&self) -> u32 {
        Self::SIZE
    }
    #[inline]
    fn is_mut(&self) -> bool {
        MUTABLE_PROP
    }

    unsafe fn set(&mut self, data: *const c_void, data_size: u32) -> OSStatus {
        ret_assert!(!data.is_null(), OSStatusError::HW_ILLEGAL_OPERATION_ERR);
        ret_assert!(
            (data as *const T).is_aligned(),
            OSStatusError::HW_BAD_OBJECT_ERR
        );
        ret_assert!(
            data_size == Self::SIZE,
            OSStatusError::HW_BAD_PROPERTY_SIZE_ERR
        );
        ret_assert!(self.is_mut());

        let val = ptr::read(data as *const T);

        self.0 = val;
        Ok(())
    }

    unsafe fn get(
        &self,
        out_alloc_size: u32,
        data_out: *mut c_void,
        data_len_out: *mut u32,
    ) -> OSStatus {
        ret_assert!(
            !data_out.is_null() && !data_len_out.is_null(),
            OSStatusError::HW_ILLEGAL_OPERATION_ERR
        );
        ret_assert!(
            out_alloc_size >= Self::SIZE,
            OSStatusError::HW_BAD_PROPERTY_SIZE_ERR
        );
        let data_out = data_out as *mut T;
        ret_assert!(data_out.is_aligned(), OSStatusError::HW_BAD_OBJECT_ERR);
        ptr::write(data_out, self.0.clone());
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        &self.0
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
/// A convenient wrapper for an array of Copy types as a [RawProperty]
pub struct ArrayProp<T, const SEL: u32, const MUTABLE_PROP: bool> {
    props: Vec<T>,
}
impl<T, const SEL: u32, const MUTABLE_PROP: bool> Deref for ArrayProp<T, SEL, MUTABLE_PROP> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.props
    }
}
impl<T, const SEL: u32, const MUTABLE_PROP: bool> DerefMut for ArrayProp<T, SEL, MUTABLE_PROP> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.props
    }
}
impl<T, const SEL: u32, const MUTABLE_PROP: bool> ArrayProp<T, SEL, MUTABLE_PROP> {
    const ITEM_SIZE: u32 = const {
        let size = std::mem::size_of::<T>();
        assert!(size <= u32::MAX as usize);
        size as u32
    };
    pub fn new_with(props: Vec<T>) -> Self {
        Self { props }
    }
    pub fn new() -> Self {
        Self { props: Vec::new() }
    }
}

impl<T, const SEL: u32, const MUTABLE_PROP: bool> Default for ArrayProp<T, SEL, MUTABLE_PROP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + 'static, const SEL: u32, const MUTABLE_PROP: bool> RawProperty
    for ArrayProp<T, SEL, MUTABLE_PROP>
{
    fn selector(&self) -> PropertySelector {
        SEL.into()
    }
    #[inline]
    fn byte_size(&self) -> u32 {
        Self::ITEM_SIZE * self.len() as u32
    }
    #[inline]
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
        ret_assert!(!data.is_null(), OSStatusError::HW_ILLEGAL_OPERATION_ERR);
        ret_assert!(
            data_size == Self::ITEM_SIZE,
            OSStatusError::HW_BAD_PROPERTY_SIZE_ERR
        );
        let data = data as *const T;
        ret_assert!(data.is_aligned(), OSStatusError::HW_BAD_OBJECT_ERR);
        ret_assert!(self.is_mut());

        let r = slice::from_raw_parts(data, (data_size / Self::ITEM_SIZE) as usize);
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
            !data_out.is_null() && !data_len_out.is_null(),
            OSStatusError::HW_ILLEGAL_OPERATION_ERR
        );
        ret_assert!(
            out_alloc_size >= Self::ITEM_SIZE,
            OSStatusError::HW_BAD_PROPERTY_SIZE_ERR
        );
        let data_out = data_out as *mut T;
        ret_assert!(data_out.is_aligned(), OSStatusError::HW_BAD_OBJECT_ERR);

        let s = slice::from_raw_parts_mut(
            data_out as *mut MaybeUninit<T>,
            (out_alloc_size / Self::ITEM_SIZE) as usize,
        );
        let to_copy = s.len().min(self.props.len());
        let slice = &self.props[0..s.len().min(self.props.len())];
        let slice =
            unsafe { slice::from_raw_parts(slice.as_ptr() as *const MaybeUninit<T>, slice.len()) };

        s.copy_from_slice(slice);

        *data_len_out = (to_copy * Self::ITEM_SIZE as usize)
            .try_into()
            .replace_err(OSStatusError::HW_BAD_PROPERTY_SIZE_ERR)?;
        Ok(())
    }
}
