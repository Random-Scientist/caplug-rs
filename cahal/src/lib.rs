use std::num::NonZeroU32;

use coreaudio_sys::{
    kAudioDevicePermissionsError, kAudioDeviceUnsupportedFormatError, kAudioHardwareBadDeviceError,
    kAudioHardwareBadObjectError, kAudioHardwareBadPropertySizeError, kAudioHardwareBadStreamError,
    kAudioHardwareIllegalOperationError, kAudioHardwareNotReadyError,
    kAudioHardwareNotRunningError, kAudioHardwareUnknownPropertyError,
    kAudioHardwareUnspecifiedError, kAudioHardwareUnsupportedOperationError,
};

pub mod plugin_driver_interface;
pub mod property;

// The layout of this enum is guaranteed to == u32 due the the Option niche optimization guaranteed by the Reference
#[derive(Debug)]
pub enum OSResult<T> {
    Ok(T),
    Err(NonZeroU32),
}
#[derive(Debug, Clone, Copy)]
pub struct OSStatusError(NonZeroU32);
impl OSStatusError {
    const HW_NOT_RUNNING_ERR: NonZeroU32 = kAudioHardwareNotRunningError.try_into().unwrap();
    const HW_UNSPECIFIED_ERR: NonZeroU32 = kAudioHardwareUnspecifiedError.try_into().unwrap();
    const HW_UNKNOWN_PROP_ERR: NonZeroU32 = kAudioHardwareUnknownPropertyError.try_into().unwrap();
    const HW_BAD_PROPERTY_SIZE_ERR: NonZeroU32 =
        kAudioHardwareBadPropertySizeError.try_into().unwrap();
    const HW_ILLEGAL_OPERATION_RR: NonZeroU32 =
        kAudioHardwareIllegalOperationError.try_into().unwrap();
    const HW_BAD_OBJECT_ERR: NonZeroU32 = kAudioHardwareBadObjectError.try_into().unwrap();
    const HW_BAD_DEVICE_ERR: NonZeroU32 = kAudioHardwareBadDeviceError.try_into().unwrap();
    const HW_BAD_STREAM_ERR: NonZeroU32 = kAudioHardwareBadStreamError.try_into().unwrap();
    const HW_UNSUPPORTED_OP: NonZeroU32 =
        kAudioHardwareUnsupportedOperationError.try_into().unwrap();
    const HW_NOT_READ_ERR: NonZeroU32 = kAudioHardwareNotReadyError.try_into().unwrap();
    const DEV_UNSUPPORTED_FMT_ERR: NonZeroU32 =
        kAudioDeviceUnsupportedFormatError.try_into().unwrap();
    const DEV_PERMISSIONS_ERR: NonZeroU32 = kAudioDevicePermissionsError.try_into().unwrap();
}
impl From<OSStatusError> for OSResult<()> {
    fn from(value: OSStatusError) -> Self {
        OSResult::Err(value.0)
    }
}
pub type OSStatus = OSResult<()>;
impl OSStatus {
    fn from_raw(val: u32) -> Self {
        unsafe { std::mem::transmute(val) }
    }
    fn as_raw(&self) -> u32 {
        unsafe { std::mem::transmute(*self) }
    }
}
