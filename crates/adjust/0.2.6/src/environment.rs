#[macro_export]
/// Reads an environment variable, and goes into panic if it is not found.
/// ```rs
/// fn main() {
///   println!("MAX_DUCK_SIZE is `{}`", read_env!(MAX_DUCK_SIZE));
/// }
/// ```
macro_rules! read_env {
  ($var:ident) => {
    std::env::var(&stringify!($var)).expect(&format!("required environment variable `{}` not found", stringify!($var)));
  };
}

#[macro_export]
/// Does the same thing as `read_env`, but creates `static` variable for the environment var.
/// ```rs
/// static_env!(MAX_DUCK_SIZE);
/// // Which will be expanded into
/// // static MAX_DUCK_SIZE: LazyLock<String> = ...;
///
/// fn main() {
///   println!("MAX_DUCK_SIZE is {}", *MAX_DUCK_SIZE);
/// }
/// ```
macro_rules! static_env {
  ($(#[$meta:meta])? $vis:vis $var:ident) => {
    $vis static $var: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| read_env!($var));
  };
}