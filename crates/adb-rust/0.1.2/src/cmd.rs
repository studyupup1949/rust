use tokio::io::{AsyncBufReadExt, BufReader}; // 确保导入所需的trait
use tokio::process::Command;

/// ADBCmd allows you to execute adb commands asynchronously.
pub struct ADBCmd {}
impl ADBCmd {
    pub fn new() -> ADBCmd {
        ADBCmd {}
    }
    // run_async runs the given adb command asynchronously.

    /// ```no_run
    /// use std::process::Command;
    ///
    /// use ADB::cmd::ADBCmd;
    ///
    /// let adb_cmd = ADBCmd::new();
    ///
    /// db_cmd.run_async(vec!["devices".to_string()], |line| {
    ///     println!("{}", line);
    ///     line
    /// }).await;
    /// ```
    pub async fn run_async<F>(&self, args: Vec<String>, mut fnc: F)
    where
        F: FnMut(String) -> String + 'static,
    {
        let mut cmd = Command::new("adb")
            .args(args)
            .stdout(std::process::Stdio::piped()) // 将标准输出重定向到管道
            .spawn() // 启动子进程
            .expect("failed to execute command");

        if let Some(stdout) = cmd.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let _result = fnc(line);
                // 这里你可以直接使用 `result`，或者根据需要做进一步的处理
            }
        }
    }
    // run runs the given adb command synchronously.

    /// ```no_run
    /// use std::process::Command;
    ///
    /// use ADB::cmd::ADBCmd;
    ///
    /// let adb_cmd = ADBCmd::new();
    ///
    /// let result = adb_cmd.run(vec!["devices".to_string()]);
    /// ```
    pub fn run(&self, args: Vec<String>) -> Result<String, String> {
        let output = std::process::Command::new("adb").args(args).output();

        match output {
            Ok(child) => {
                if child.status.success() {
                    let stdout = String::from_utf8_lossy(&child.stdout).to_string();
                    Ok(stdout)
                } else {
                    let stderr = String::from_utf8_lossy(&child.stderr).to_string();
                    Err(stderr)
                }
            }
            Err(err) => {
                let err_str = format!("Failed to execute Adb command： {}", err) as String;
                Err(err_str)
            }
        }
    }
}
