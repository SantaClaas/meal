#[macro_export]
macro_rules! redirect_to {
    ($($arg:tt)*) => {
        Redirect::to(&format!($($arg)*))
    };
}
