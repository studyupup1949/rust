use std::future::Future;
use std::path::Path;
use std::process::Child;
use std::process::Output;
use std::process::Stdio;
use std::os::windows::process::CommandExt;
use tokio::io::{AsyncBufReadExt, BufReader}; // 确保导入所需的trait
use tokio::process::Command;
pub trait ADBCmdTrait {
    fn new(cmd: String, is_shell: bool) -> Self;
    fn create_cmd(self) -> Command;
    async fn run_async<F>(&self, args: Vec<String>, fnc: F)
    where
        F: FnMut(String) -> String + 'static;
    fn run(&self, args: Vec<String>) -> Result<String, String>;
    fn get_var_arg(self, args: Vec<String>) -> impl Future<Output = bool>;
    fn get_file_path(path: &str) -> Result<String, String>;
    fn exec(&self, args: Vec<String>) -> Result<Output, String>;
}

const CREATE_NO_WINDOW: u32 = 0x08000000;
/// ADBCmd allows you to execute adb commands asynchronously.
#[derive(Debug, Clone)]
pub struct ADBCmd {
    cmd: String,
    is_shell: bool,
}
impl ADBCmdTrait for ADBCmd {
    fn new(cmd: String, is_shell: bool) -> Self {
        ADBCmd { cmd, is_shell }
    }
    fn create_cmd(self) -> Command {
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
    async fn run_async<F>(&self, args: Vec<String>, mut fnc: F)
    where
        F: FnMut(String) -> String + 'static,
    {
        let mut cmd = <ADBCmd as Clone>::clone(&self)
            .create_cmd()
            .creation_flags(CREATE_NO_WINDOW)
            .arg("/c")
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
    /// run runs the given adb command synchronously.
    /// ```no_run
    /// use std::process::Command;
    ///
    /// use ADB::cmd::ADBCmd;
    ///
    /// let adb_cmd = ADBCmd::new("cmd".to_string(),false);
    ///
    /// let result = adb_cmd.run(vec!["devices".to_string()]);
    /// ```
    fn run(&self, args: Vec<String>) -> Result<String, String> {
        let mut output = std::process::Command::new("cmd");
        output.arg("/C");
        output.arg(&self.cmd);
        if self.is_shell {
            output.arg("shell".to_string());
        }
        output.args(args);
        let child = output
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|err| format!("Failed command : {}", err))?;
        let stdout: String = String::from_utf8_lossy(&child.stdout).to_string();
        let stderr: String = String::from_utf8_lossy(&child.stderr).to_string();

        if !stderr.is_empty() {
            Err(stderr)
        } else {
            Ok(stdout)
        }
    }
    fn exec(&self, args: Vec<String>) -> Result<Output, String> {
        let mut output = std::process::Command::new("cmd");
        output.arg("/C");
        output.arg(&self.cmd);
        if self.is_shell {
            output.arg("shell".to_string());
        }
        output.args(args);
        let child = output
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|err| format!("Failed command : {}", err));
        child
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
    async fn get_var_arg(self, args: Vec<String>) -> bool {
        let res = self.run(args);
        match res {
            Ok(_) => true,
            Err(_) => false,
        }
    }
    /// 获取文件路径
    /// ```no_run
    ///  let res= ADBCmd::new("adb",false).get_file_path("./resources/file.xml");
    /// ```
    fn get_file_path(path: &str) -> Result<String, String> {
        let mut _custom_path = Path::new(path).canonicalize();
        match _custom_path {
            Ok(p) => Ok(p.as_os_str().to_string_lossy().to_string()),
            Err(_err) => Err(format!("The file path does not exist or is incorrect")),
        }
    }
}
