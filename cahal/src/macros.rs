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
