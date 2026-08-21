// SPDX-License-Identifier: Apache-2.0

//! Background daemon that manages TCP transport, peer discovery, IPC, and
//! the main event loop for an agent.

use crate::{discovery, ipc, protocol, storage, transport};
use protocol::{AgentInfo, IpcRequest, IpcResponse, Message, Target};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

/// Start the daemon for the given agent name (blocking).
pub fn run(name: &str) {
    // Init storage
    storage::init_storage(name).expect("failed to init storage");

    // Write PID file
    let agent_dir = storage::agent_dir(name);
    let pid = std::process::id();
    std::fs::write(agent_dir.join("daemon.pid"), pid.to_string()).expect("failed to write pid");

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(run_async(name));

    // Cleanup
    let _ = std::fs::remove_file(agent_dir.join("daemon.pid"));
    let _ = std::fs::remove_file(agent_dir.join("daemon.sock"));
}

/// Shared context passed to the IPC handler to avoid too many arguments.
struct IpcContext {
    name: String,
    peers: Arc<RwLock<std::collections::HashMap<String, AgentInfo>>>,
    groups: Arc<RwLock<HashSet<String>>>,
    start_time: std::time::Instant,
    send_tx: mpsc::Sender<(Target, String)>,
    shutdown_tx: mpsc::Sender<()>,
    tcp_port: u16,
    discovery: Arc<discovery::Discovery>,
}

impl IpcContext {
    /// Snapshot the current groups as a `Vec`.
    fn groups_vec(&self) -> Vec<String> {
        snapshot_groups(&self.groups)
    }

    /// Mutate the groups set, persist to disk, and update mDNS.
    fn modify_groups(&self, f: impl FnOnce(&mut HashSet<String>)) {
        {
            let mut g = self.groups.write().expect("lock poisoned");
            f(&mut g);
            save_groups(&self.name, &g);
        }
        let _ = self
            .discovery
            .update_groups(self.tcp_port, &self.groups_vec());
    }
}

/// Persisted agent configuration.
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct AgentConfig {
    #[serde(default)]
    groups: HashSet<String>,
}

/// Read the current groups set as a `Vec<String>`.
fn snapshot_groups(groups: &Arc<RwLock<HashSet<String>>>) -> Vec<String> {
    groups
        .read()
        .expect("lock poisoned")
        .iter()
        .cloned()
        .collect()
}

/// Load persisted groups from the agent config file.
fn load_groups(name: &str) -> HashSet<String> {
    let config_path = storage::agent_dir(name).join("config.json");
    std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|data| serde_json::from_str::<AgentConfig>(&data).ok())
        .map(|cfg| cfg.groups)
        .unwrap_or_default()
}

/// Spawn the mDNS event processor and periodic local peer scanner.
fn spawn_discovery_tasks(
    discovery: &Arc<discovery::Discovery>,
    peers: &Arc<RwLock<std::collections::HashMap<String, AgentInfo>>>,
) {
    // Spawn mDNS event processor if available
    if let Some(browse_rx) = discovery.browse() {
        let disc = discovery.clone();
        tokio::task::spawn_blocking(move || {
            while let Ok(event) = browse_rx.recv() {
                disc.handle_mdns_event(event);
            }
        });
    }

    // Periodic local peer scan (every 2 seconds)
    let disc = discovery.clone();
    let peers = peers.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            let local_peers = disc.scan_local_peers();
            if let Ok(mut p) = peers.write() {
                p.clear();
                for agent in local_peers {
                    p.insert(agent.name.clone(), agent);
                }
            }
        }
    });
}

