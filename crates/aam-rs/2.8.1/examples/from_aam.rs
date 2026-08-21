use aam_rs::FromAam;
use aam_rs::from_aam::FromAam as _;

#[derive(Debug, Clone, FromAam, PartialEq)]
pub struct Listener {
    pub timeout: u32,
    pub on_timeout: Option<String>,
    pub on_resume: Option<String>,
    pub max_cpu_usage: f32,
    pub max_gpu_usage: f32,
    pub min_ram_mb: u32,
    pub min_vram_mb: u32,
    pub keep_alive_processes: Vec<String>,
}

#[derive(Debug, Clone, FromAam, PartialEq)]
pub struct GeneralConfig {
    pub lock_cmd: Option<String>,
    pub unlock_cmd: Option<String>,
    pub before_sleep_cmd: Option<String>,
    pub after_sleep_cmd: Option<String>,
    pub ignore_dbus_inhibit: bool,
}

#[derive(Debug, Clone, FromAam, PartialEq)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub listeners: Vec<Listener>,
}

fn main() {
    // Full AAM format: key = value lines
    let config = AppConfig::from_aam_str(
        "general = { lock_cmd = swaylock, ignore_dbus_inhibit = true }\n\
         listeners = [\
           { timeout = 300, max_cpu_usage = 90.0, max_gpu_usage = 80.0, min_ram_mb = 512, min_vram_mb = 256, keep_alive_processes = [firefox, code] },\
           { timeout = 600, on_timeout = suspend, max_cpu_usage = 50.0, max_gpu_usage = 40.0, min_ram_mb = 1024, min_vram_mb = 512, keep_alive_processes = [steam] }\
         ]\n"
    ).expect("failed to parse config");

    println!("Parsed config:\n{config:#?}");
    assert_eq!(config.general.lock_cmd, Some("swaylock".into()));
    assert!(config.general.ignore_dbus_inhibit);
    assert_eq!(config.listeners.len(), 2);
    assert_eq!(config.listeners[0].timeout, 300);
    assert_eq!(config.listeners[1].on_timeout, Some("suspend".into()));
    println!("\nAll assertions passed!");
}
