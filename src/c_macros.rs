#[macro_export]
macro_rules! c_str_ptr {
    ($lit:expr) => {
        // Currently, there is no working way to concatenate a byte string
        // literal out of bytestring or string literals. Otherwise, we could
        // use from_static_bytes and accept byte strings as well.
        // See https://github.com/rust-lang/rfcs/pull/566
        std::ffi::CStr::from_ptr(concat!($lit, "\0").as_ptr() as *const std::os::raw::c_char).as_ptr() as *const u8
    };
}