async fn run_async(name: &str) {
    let name = name.to_string();
    let start_time = std::time::Instant::now();

    let groups: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(load_groups(&name)));

    // Start TCP listener
    let (msg_tx, mut msg_rx) = mpsc::channel::<Message>(256);
    let (tcp_port, _tcp_handle) = transport::start_listener(msg_tx)
        .await
        .expect("failed to start TCP listener");

    // Start discovery (local registry + optional mDNS)
    let groups_vec = snapshot_groups(&groups);
    let discovery = Arc::new(
        discovery::Discovery::new(&name, tcp_port, &groups_vec).expect("failed to start discovery"),
    );
    let peers = discovery.peers.clone();

    spawn_discovery_tasks(&discovery, &peers);

    // Start IPC server
    let sock_path = storage::agent_dir(&name).join("daemon.sock");
    let (send_tx, mut ipc_send_rx) = mpsc::channel::<(Target, String)>(64);
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    let ctx = Arc::new(IpcContext {
        name: name.clone(),
        peers: peers.clone(),
        groups: groups.clone(),
        start_time,
        send_tx,
        shutdown_tx,
        tcp_port,
        discovery: discovery.clone(),
    });

    let ipc_handle = tokio::spawn(async move {
        let _ = ipc::run_server(&sock_path, move |req| handle_ipc(&ctx, req)).await;
    });

    // Main event loop
    loop {
        tokio::select! {
            Some(msg) = msg_rx.recv() => {
                let _ = storage::append_message(&name, &msg);
            }
            Some((to, content)) = ipc_send_rx.recv() => {
                let msg = make_message(&name, to, &content);
                let _ = storage::append_message(&name, &msg);
                deliver(&msg, &peers);
            }
            _ = shutdown_rx.recv() => {
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    // Cleanup
    ipc_handle.abort();
    if let Ok(d) = Arc::try_unwrap(discovery) {
        d.shutdown();
    }
}

/// Convert a storage read result into an `IpcResponse::Messages`.
fn messages_response(result: std::io::Result<(Vec<Message>, usize)>, label: &str) -> IpcResponse {
    match result {
        Ok((msgs, total)) => IpcResponse::Messages {
            truncated: msgs.len() < total,
            msgs,
            total,
        },
        Err(e) => IpcResponse::Error(format!("{label}: {e}")),
    }
}

fn handle_ipc(ctx: &IpcContext, req: IpcRequest) -> IpcResponse {
    match req {
        IpcRequest::Ping => IpcResponse::Ok { id: None },

        IpcRequest::ListAgents => {
            let agents: Vec<AgentInfo> = ctx
                .peers
                .read()
                .expect("lock poisoned")
                .values()
                .cloned()
                .collect();
            IpcResponse::Agents(agents)
        }

        IpcRequest::Send { to, content } => {
            let id = uuid::Uuid::new_v4().to_string();
            let _ = ctx.send_tx.try_send((to, content));
            IpcResponse::Ok { id: Some(id) }
        }

        IpcRequest::Inbox { limit } => {
            messages_response(storage::read_inbox(&ctx.name, limit), "read inbox")
        }

        IpcRequest::Feed { limit } => {
            messages_response(storage::read_feed(&ctx.name, limit), "read feed")
        }

        IpcRequest::Log { target, limit } => messages_response(
            storage::read_log(&ctx.name, target.as_deref(), limit),
            "read log",
        ),

        IpcRequest::JoinGroup { group } => {
            ctx.modify_groups(|g| {
                g.insert(group);
            });
            IpcResponse::Ok { id: None }
        }

        IpcRequest::LeaveGroup { group } => {
            ctx.modify_groups(|g| {
                g.remove(&group);
            });
            IpcResponse::Ok { id: None }
        }

        IpcRequest::Status => {
            let uptime = ctx.start_time.elapsed().as_secs();
            let groups = ctx.groups_vec();
            let peers = ctx.peers.read().expect("lock poisoned").len();
            IpcResponse::Status {
                name: ctx.name.clone(),
                uptime_secs: uptime,
                groups,
                peers,
            }
        }

        IpcRequest::Shutdown => {
            let _ = ctx.shutdown_tx.try_send(());
            IpcResponse::Ok { id: None }
        }

        IpcRequest::Help => IpcResponse::Help {
            commands: protocol::command_list(),
        },
    }
}

fn make_message(from: &str, to: Target, content: &str) -> Message {
    Message {
        id: uuid::Uuid::new_v4().to_string(),
        from: from.to_string(),
        to,
        content: content.to_string(),
        ts: chrono::Utc::now().to_rfc3339(),
    }
}

/// Deliver a message to the appropriate peers based on its target.
fn deliver(msg: &Message, peers: &Arc<RwLock<std::collections::HashMap<String, AgentInfo>>>) {
    let targets: Vec<SocketAddr> = {
        let p = peers.read().expect("lock poisoned");
        match &msg.to {
            Target::Direct(name) => p
                .get(name)
                .and_then(AgentInfo::socket_addr)
                .into_iter()
                .collect(),
            Target::Group(group) => p
                .values()
                .filter(|a| a.groups.contains(group))
                .filter_map(AgentInfo::socket_addr)
                .collect(),
            Target::Broadcast => p.values().filter_map(AgentInfo::socket_addr).collect(),
        }
    };
    for addr in targets {
        let msg = msg.clone();
        tokio::spawn(async move {
            let _ = transport::send_message(addr, &msg).await;
        });
    }
}

fn save_groups(name: &str, groups: &HashSet<String>) {
    let config_path = storage::agent_dir(name).join("config.json");
    let cfg = AgentConfig {
        groups: groups.clone(),
    };
    let _ = std::fs::write(
        config_path,
        serde_json::to_string_pretty(&cfg).expect("serialize config"),
    );
}
