### A Simple ADB Client Written in Rust
===========================

ADBCmd can execute commands synchronously or asynchronously using callback function closures.

Currently, this library is primarily used for invoking "cmd" commands in Rust, providing a variety of practical APIs.
- **new**: Creates an instance of ADBCmd, returning an ADBCmd instance.
- **create_adb_cmd**: Creates an asynchronous `Command` instance based on `tokio::process::Command`, returning the instance of `Command`.
- **run_async**: Executes an asynchronous command by passing callback parameters to get the `Command` output in real-time.
- **run**: Executes a synchronous command (based on `std::process::Command`) to obtain the output of the current synchronous command, returning a `Result<String, String>`.
- **get_file_path**: Gets the file path of the given path; if it does not exist, it returns an error message, returning a `Result<String, String>`.

### Usage
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
adb-rust="0.2.1"
```
- Invocation
```rust
use adb_rust::cmd::{ADBCmd, ADBCmdResult};

pub fn push_xml_file() {
    let file: &str = "./resources/file.xml";
    let paths: String = ADBCmd::get_file_path(file).unwrap();
    let args: Vec<String> = vec!["push".to_string(), paths, "/sdcard/".to_string()];
    let res: Result<String, String> = ADBCmd::new("adb", true).run(&args);
    ...
    ...        
}
```

### API

```rust
pub mod tests {
    use crate::cmd::{ADBCmd, ADBCmdTrait};
    use super::*;
    use tokio;

    #[test]
    fn test_adb_cmd() {
        let path = ADBCmd::get_file_path("./resources/file.xml").unwrap();
        let cleaned_path = path.replace("\\\\?\\", "");
        let args = vec!["push".to_string(), cleaned_path, "/data/local/tmp/".to_string()];
        let binding = ADBCmd::new("adb", false);
        let result = binding.run(&args);
        match result {
            Ok(stdout) => println!("{}", stdout),
            Err(stderr) => println!("{}", stderr),
        }
    }

    #[tokio::test]
    async fn test_run_async() {
        let adb_cmd = ADBCmd::new("adb", false);
        let args = vec!["devices".to_string()];
        adb_cmd.run_async(args, |line| {
            println!("{}", line);
            line
        }).await;
    }
}
```