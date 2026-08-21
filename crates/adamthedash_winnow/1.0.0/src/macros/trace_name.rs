/// Given string with module path prepended
/// `trace_name!("hello") -> "module::path::hello"`
#[macro_export]
macro_rules! trace_name {
    ($name:literal) => {
        concat!(module_path!(), "::", $name)
    };
}
