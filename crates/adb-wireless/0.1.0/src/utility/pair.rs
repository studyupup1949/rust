use std::net::SocketAddr;

use super::error::CliError;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use rand::Rng;

const SERVICE_TYPE: &str = "_adb-tls-pairing._tcp.local.";

pub struct PairService {
    pub service_name: String,
    pub password: String,
    mdns: ServiceDaemon,
}

fn random_number_string(length: usize) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| rng.random_range(0..10).to_string())
        .collect()
}

impl PairService {
    pub fn new() -> Result<Self, CliError> {
        let mdns = ServiceDaemon::new()?;

        let service_name = format!("adb-wireless-{}", random_number_string(6));
        let password = random_number_string(8);

        Ok(Self {
            service_name,
            password,
            mdns,
        })
    }

    pub fn qrtext(&self) -> String {
        format!("WIFI:T:ADB;S:{};P:{};;", self.service_name, self.password)
    }

    pub fn start_discovery(&self) -> Result<(), CliError> {
        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &self.service_name,
            &format!("{}.local.", self.service_name),
            "",
            0,
            None,
        )?;

        self.mdns.register(service_info)?;

        Ok(())
    }

    pub fn wait_for_pairing(&self) -> Result<SocketAddr, CliError> {
        let receiver = self.mdns.browse(SERVICE_TYPE)?;

        loop {
            match receiver.recv() {
                Ok(ServiceEvent::ServiceResolved(info)) => {
                    if info.get_fullname().contains(&self.service_name) {
                        let client_addresses = info.get_addresses();
                        let port = info.get_port();
                        let _ = self.mdns.stop_browse(SERVICE_TYPE);
                        if let Some(addr) = client_addresses.iter().next() {
                            return Ok(SocketAddr::new(*addr, port));
                        } else {
                            return Err(CliError::MdnsError(mdns_sd::Error::Msg(
                                "No client address found".to_string(),
                            )));
                        }
                    }
                }
                Ok(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
                Err(err) => {
                    let _ = self.mdns.stop_browse(SERVICE_TYPE);
                    return Err(CliError::UnexpectedError(err.to_string()));
                }
            }
        }
    }
}

impl Drop for PairService {
    fn drop(&mut self) {
        let _ = self.mdns.stop_browse(SERVICE_TYPE);
        let _ = self.mdns.unregister(&self.service_name);
    }
}
