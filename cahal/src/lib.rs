pub mod audio_object;
pub mod macros;
pub mod plugin_driver_interface;
pub mod property;
pub mod raw_plugin_driver_interface;
pub use core_foundation;
pub use coreaudio_sys as base;

pub mod os_err {
    use std::num::NonZeroU32;

    use coreaudio_sys::{
        kAudioDevicePermissionsError, kAudioDeviceUnsupportedFormatError,
        kAudioHardwareBadDeviceError, kAudioHardwareBadObjectError,
        kAudioHardwareBadPropertySizeError, kAudioHardwareBadStreamError,
        kAudioHardwareIllegalOperationError, kAudioHardwareNotReadyError,
        kAudioHardwareNotRunningError, kAudioHardwareUnknownPropertyError,
        kAudioHardwareUnspecifiedError, kAudioHardwareUnsupportedOperationError,
    };

    use crate::const_nonzero_u32;

    pub type OSResult<T> = Result<T, OSStatusError>;
    pub fn result_from_raw(value: i32) -> OSStatus {
        if let Some(val) = NonZeroU32::new(value as u32) {
            Err(OSStatusError(val))
        } else {
            Ok(())
        }
    }
    pub fn result_to_raw(value: OSStatus) -> i32 {
        match value {
            Ok(()) => 0,
            Err(n) => n.0.get() as i32,
        }
    }
    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct OSStatusError(NonZeroU32);
    impl OSStatusError {
        //TODO: Future me. Make a macro for this to make it less painful
        pub const HW_NOT_RUNNING_ERR: Self =
            Self(const_nonzero_u32!(kAudioHardwareNotRunningError));
        pub const HW_UNSPECIFIED_ERR: Self =
            Self(const_nonzero_u32!(kAudioHardwareUnspecifiedError));
        pub const HW_UNKNOWN_PROP_ERR: Self =
            Self(const_nonzero_u32!(kAudioHardwareUnknownPropertyError));
        pub const HW_BAD_PROPERTY_SIZE_ERR: Self =
            Self(const_nonzero_u32!(kAudioHardwareBadPropertySizeError));
        pub const HW_ILLEGAL_OPERATION_ERR: Self =
            Self(const_nonzero_u32!(kAudioHardwareIllegalOperationError));
        pub const HW_BAD_OBJECT_ERR: Self = Self(const_nonzero_u32!(kAudioHardwareBadObjectError));
        pub const HW_BAD_DEVICE_ERR: Self = Self(const_nonzero_u32!(kAudioHardwareBadDeviceError));
        pub const HW_BAD_STREAM_ERR: Self = Self(const_nonzero_u32!(kAudioHardwareBadStreamError));
        pub const HW_UNSUPPORTED_OP: Self =
            Self(const_nonzero_u32!(kAudioHardwareUnsupportedOperationError));
        pub const HW_NOT_READ_ERR: Self = Self(const_nonzero_u32!(kAudioHardwareNotReadyError));
        pub const DEV_UNSUPPORTED_FMT_ERR: Self =
            Self(const_nonzero_u32!(kAudioDeviceUnsupportedFormatError));
        pub const DEV_PERMISSIONS_ERR: Self =
            Self(const_nonzero_u32!(kAudioDevicePermissionsError));
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
}

#[cfg(test)]
mod tests {
    use crate::property::{Prop, RawProperty};

    #[test]
    fn test_property() {
        const SEL1: u32 = 10;
        const SEL2: u32 = 12;
        let i = Prop::<_, SEL1, true>::new(123);
        let i2 = Prop::<_, SEL2, true>::new(123);
        let mut v = Vec::<Box<dyn RawProperty>>::new();
        v.push(Box::new(i));
        v.push(Box::new(i2));
        assert_eq!(v.pop().unwrap().selector(), SEL2.into());
        assert_eq!(v.pop().unwrap().selector(), SEL1.into());
    }
}
