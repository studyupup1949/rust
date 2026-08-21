pub mod cmd;
use cmd::ADBCmd;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adb_cmd() {
        let adb = ADBCmd::new();
        let args = vec!["devices".to_string()];
        let _ = adb.run_async(args, |line| {
            println!("line{}", line);
            line
        });
    }
}
