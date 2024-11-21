#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        panic!($($arg)*);
    };
}