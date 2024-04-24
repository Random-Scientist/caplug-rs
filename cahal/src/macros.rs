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
