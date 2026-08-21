use std::process::{Command, Output};

pub struct Adb {
    adb_path: String,
}

impl Adb {
    pub fn new(adb_path: &str) -> Self {
        Adb {
            adb_path: adb_path.to_string(),
        }
    }

    pub fn execute(&self, args: &[&str]) -> Result<Output, std::io::Error> {
        Command::new(&self.adb_path)
            .args(args)
            .output()
    }

    pub fn devices(&self) -> Result<String, std::io::Error> {
        let output = self.execute(&["devices"])?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn install(&self, apk_path: &str) -> Result<String, std::io::Error> {
        let output = self.execute(&["install", apk_path])?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn uninstall(&self, package_name: &str) -> Result<String, std::io::Error> {
        let output = self.execute(&["uninstall", package_name])?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
