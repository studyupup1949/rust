use std::path::Path;

use tokio::io::{AsyncBufReadExt, BufReader}; // 确保导入所需的trait
use tokio::process::Command;

/// ADBCmd allows you to execute adb commands asynchronously.
#[derive(Debug, Clone)]
pub struct ADBCmd {
    cmd: String,
    is_shell: bool,
}
impl ADBCmd {
    pub fn new(cmd: String, is_shell: bool) -> Self {
        ADBCmd { cmd, is_shell }
    }
    pub fn create_cmd(self) -> Command {
        let mut command = Command::new(self.cmd);
        if self.is_shell {
            command.arg("shell");
        }
        command
    }
    // run_async runs the given adb command asynchronously.
    /// ```no_run
    ///
    /// use std::process::Command;
    ///
    /// use ADB::cmd::ADBCmd;
    ///
    /// let adb_cmd = ADBCmd::new();
    /// #[tokio::main]
    /// adb_cmd.run_async(vec!["devices".to_string()], |line| {
    ///     println!("{}", line);
    ///     line
    /// }).await;
    /// ```
    pub async fn run_async<F>(&self, args: Vec<String>, mut fnc: F)
    where
        F: FnMut(String) -> String + 'static,
    {
        let mut cmd = <ADBCmd as Clone>::clone(&self)
            .create_cmd()
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
    pub async fn run(&self, args: Vec<String>) -> Result<String, String> {
        // let output = std::process::Command::new(self.cmd).args(args).output();
        let output = <ADBCmd as Clone>::clone(&self)
            .create_cmd()
            .args(args)
            .output();
        match output.await {
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

    /// Get variable on command

    /// ```no_run
    ///  let mut arg=String::from("mmi");
    ///  let exec_args=vec![];
    ///  let args= vec![
    ///     "ls".to_string(),
    ///     "/system/bin/sxrmmi".to_string()
    ///     ]
    ///  let _prefix= ADBCmd::new("adb",false).get_var_arg(args);
    ///  if _prefix.is_ok(){
    ///      arg = format!("{}{}","sxr".to_string(),arg);
    ///  }else{
    ///      arg = format!("{}{}","ssnwt".to_string(),arg);
    ///  }
    ///  exec_args.push(arg);
    ///  exec_args.push("stop".to_string());
    ///  let res= ADBCmd::new("adb",false).run(exec_args);
    ///  
    /// ```
    pub async fn get_var_arg(self, args: Vec<String>) -> bool {
        let res = self.run(args).await;
        match res {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    /// 获取文件路径
    /// ```no_run
    ///  let res= ADBCmd::new("adb",false).get_file_path("./resources/file.xml");
    /// ```
    pub fn get_file_path(path: &str) -> Result<String, String> {
        let mut _custom_path = Path::new(path).canonicalize();
        match _custom_path {
            Ok(p) => Ok(p.as_os_str().to_string_lossy().to_string()),
            Err(_err) => Err(format!("The file path does not exist or is incorrect")),
        }
    }
}
