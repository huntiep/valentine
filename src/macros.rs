#[macro_export]
macro_rules! try_opt {
    ($expr:expr) => (match $expr {
        ::std::option::Option::Some(val) => val,
        ::std::option::Option::None => return ::std::option::Option::None,
    })
}
