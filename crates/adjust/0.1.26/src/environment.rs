#[macro_export]
/// Loads enviroment variable, and makes it
macro_rules! load_env {
  ($(#[$meta:meta])? $vis:vis $var:ident) => {
    $vis static $var: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
      let var = std::env::var(stringify!($var));

      if var.is_err() {
        log::error!("Environment variable ${} not found!", stringify!($var));
      }

      var.unwrap()
    });
  };
}