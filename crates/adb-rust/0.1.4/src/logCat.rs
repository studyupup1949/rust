use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

pub struct LogCat {
    log_child: Child,
}
impl LogCat {
    pub fn new() -> Self {
        let log_adb = Command::new("adb")
            .arg("logcat")
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start adb logcat");
        LogCat { log_child: log_adb }
    }
    pub fn read_child<F>(self, mut fnc: F)
    where
        F: FnMut(String) -> String + 'static,
    {
        let stdout = self.log_child.stdout.expect("failed to capture stdout");

        // 使用 BufReader 逐行读取输出
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // 这里处理每一行的输出
                    let res = format!("ok:{}", line);
                    fnc(res);
                }
                Err(e) => {
                    eprintln!("Error reading line: {}", e);
                }
            }
        }
    }
    pub fn kill_child(mut self) {
        let _ = self.log_child.kill();
    }
}
