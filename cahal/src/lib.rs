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

type OSResult<T> = Result<T, OSStatusError>;
#[derive(Debug, Clone, Copy)]
pub struct OSStatusError(NonZeroU32);
macro_rules! const_nonzero_u32 {
    ($e:expr) => {{
        const _: () = assert!($e != 0, "Tried to initialize const NonZeroU32 with 0");
        unsafe { ::core::num::NonZeroU32::new_unchecked($e) }
    }};
}
impl OSStatusError {
    //TODO: Future me. Make a macro for this to make it less painful
    const HW_NOT_RUNNING_ERR: Self = Self(const_nonzero_u32!(kAudioHardwareNotRunningError));
    const HW_UNSPECIFIED_ERR: Self = Self(const_nonzero_u32!(kAudioHardwareUnspecifiedError));
    const HW_UNKNOWN_PROP_ERR: Self = Self(const_nonzero_u32!(kAudioHardwareUnknownPropertyError));
    const HW_BAD_PROPERTY_SIZE_ERR: Self =
        Self(const_nonzero_u32!(kAudioHardwareBadPropertySizeError));
    const HW_ILLEGAL_OPERATION_ERR: Self =
        Self(const_nonzero_u32!(kAudioHardwareIllegalOperationError));
    const HW_BAD_OBJECT_ERR: Self = Self(const_nonzero_u32!(kAudioHardwareBadObjectError));
    const HW_BAD_DEVICE_ERR: Self = Self(const_nonzero_u32!(kAudioHardwareBadDeviceError));
    const HW_BAD_STREAM_ERR: Self = Self(const_nonzero_u32!(kAudioHardwareBadStreamError));
    const HW_UNSUPPORTED_OP: Self =
        Self(const_nonzero_u32!(kAudioHardwareUnsupportedOperationError));
    const HW_NOT_READ_ERR: Self = Self(const_nonzero_u32!(kAudioHardwareNotReadyError));
    const DEV_UNSUPPORTED_FMT_ERR: Self =
        Self(const_nonzero_u32!(kAudioDeviceUnsupportedFormatError));
    const DEV_PERMISSIONS_ERR: Self = Self(const_nonzero_u32!(kAudioDevicePermissionsError));
}
impl From<OSStatusError> for OSResult<()> {
    fn from(value: OSStatusError) -> Self {
        OSResult::Err(value)
    }
}
pub type OSStatus = OSResult<()>;

pub trait ResultExt<T> {
    fn replace_err<U>(self, err: U) -> Result<T, U>;
}
impl<T, E> ResultExt<T> for Result<T, E> {
    fn replace_err<U>(self, err: U) -> Result<T, U> {
        match self {
            Ok(v) => Ok(v),
            Err(_) => Err(err),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::property::{RawProperty, SingularTypedProperty};

    #[test]
    fn test_property() {
        const SEL1: u32 = 10;
        const SEL2: u32 = 12;
        let i = unsafe { SingularTypedProperty::<_, SEL1, true>::new(123) };
        let i2 = unsafe { SingularTypedProperty::<_, SEL2, true>::new(123) };
        let mut v = Vec::<Box<dyn RawProperty>>::new();
        v.push(Box::new(i));
        v.push(Box::new(i2));
        assert_eq!(v.pop().unwrap().selector(), SEL2);
        assert_eq!(v.pop().unwrap().selector(), SEL1);
    }
}
