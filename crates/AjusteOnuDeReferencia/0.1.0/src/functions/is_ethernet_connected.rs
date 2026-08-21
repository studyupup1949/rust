use std::process::Command;

pub fn is_ethernet_connected() -> Result<bool, String> {
    let result = if cfg!(target_os = "linux") {
        let output = Command::new("ip")
            .args(&["link", "show", "eth0"])
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains("UP"))
    } else if cfg!(target_os = "windows") {
        let output = Command::new("powershell")
            .args(&["Get-NetAdapter | Where-Object { $_.Name -eq 'Ethernet' -and $_.Status -eq 'Up' } | Measure-Object | ForEach-Object { $_.Count }"])
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;
        
        let count = String::from_utf8_lossy(&output.stdout).trim().parse::<usize>().unwrap_or(0);
        Ok(count > 0)
    } else {
        Ok(false)
    };
    
    result
}