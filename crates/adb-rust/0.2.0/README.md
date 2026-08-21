### A simple adb client written in rust.
===========================

ADBCmd can be executed using synchronous execution commands, and can also be executed using asynchronous, callback function closure methods.

目前，这个库主要用于在Rust中调用 “cmd” 的命令，包含一些实用的API。
- **new()**: 创建一个ADBCmd实例,返回一个ADBCmd实例。
- **create_adb_cmd()**: 创建一个异步的（基于tokio::process:Command）的Command实例，返回Command的实例，
- **run_async()**: 运行一个异步的命令，通过传入回调参数的形式，实时获取Command的输出
- **run()**:运行一个同步的命令（基于std::process::Command），获取当前同步命令的输出，返回一个Result<String, String>
- **get_file_path()**: 获取传入路径的文件路径，如果不存在则返回错误信息，返回一个Result<String, String>


### 使用方法
```toml
///cargo.toml
    [dependencies]
    tokio = { version = "1.0", features = ["full"] }
    adb-rust="0.1.6"
```
- 调用
```rust
use adb_rust::cmd::{ADBCmd, ADBCmdResult}
pub fn push_xml_file(){
    let file:&str= "./resources/file.xml"
    let paths:String = ADBCmd::get_file_path(file).unwrap();
    let args:Vec<&String> = vec!["push".to_string(),&paths,"/sdcard/".to_string()];
    let res:Result<String, String> = ADBCmd::new("adb",true).run(args);
    ...
    ...        
}

```
    

### APi

```rust
pub mod tests {
    use crate::cmd::{ADBCmd, ADBCmdTrait};

    use super::*;
    use tokio;

    #[test]

    fn test_adb_cmd() {
        let res = cmd::ADBCmd::get_file_path("./resources/file.xml").unwrap();
        let res = res.replace("\\\\?\\", "");
        let args = vec!["push".to_string(), res, "/data/local/tmp/".to_string()];
        let binding = ADBCmd::new("adb".to_string(), false);
        let child = binding.run(args);
        match child {
            Ok(stdout) => {
                println!("{}", stdout)
            }
            Err(stderr) => {
                println!("{}", stderr)
            }
        }
    }

    #[tokio::test]
    async fn test_run_async() {
        let adb_cmd = ADBCmd::new("adb".to_string(), false);
        let args = ["devices".to_string()];
        adb_cmd
            .run_async(args.to_vec(), |line| {
                println!("{}", line);
                line
            })
            .await;
    }
}


```