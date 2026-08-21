pub mod cmd;

#[cfg(test)]
mod tests {
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
