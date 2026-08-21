use anyhow::{anyhow, Result as AnyResult};
use std::process::Command;

#[cfg(feature = "dev")]
/// vid:vendor-id,pid:product-id
pub fn LinuxFindUsbDevice(vid: &str, pid: &str) -> AnyResult<bool> {
    let dev_id = format!("ID {}:{}", vid, pid);
    match Command::new("/bin/lsusb").output() {
        Ok(output) => {
            let mut sOutput = String::from_utf8(output.stdout)?;
            tracing::error!("LSUSB-OUTPUT:\n{}",sOutput);
            if sOutput.len() == 0 {
                let sErr = String::from_utf8(output.stderr)?;
                return Err(anyhow!("check-usb-device-error {}", sErr));
            }
            let usblist = sOutput.split("\n");
            for dev in usblist {
                if dev.contains(&dev_id) {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        Err(e) => Err(anyhow!("check-usb-device-error {}", e)),
    }
}

#[cfg(test)]
#[cfg(feature = "dev")]
mod tests {
    use super::*;
    use anyhow::{anyhow, Result as AnyResult};

    #[test]
    fn test_find_usb_device() {
        println!("{:?}", LinuxFindUsbDevice("1d6b", "0003"));
    }
}
