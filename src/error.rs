pub type Result<T, E = String> = std::result::Result<T, E>;

#[macro_export]
macro_rules! anyhow {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        format!($fmt $(, $arg)*)
    };
    ($err:expr $(,)?) => {
        ::std::string::ToString::to_string(&$err)
    };
}

#[macro_export]
macro_rules! bail {
    ($($arg:tt)*) => {
        return Err($crate::anyhow!($($arg)*))
    };
}
