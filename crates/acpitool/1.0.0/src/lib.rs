use std::error::Error;
use std::path;

pub struct Config {
    pub acpi_path: path::PathBuf,
    pub show_battery: bool,
    pub show_ac_adapter: bool,
    pub show_thermal_sensors: bool,
    pub show_cooling_devices: bool,
    pub detailed: bool,
    pub units: acpi_client::Units,
}

pub fn run(cfg: Config) -> Result<(), Box<dyn Error>> {
    if cfg.show_battery {
        let batteries: Vec<acpi_client::BatteryInfo> =
            match acpi_client::get_battery_info(&cfg.acpi_path.join("power_supply")) {
                Ok(bat) => bat,
                Err(e) => {
                    eprintln!("Application error: {}", e);
                    std::process::exit(1);
                }
            };
        for bat in batteries {
            display_battery_info(&bat, cfg.detailed);
        }
    }
    if cfg.show_ac_adapter {
        let adapters: Vec<acpi_client::ACAdapterInfo> =
            match acpi_client::get_ac_adapter_info(&cfg.acpi_path.join("power_supply")) {
                Ok(ac) => ac,
                Err(e) => {
                    eprintln!("Application error: {}", e);
                    std::process::exit(1);
                }
            };
        for ac in adapters {
            display_ac_adapter_info(&ac);
        }
    }
    if cfg.show_thermal_sensors {
        let sensors: Vec<acpi_client::ThermalSensor> =
            match acpi_client::get_thermal_sensor_info(&cfg.acpi_path.join("thermal"), cfg.units) {
                Ok(tz) => tz,
                Err(e) => {
                    eprintln!("Application error: {}", e);
                    std::process::exit(1);
                }
            };
        for tz in sensors {
            display_thermal_zone_info(&tz, cfg.detailed);
        }
    }
    if cfg.show_cooling_devices {
        let devices: Vec<acpi_client::CoolingDevice> =
            match acpi_client::get_cooling_device_info(&cfg.acpi_path.join("thermal")) {
                Ok(cd) => cd,
                Err(e) => {
                    eprintln!("Application error: {}", e);
                    std::process::exit(1);
                }
            };
        for cd in devices {
            display_cooling_device_info(&cd);
        }
    }

    Ok(())
}

fn display_battery_info(bat: &acpi_client::BatteryInfo, detailed: bool) {
    let state = match &bat.state {
        acpi_client::ChargingState::Charging => "Charging",
        acpi_client::ChargingState::Discharging => "Discharging",
        acpi_client::ChargingState::Full => "Full",
    };
    let mut seconds = bat.time_remaining.as_secs();
    let hours = seconds / 3600;
    seconds = seconds - hours * 3600;
    let minutes = seconds / 60;
    seconds = seconds - minutes * 60;
    let not_full_string = format!(", {:02}:{:02}:{:02}", hours, minutes, seconds);
    let charge_time_string = match &bat.state {
        acpi_client::ChargingState::Charging => {
            if bat.present_rate > 0 {
                format!("{} {}", not_full_string, "until charged")
            } else {
                format!(", charging at zero rate")
            }
        }
        acpi_client::ChargingState::Discharging => format!("{} {}", not_full_string, "remaining"),
        _ => String::from(""),
    };
    println!(
        "{}: {}, {:.1}%{}",
        &bat.name, state, bat.percentage, charge_time_string
    );

    if detailed {
        println!(
            "{}: design capacity {} mAh, last full capacity {} mAh = {}%",
            &bat.name,
            bat.design_capacity,
            bat.last_capacity,
            (100 * bat.last_capacity) / bat.design_capacity
        );
    }
}

fn display_ac_adapter_info(ac: &acpi_client::ACAdapterInfo) {
    let status_str = match ac.status {
        acpi_client::Status::Online => "online",
        acpi_client::Status::Offline => "offline",
    };
    println!("{}: {}", &ac.name, status_str);
}

fn display_thermal_zone_info(tz: &acpi_client::ThermalSensor, detailed: bool) {
    let temperature_str = match tz.units {
        acpi_client::Units::Celsius => "degrees C",
        acpi_client::Units::Fahrenheit => "degrees F",
        acpi_client::Units::Kelvin => "kelvin",
    };
    println!(
        "{}: {:.1} {}",
        &tz.name, tz.current_temperature, temperature_str
    );

    if detailed {
        for tp in &tz.trip_points {
            println!("{}: trip point {} switches to mode {} at temperature {:.1} {}", &tz.name, tp.number, &tp.action_type, tp.temperature, temperature_str);
        }
    }
}

fn display_cooling_device_info(cd: &acpi_client::CoolingDevice) {
    let state_str = if cd.state.is_some() {
        format!("{} of {}", cd.state.unwrap().current_state, cd.state.unwrap().max_state)
    } else {
        format!("no state information available")
    };
    println!("{}: {} {}", &cd.name, &cd.device_type, state_str);
}
