#[cfg(feature = "fs")]
use std::io;

#[cfg(feature = "fs")]
pub fn get_exe_dir()->String {
    std::env::current_exe().unwrap()
        .parent().unwrap().to_path_buf().into_os_string().into_string().unwrap()
}

#[cfg(feature = "fs")]
pub fn mkdir(path:&str)->io::Result<()> {
    std::fs::create_dir_all(path)
}

#[cfg(test)]
#[cfg(feature = "fs")]
mod tests {
    use super::*;

    #[test]
    fn test_get_exe_dir() {
        let result = get_exe_dir();
        println!("got_exe_dir: {}",result);
        assert_eq!(result.len()>0, true);
    }

    #[test]
    fn test_mkdir() {
        let result = mkdir("/tmp/123456");
        println!("test_mkdir: {:?}",result);
        assert_eq!(result.is_ok(),true);
    }
}