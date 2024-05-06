#[macro_export]
macro_rules! ret_assert {
    ($cond:expr, $err:expr) => {
        if !($cond) {
            return Err($err);
        }
    };
    ($cond:expr) => {
        if !($cond) {
            return Err(OSStatusError::HW_UNSPECIFIED_ERR);
        }
    };
}
#[macro_export]
macro_rules! const_nonzero_u32 {
    ($e:expr) => {{
        const _: () = assert!($e != 0, "Tried to initialize const NonZeroU32 with 0");
        unsafe { ::core::num::NonZeroU32::new_unchecked($e) }
    }};
}
/// Creates the necessary CFPlugin entry point function (named "__audio_driver_entry")
#[macro_export]
macro_rules! entry_point {
    ($t:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn __create_driver(alloc: ::cahal::base::CFAllocatorRef, requested_uuid: ::cahal::base::CFUUIDRef) -> *mut ::std::ffi::c_void {
            <$t as ::cahal::raw_plugin_driver_interface::RawAudioServerPlugInDriverInterface>::create(alloc, requested_uuid)
        }
    };
}
