// SPDX-License-Identifier: Apache-2.0

//! Peer discovery via local registry files and optional mDNS.
//!
//! Each agent writes `~/.achat/registry/<name>.json` on startup and removes it
//! on shutdown. Peers are discovered by scanning this directory periodically.
//! mDNS is attempted for cross-machine discovery but is optional.

use crate::protocol::AgentInfo;
use crate::{storage, util};
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};

/// Manages peer discovery through local registry files and optional mDNS.
pub struct Discovery {
    /// Shared map of discovered peers, keyed by agent name.
    pub peers: Arc<RwLock<HashMap<String, AgentInfo>>>,
    instance_name: String,
    mdns: Option<mdns_sd::ServiceDaemon>,
}

const SERVICE_TYPE: &str = "_achat._tcp.local.";

impl Discovery {
    /// Create a new discovery instance, register in the local registry, and
    /// optionally start an mDNS service.
    pub fn new(name: &str, port: u16, groups: &[String]) -> io::Result<Self> {
        let peers = Arc::new(RwLock::new(HashMap::new()));

        // Write local registry entry
        write_registry(name, port, groups)?;

        // Try mDNS (may fail on port conflict with another achat daemon on same machine)
        let mdns = match mdns_sd::ServiceDaemon::new() {
            Ok(daemon) => {
                if let Ok(svc) = make_mdns_service(name, port, groups) {
                    let _ = daemon.register(svc);
                }
                Some(daemon)
            }
            Err(_) => None, // mDNS unavailable, rely on local registry only
        };

        Ok(Self {
            peers,
            instance_name: name.to_string(),
            mdns,
        })
    }

    /// Scan the local registry and return all live peers.
    pub fn scan_local_peers(&self) -> Vec<AgentInfo> {
        let registry_dir = storage::base_dir().join("registry");
        let Ok(entries) = std::fs::read_dir(&registry_dir) else {
            return vec![];
        };
        let mut result = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Some(info) = read_registry_entry(&path) else {
                continue;
            };
            if info.name == self.instance_name {
                continue;
            }
            if peer_is_alive(&info.name) {
                result.push(info);
            } else {
                let _ = std::fs::remove_file(&path);
            }
        }
        result
    }

    /// Start mDNS browsing if available. Returns a receiver or `None`.
    pub fn browse(&self) -> Option<mdns_sd::Receiver<mdns_sd::ServiceEvent>> {
        self.mdns.as_ref().and_then(|m| m.browse(SERVICE_TYPE).ok())
    }

    /// Process a single mDNS event.
    pub fn handle_mdns_event(&self, event: mdns_sd::ServiceEvent) {
        match event {
            mdns_sd::ServiceEvent::ServiceResolved(info) => {
                let name = info
                    .get_fullname()
                    .split('.')
                    .next()
                    .unwrap_or("")
                    .to_string();
                if name == self.instance_name {
                    return;
                }
                let addr = info
                    .get_addresses()
                    .iter()
                    .next()
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default();
                let port = info.get_port();
                let groups: Vec<String> = info
                    .get_property_val_str("groups")
                    .unwrap_or("")
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(std::string::ToString::to_string)
                    .collect();
                let agent = AgentInfo {
                    name: name.clone(),
                    addr,
                    port,
                    groups,
                };
                if let Ok(mut peers) = self.peers.write() {
                    peers.insert(name, agent);
                }
            }
            mdns_sd::ServiceEvent::ServiceRemoved(_, fullname) => {
                let name = fullname.split('.').next().unwrap_or("").to_string();
                if let Ok(mut peers) = self.peers.write() {
                    peers.remove(&name);
                }
            }
            mdns_sd::ServiceEvent::SearchStarted(_)
            | mdns_sd::ServiceEvent::ServiceFound(..)
            | mdns_sd::ServiceEvent::SearchStopped(_) => {}
        }
    }

    /// Update registry and mDNS with new groups.
    pub fn update_groups(&self, port: u16, groups: &[String]) -> io::Result<()> {
        write_registry(&self.instance_name, port, groups)?;
        if let Some(ref mdns) = self.mdns {
            if let Ok(svc) = make_mdns_service(&self.instance_name, port, groups) {
                let _ = mdns.register(svc);
            }
        }
        Ok(())
    }

    /// Remove the local registry entry and unregister from mDNS.
    pub fn unregister(&self) {
        let path = storage::base_dir()
            .join("registry")
            .join(format!("{}.json", self.instance_name));
        let _ = std::fs::remove_file(path);

        if let Some(ref mdns) = self.mdns {
            let fullname = format!("{}.{}", self.instance_name, SERVICE_TYPE);
            let _ = mdns.unregister(&fullname);
        }
    }

    /// Unregister and shut down the mDNS daemon.
    pub fn shutdown(self) {
        self.unregister();
        if let Some(mdns) = self.mdns {
            let _ = mdns.shutdown();
        }
    }
}

fn write_registry(name: &str, port: u16, groups: &[String]) -> io::Result<()> {
    let registry_dir = storage::base_dir().join("registry");
    std::fs::create_dir_all(&registry_dir)?;

    let info = AgentInfo {
        name: name.to_string(),
        addr: "127.0.0.1".to_string(),
        port,
        groups: groups.to_vec(),
    };
    let path = registry_dir.join(format!("{name}.json"));
    let json = serde_json::to_string_pretty(&info)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// Try to read and deserialize a registry JSON file.
fn read_registry_entry(path: &std::path::Path) -> Option<AgentInfo> {
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Check if a peer agent's daemon process is still running.
fn peer_is_alive(name: &str) -> bool {
    let pid_file = storage::agent_dir(name).join("daemon.pid");
    std::fs::read_to_string(&pid_file)
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .is_some_and(util::is_process_alive)
}

/// Build an mDNS `ServiceInfo` for registration.
fn make_mdns_service(
    name: &str,
    port: u16,
    groups: &[String],
) -> Result<mdns_sd::ServiceInfo, mdns_sd::Error> {
    let host = format!("{name}.local.");
    let mut properties = HashMap::new();
    properties.insert("v".to_string(), "1".to_string());
    if !groups.is_empty() {
        properties.insert("groups".to_string(), groups.join(","));
    }
    mdns_sd::ServiceInfo::new(SERVICE_TYPE, name, &host, "", port, properties)
}
