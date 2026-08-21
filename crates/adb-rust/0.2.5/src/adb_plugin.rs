use std::boxed::Box;
use std::collections::HashMap;
use std::process::{Child, Command, Output};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};

pub trait ADBPlugin {
    fn new(cmd: String) -> Self;
    // 使方法借用而不是拥有 `self`，允许修改实例
    fn set_plugin(self, status: String, fnc: Box<dyn FnMut(String)>);

    // 返回一个 `Box<dyn Fn()>`，因为 `dyn Fn` 的大小在编译时是不确定的
    fn use_plugin(self, status: String) -> Box<dyn FnMut(String)>;

    // 方法借用 `self`
    async fn use_cmd(self, args: Vec<&str>);
}

#[derive(Debug )]
pub struct ADBPluginImpl {
    plugins: HashMap<String, Arc<Mutex<Box<dyn FnMut(String)>>>>,
    cmd: Command,
}


impl ADBPlugin for ADBPluginImpl {
    fn new(cmd: String) -> Self {
        ADBPluginImpl {
            plugins: HashMap::new(),
            cmd: Command::new(cmd),
        }
    }
    fn set_plugin(mut self, status: String, fnc: Box<dyn FnMut(String)>) {
        // 使用 Arc 和 Mutex 来共享和修改闭包
        self.plugins.insert(status, Arc::new(Mutex::new(fnc)));
    }

    fn use_plugin(self, status: String) -> Box<dyn FnMut(String)> {
        if let Some(fnc) = self.plugins.get(&status) {
            // 使用 clone 来获取 Arc 的一个新引用
            let fnc_clone = fnc.clone();
            // 返回一个可以调用存储闭包的新闭包
            Box::new(move |_| {
                // 锁定并调用闭包
                let mut fnc_lock = fnc_clone.lock().unwrap();
                fnc_lock("/* String */".to_string())
            })
        } else {
            // 如果没有找到闭包，返回一个空操作的闭包
            Box::new(|_| {
                "".to_string();
            })
        }
    }

    async fn use_cmd(mut self, args: Vec<&str>) {
        let mut s = self.use_plugin("status".to_string());

        if let Some(stdout) = self
            .cmd
            .args(args)
            .stdout(std::process::Stdio::piped()) // 将标准输出重定向到管道
            .spawn() // 启动子进程
            .expect("failed to execute command")
            .stdout
            .take()
        {}
    }
}
