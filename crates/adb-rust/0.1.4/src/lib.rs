pub mod cmd;

#[cfg(test)]
mod tests {
    use crate::cmd::{ADBCmd, ADBCmdTrait};

    use super::*;
    use tokio;

    #[tokio::test]

    async fn test_adb_cmd() {
        let res = cmd::ADBCmd::get_file_path("./resources/file.xml").unwrap();
        let res = res.replace("\\\\?\\", "");
        let args = vec!["push".to_string(), res, "/data/local/tmp/".to_string()];
        let binding = ADBCmd::new("adb".to_string(), false);
        let child = binding.run(args);
        match child.await {
            Ok(stdout) => {
                println!("{}", stdout)
            }
            Err(stderr) => {
                println!("{}", stderr)
            }
        }
    }
}
