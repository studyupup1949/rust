use super::error::CliError;

pub struct PortMapping {
    pub host_port: u16,
    pub device_port: u16,
}

impl PortMapping {
    pub fn new(mapping: &str) -> Result<Self, CliError> {
        let parts: Vec<&str> = mapping.split(':').collect();
        if parts.len() != 2 {
            return Err(CliError::InvalidPortMapping(mapping.to_string()));
        }

        let device_port = parts[0]
            .parse::<u16>()
            .map_err(|_| CliError::InvalidPortMapping(mapping.to_string()))?;
        let host_port = parts[1]
            .parse::<u16>()
            .map_err(|_| CliError::InvalidPortMapping(mapping.to_string()))?;

        Ok(PortMapping {
            host_port,
            device_port,
        })
    }
}
