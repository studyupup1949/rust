pub const MAX_TOPOLOGY_SIZE: usize = 10 * 1024 * 1024;
pub const MAX_PEER_SNAPSHOT_SIZE: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct TopologyConfig {
    pub local_roots: Vec<LocalRoot>,
    pub public_roots: Vec<PublicRoot>,
    pub seed_peers: Vec<PeerAddress>,
    pub use_bootstrap_after_slot: i64,
    pub peer_snapshot_file: Option<String>,
    pub peer_snapshot: Option<PeerSnapshot>,
}

impl TopologyConfig {
    pub fn summary(&self) -> TopologySummary {
        let local_peers = self.local_roots.iter().map(|root| root.peers.len()).sum();
        let public_peers = self.public_roots.iter().map(|root| root.peers.len()).sum();
        let bootstrap_peers = self.seed_peers.len();
        let total_peer_entries = local_peers + public_peers + bootstrap_peers;
        let mut unique_peers = std::collections::HashSet::with_capacity(total_peer_entries);
        for root in &self.local_roots {
            unique_peers.extend(root.peers.iter());
        }
        for root in &self.public_roots {
            unique_peers.extend(root.peers.iter());
        }
        unique_peers.extend(self.seed_peers.iter());
        let ledger_peers_enabled = self.use_bootstrap_after_slot >= 0;
        let peer_snapshot_configured =
            self.peer_snapshot_file.is_some() || self.peer_snapshot.is_some();
        let peer_snapshot_usable_by_topology = peer_snapshot_configured && ledger_peers_enabled;
        let trustable_local_peers = self
            .local_roots
            .iter()
            .filter(|root| root.trustable)
            .map(|root| root.peers.len())
            .sum();

        TopologySummary {
            local_roots: self.local_roots.len(),
            public_roots: self.public_roots.len(),
            local_peers,
            public_peers,
            bootstrap_peers,
            unique_peers: unique_peers.len(),
            duplicate_peer_entries: total_peer_entries.saturating_sub(unique_peers.len()),
            local_valency: self
                .local_roots
                .iter()
                .map(|root| root.valency as usize)
                .sum(),
            public_valency: self
                .public_roots
                .iter()
                .map(|root| root.valency as usize)
                .sum(),
            local_warm_valency: self
                .local_roots
                .iter()
                .map(|root| root.warm_valency as usize)
                .sum(),
            public_warm_valency: self
                .public_roots
                .iter()
                .map(|root| root.warm_valency as usize)
                .sum(),
            advertised_local_roots: self
                .local_roots
                .iter()
                .filter(|root| root.advertise)
                .count(),
            advertised_public_roots: self
                .public_roots
                .iter()
                .filter(|root| root.advertise)
                .count(),
            trustable_local_roots: self
                .local_roots
                .iter()
                .filter(|root| root.trustable)
                .count(),
            empty_local_roots: self
                .local_roots
                .iter()
                .filter(|root| root.peers.is_empty())
                .count(),
            empty_public_roots: self
                .public_roots
                .iter()
                .filter(|root| root.peers.is_empty())
                .count(),
            trustable_local_peers,
            peer_snapshot_configured,
            ledger_peers_enabled,
            bootstrap_peers_configured: !self.seed_peers.is_empty(),
            peer_snapshot_usable_by_topology,
            trusted_peer_source_configured: trustable_local_peers > 0
                || !self.seed_peers.is_empty()
                || peer_snapshot_usable_by_topology,
        }
    }

    pub fn without_seed_peers(&self) -> Self {
        let mut copy = self.clone();
        copy.seed_peers.clear();
        copy
    }

    pub fn with_peer_snapshot_fixture(mut self, input: &str) -> Result<Self, TopologyError> {
        self.peer_snapshot = Some(parse_peer_snapshot_fixture(input)?);
        Ok(self)
    }

    pub fn with_cardano_peer_snapshot_json(mut self, input: &str) -> Result<Self, TopologyError> {
        self.peer_snapshot = Some(parse_cardano_peer_snapshot_json(input)?);
        Ok(self)
    }

    pub fn validate_valencies(&self) -> Result<(), TopologyError> {
        for root in &self.local_roots {
            validate_root_valency("local", root.peers.len(), root.valency, root.warm_valency)?;
        }
        for root in &self.public_roots {
            validate_root_valency("public", root.peers.len(), root.valency, root.warm_valency)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopologySummary {
    pub local_roots: usize,
    pub public_roots: usize,
    pub local_peers: usize,
    pub public_peers: usize,
    pub bootstrap_peers: usize,
    pub unique_peers: usize,
    pub duplicate_peer_entries: usize,
    pub local_valency: usize,
    pub public_valency: usize,
    pub local_warm_valency: usize,
    pub public_warm_valency: usize,
    pub advertised_local_roots: usize,
    pub advertised_public_roots: usize,
    pub trustable_local_roots: usize,
    pub empty_local_roots: usize,
    pub empty_public_roots: usize,
    pub trustable_local_peers: usize,
    pub peer_snapshot_configured: bool,
    pub ledger_peers_enabled: bool,
    pub bootstrap_peers_configured: bool,
    pub peer_snapshot_usable_by_topology: bool,
    pub trusted_peer_source_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeerAddress {
    pub address: String,
    pub port: u16,
}

impl PeerAddress {
    pub fn new(address: impl Into<String>, port: u16) -> Self {
        Self {
            address: address.into(),
            port,
        }
    }

    pub fn parse(value: &str) -> Result<Self, TopologyError> {
        let (address, port) = value
            .rsplit_once(':')
            .ok_or_else(|| TopologyError::InvalidPeer(value.to_string()))?;
        if address.trim().is_empty() {
            return Err(TopologyError::InvalidPeer(value.to_string()));
        }
        let port = port
            .parse::<u16>()
            .map_err(|_| TopologyError::InvalidPeer(value.to_string()))?;
        Ok(Self::new(address.trim(), port))
    }
}

impl std::str::FromStr for PeerAddress {
    type Err = TopologyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRoot {
    pub peers: Vec<PeerAddress>,
    pub advertise: bool,
    pub trustable: bool,
    pub valency: u16,
    pub warm_valency: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicRoot {
    pub peers: Vec<PeerAddress>,
    pub advertise: bool,
    pub valency: u16,
    pub warm_valency: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeerSnapshot {
    pub network_magic: u32,
    pub client_version: u16,
    pub point: PeerSnapshotPoint,
    pub priority_pools: Vec<PeerPool>,
    pub pools: Vec<PeerPool>,
}

impl PeerSnapshot {
    pub fn relay_peers(&self) -> Vec<PeerAddress> {
        let mut peers = Vec::with_capacity(self.relay_count());
        for pool in &self.priority_pools {
            peers.extend(pool.relays.iter().cloned());
        }
        for pool in &self.pools {
            peers.extend(pool.relays.iter().cloned());
        }
        peers
    }

    pub fn has_relays(&self) -> bool {
        self.relay_count() > 0
    }

    fn relay_count(&self) -> usize {
        self.priority_pools
            .iter()
            .map(|pool| pool.relays.len())
            .sum::<usize>()
            + self
                .pools
                .iter()
                .map(|pool| pool.relays.len())
                .sum::<usize>()
    }

    pub fn validate(&self, rules: &PeerSnapshotRules) -> Result<(), PeerSnapshotError> {
        if let Some(expected) = rules.network_magic {
            if self.network_magic != expected {
                return Err(PeerSnapshotError::NetworkMismatch {
                    expected,
                    actual: self.network_magic,
                });
            }
        }
        if self.point.point_hash.trim().is_empty() {
            return Err(PeerSnapshotError::EmptyPointHash);
        }
        let pool_count = self.priority_pools.len() + self.pools.len();
        if pool_count > rules.max_pools {
            return Err(PeerSnapshotError::TooManyPools {
                max: rules.max_pools,
                actual: pool_count,
            });
        }
        let relay_count = self.relay_count();
        if relay_count > rules.max_relays {
            return Err(PeerSnapshotError::TooManyRelays {
                max: rules.max_relays,
                actual: relay_count,
            });
        }
        let mut relays = std::collections::HashSet::new();
        for pool in self.priority_pools.iter().chain(self.pools.iter()) {
            validate_pool(pool, &mut relays)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerSnapshotRules {
    pub network_magic: Option<u32>,
    pub max_pools: usize,
    pub max_relays: usize,
}

impl Default for PeerSnapshotRules {
    fn default() -> Self {
        Self {
            network_magic: None,
            max_pools: 10_000,
            max_relays: 50_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerSnapshotPoint {
    pub point_hash: String,
    pub point_slot: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeerPool {
    pub accumulated_stake: f64,
    pub relative_stake: f64,
    pub relays: Vec<PeerAddress>,
}

fn validate_pool(
    pool: &PeerPool,
    relays: &mut std::collections::HashSet<PeerAddress>,
) -> Result<(), PeerSnapshotError> {
    if !pool.accumulated_stake.is_finite()
        || !pool.relative_stake.is_finite()
        || !(0.0..=1.0).contains(&pool.accumulated_stake)
        || !(0.0..=1.0).contains(&pool.relative_stake)
    {
        return Err(PeerSnapshotError::InvalidStake);
    }
    for peer in &pool.relays {
        if peer.address.trim().is_empty() || peer.port == 0 {
            return Err(PeerSnapshotError::InvalidRelay(peer.clone()));
        }
        if !relays.insert(peer.clone()) {
            return Err(PeerSnapshotError::DuplicateRelay(peer.clone()));
        }
    }
    Ok(())
}

pub fn parse_seed_peers(input: &str) -> Result<Vec<PeerAddress>, TopologyError> {
    let mut peers = Vec::new();
    for (line_no, raw_line) in input.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let value = line
            .strip_prefix("peer=")
            .or_else(|| line.strip_prefix("seed="))
            .ok_or(TopologyError::InvalidLine { line: line_no + 1 })?;
        peers.push(PeerAddress::parse(value)?);
    }
    Ok(peers)
}

pub fn parse_cardano_topology_json(input: &str) -> Result<TopologyConfig, TopologyError> {
    if input.len() > MAX_TOPOLOGY_SIZE {
        return Err(TopologyError::InvalidJson(format!(
            "topology too large: max={} actual={}",
            MAX_TOPOLOGY_SIZE,
            input.len()
        )));
    }

    let root = JsonParser::new(input).parse()?;
    let object = root.as_object("topology")?;
    let mut topology = TopologyConfig {
        local_roots: Vec::new(),
        public_roots: Vec::new(),
        seed_peers: Vec::new(),
        use_bootstrap_after_slot: -1,
        peer_snapshot_file: None,
        peer_snapshot: None,
    };

    if let Some(value) = json_object_get(object, "useLedgerAfterSlot") {
        if !value.is_null() {
            topology.use_bootstrap_after_slot = value.as_i64("useLedgerAfterSlot")?;
        }
    }
    if let Some(value) = json_object_get(object, "peerSnapshotFile") {
        if !value.is_null() {
            topology.peer_snapshot_file = Some(value.as_str("peerSnapshotFile")?.to_string());
        }
    }
    if let Some(value) = json_object_get(object, "bootstrapPeers") {
        if !value.is_null() {
            topology.seed_peers = parse_cardano_access_point_array(value, "bootstrapPeers")?;
        }
    }
    if let Some(value) = json_object_get(object, "localRoots") {
        if !value.is_null() {
            for root in value.as_array("localRoots")? {
                topology.local_roots.push(parse_cardano_local_root(root)?);
            }
        }
    }
    if let Some(value) = json_object_get(object, "publicRoots") {
        if !value.is_null() {
            for root in value.as_array("publicRoots")? {
                topology.public_roots.push(parse_cardano_public_root(root)?);
            }
        }
    }
    if let Some(value) = json_object_get(object, "Producers") {
        if !value.is_null() {
            for producer in value.as_array("Producers")? {
                topology
                    .public_roots
                    .push(parse_legacy_producer_root(producer)?);
            }
        }
    }

    topology.validate_valencies()?;
    Ok(topology)
}

pub fn parse_cardano_peer_snapshot_json(input: &str) -> Result<PeerSnapshot, TopologyError> {
    if input.len() > MAX_PEER_SNAPSHOT_SIZE {
        return Err(TopologyError::InvalidJson(format!(
            "peer snapshot too large: max={} actual={}",
            MAX_PEER_SNAPSHOT_SIZE,
            input.len()
        )));
    }

    let root = JsonParser::new(input).parse()?;
    let object = root.as_object("peerSnapshot")?;
    let point = required_json_field(object, "Point")?.as_object("Point")?;

    Ok(PeerSnapshot {
        network_magic: required_json_field(object, "NetworkMagic")?.as_u32("NetworkMagic")?,
        client_version: required_json_field(object, "NodeToClientVersion")?
            .as_u16("NodeToClientVersion")?,
        point: PeerSnapshotPoint {
            point_hash: required_json_field(point, "blockPointHash")?
                .as_str("Point.blockPointHash")?
                .to_string(),
            point_slot: required_json_field(point, "blockPointSlot")?
                .as_u64("Point.blockPointSlot")?,
        },
        priority_pools: parse_cardano_peer_snapshot_pool_array(
            json_object_get(object, "bigLedgerPools"),
            "bigLedgerPools",
        )?,
        pools: parse_cardano_peer_snapshot_pool_array(
            json_object_get(object, "ledgerPools"),
            "ledgerPools",
        )?,
    })
}

fn required_json_field<'a>(
    object: &'a JsonObject,
    name: &'static str,
) -> Result<&'a JsonValue, TopologyError> {
    json_object_get(object, name)
        .ok_or_else(|| TopologyError::InvalidJson(format!("missing {name}")))
}

fn parse_cardano_peer_snapshot_pool_array(
    value: Option<&JsonValue>,
    name: &'static str,
) -> Result<Vec<PeerPool>, TopologyError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if value.is_null() {
        return Ok(Vec::new());
    }
    value
        .as_array(name)?
        .iter()
        .map(|pool| parse_cardano_peer_snapshot_pool(pool, name))
        .collect()
}

fn parse_cardano_peer_snapshot_pool(
    value: &JsonValue,
    name: &'static str,
) -> Result<PeerPool, TopologyError> {
    let object = value.as_object(name)?;
    let relays = match json_object_get(object, "relays") {
        Some(value) if !value.is_null() => value
            .as_array("relays")?
            .iter()
            .map(parse_cardano_peer_snapshot_relay)
            .collect::<Result<Vec<_>, _>>()?,
        _ => Vec::new(),
    };
    Ok(PeerPool {
        accumulated_stake: json_object_get(object, "accumulatedStake")
            .map(|value| value.as_f64("accumulatedStake"))
            .transpose()?
            .unwrap_or(0.0),
        relative_stake: json_object_get(object, "relativeStake")
            .map(|value| value.as_f64("relativeStake"))
            .transpose()?
            .unwrap_or(0.0),
        relays,
    })
}

fn parse_cardano_peer_snapshot_relay(value: &JsonValue) -> Result<PeerAddress, TopologyError> {
    let object = value.as_object("relays[]")?;
    let address = json_object_get(object, "address")
        .map(|value| value.as_str("relays[].address"))
        .transpose()?
        .unwrap_or("");
    let port = json_object_get(object, "port")
        .map(|value| value.as_u16("relays[].port"))
        .transpose()?
        .unwrap_or(0);
    Ok(PeerAddress::new(address, port))
}

pub fn parse_topology_fixture(input: &str) -> Result<TopologyConfig, TopologyError> {
    if input.len() > MAX_TOPOLOGY_SIZE {
        return Err(TopologyError::InvalidLine { line: 0 });
    }

    let mut topology = TopologyConfig {
        local_roots: Vec::new(),
        public_roots: Vec::new(),
        seed_peers: Vec::new(),
        use_bootstrap_after_slot: -1,
        peer_snapshot_file: None,
        peer_snapshot: None,
    };
    for (line_no, raw_line) in input.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with("local=") || line.starts_with("local_root=") {
            topology
                .local_roots
                .push(parse_local_root(line, line_no + 1)?);
        } else if line.starts_with("public=") || line.starts_with("public_root=") {
            topology
                .public_roots
                .push(parse_public_root(line, line_no + 1)?);
        } else if let Some(value) = line
            .strip_prefix("peer=")
            .or_else(|| line.strip_prefix("seed="))
        {
            topology.seed_peers.push(PeerAddress::parse(value)?);
        } else if let Some(value) = line.strip_prefix("bootstrap_after=") {
            topology.use_bootstrap_after_slot = value
                .trim()
                .parse::<i64>()
                .map_err(|_| TopologyError::InvalidLine { line: line_no + 1 })?;
        } else if let Some(value) = line.strip_prefix("snapshot_file=") {
            topology.peer_snapshot_file = Some(value.trim().to_string());
        } else {
            return Err(TopologyError::InvalidLine { line: line_no + 1 });
        }
    }
    topology.validate_valencies()?;
    Ok(topology)
}

fn parse_cardano_local_root(value: &JsonValue) -> Result<LocalRoot, TopologyError> {
    let object = value.as_object("localRoots[]")?;
    let peers = match json_object_get(object, "accessPoints") {
        Some(value) => parse_cardano_access_point_array(value, "localRoots[].accessPoints")?,
        None => Vec::new(),
    };
    let valency = parse_cardano_root_valency(object, &peers, "local")?;
    Ok(LocalRoot {
        peers,
        advertise: json_object_get(object, "advertise")
            .map(|value| value.as_bool("localRoots[].advertise"))
            .transpose()?
            .unwrap_or(false),
        trustable: json_object_get(object, "trustable")
            .map(|value| value.as_bool("localRoots[].trustable"))
            .transpose()?
            .unwrap_or(true),
        valency,
        warm_valency: valency,
    })
}

fn parse_cardano_public_root(value: &JsonValue) -> Result<PublicRoot, TopologyError> {
    let object = value.as_object("publicRoots[]")?;
    let peers = match json_object_get(object, "accessPoints") {
        Some(value) => parse_cardano_access_point_array(value, "publicRoots[].accessPoints")?,
        None => Vec::new(),
    };
    let valency = parse_cardano_root_valency(object, &peers, "public")?;
    Ok(PublicRoot {
        peers,
        advertise: json_object_get(object, "advertise")
            .map(|value| value.as_bool("publicRoots[].advertise"))
            .transpose()?
            .unwrap_or(false),
        valency,
        warm_valency: valency,
    })
}

fn parse_legacy_producer_root(value: &JsonValue) -> Result<PublicRoot, TopologyError> {
    let object = value.as_object("Producers[]")?;
    let peer = parse_cardano_access_point_object(object, "Producers[]")?;
    let valency = match json_object_get(object, "valency") {
        Some(value) => value.as_u16("Producers[].valency")?,
        None => 1,
    };
    Ok(PublicRoot {
        peers: vec![peer],
        advertise: false,
        valency,
        warm_valency: valency,
    })
}

fn parse_cardano_root_valency(
    object: &JsonObject,
    peers: &[PeerAddress],
    root: &'static str,
) -> Result<u16, TopologyError> {
    let requested = match json_object_get(object, "valency") {
        Some(value) => value.as_u16("valency")?,
        None => default_valency(peers, 0)?,
    };
    if peers.is_empty() {
        return Ok(0);
    }
    validate_root_valency(root, peers.len(), requested, requested)?;
    Ok(requested)
}

fn parse_cardano_access_point_array(
    value: &JsonValue,
    name: &'static str,
) -> Result<Vec<PeerAddress>, TopologyError> {
    value
        .as_array(name)?
        .iter()
        .map(|value| {
            let object = value.as_object(name)?;
            parse_cardano_access_point_object(object, name)
        })
        .collect()
}

fn parse_cardano_access_point_object(
    object: &JsonObject,
    name: &'static str,
) -> Result<PeerAddress, TopologyError> {
    let address = json_object_get(object, "address")
        .or_else(|| json_object_get(object, "addr"))
        .ok_or_else(|| TopologyError::InvalidJson(format!("missing {name}.address")))?
        .as_str("address")?;
    let port = json_object_get(object, "port")
        .ok_or_else(|| TopologyError::InvalidJson(format!("missing {name}.port")))?
        .as_u16("port")?;
    if address.trim().is_empty() || port == 0 {
        return Err(TopologyError::InvalidPeer(format!("{address}:{port}")));
    }
    Ok(PeerAddress::new(address, port))
}

#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    Object(JsonObject),
    Array(Vec<JsonValue>),
    String(String),
    Number(String),
    Bool(bool),
    Null,
}

impl JsonValue {
    fn as_object(&self, name: &'static str) -> Result<&JsonObject, TopologyError> {
        match self {
            Self::Object(entries) => Ok(entries),
            _ => Err(TopologyError::InvalidJson(format!(
                "{name} must be an object"
            ))),
        }
    }

    fn as_array(&self, name: &'static str) -> Result<&[JsonValue], TopologyError> {
        match self {
            Self::Array(values) => Ok(values),
            _ => Err(TopologyError::InvalidJson(format!(
                "{name} must be an array"
            ))),
        }
    }

    fn as_str(&self, name: &'static str) -> Result<&str, TopologyError> {
        match self {
            Self::String(value) => Ok(value),
            _ => Err(TopologyError::InvalidJson(format!(
                "{name} must be a string"
            ))),
        }
    }

    fn as_bool(&self, name: &'static str) -> Result<bool, TopologyError> {
        match self {
            Self::Bool(value) => Ok(*value),
            _ => Err(TopologyError::InvalidJson(format!(
                "{name} must be a boolean"
            ))),
        }
    }

    fn as_i64(&self, name: &'static str) -> Result<i64, TopologyError> {
        match self {
            Self::Number(value) => value.parse::<i64>().map_err(|_| {
                TopologyError::InvalidJson(format!("{name} must be a signed integer"))
            }),
            _ => Err(TopologyError::InvalidJson(format!(
                "{name} must be a signed integer"
            ))),
        }
    }

    fn as_u32(&self, name: &'static str) -> Result<u32, TopologyError> {
        match self {
            Self::Number(value) => value.parse::<u32>().map_err(|_| {
                TopologyError::InvalidJson(format!("{name} must be a 32-bit unsigned integer"))
            }),
            _ => Err(TopologyError::InvalidJson(format!(
                "{name} must be a 32-bit unsigned integer"
            ))),
        }
    }

    fn as_u64(&self, name: &'static str) -> Result<u64, TopologyError> {
        match self {
            Self::Number(value) => value.parse::<u64>().map_err(|_| {
                TopologyError::InvalidJson(format!("{name} must be a 64-bit unsigned integer"))
            }),
            _ => Err(TopologyError::InvalidJson(format!(
                "{name} must be a 64-bit unsigned integer"
            ))),
        }
    }

    fn as_u16(&self, name: &'static str) -> Result<u16, TopologyError> {
        match self {
            Self::Number(value) => value.parse::<u16>().map_err(|_| {
                TopologyError::InvalidJson(format!("{name} must be a 16-bit unsigned integer"))
            }),
            _ => Err(TopologyError::InvalidJson(format!(
                "{name} must be a 16-bit unsigned integer"
            ))),
        }
    }

    fn as_f64(&self, name: &'static str) -> Result<f64, TopologyError> {
        match self {
            Self::Number(value) => value
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite())
                .ok_or_else(|| {
                    TopologyError::InvalidJson(format!("{name} must be a finite number"))
                }),
            _ => Err(TopologyError::InvalidJson(format!(
                "{name} must be a finite number"
            ))),
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

type JsonObject = std::collections::BTreeMap<String, JsonValue>;

fn json_object_get<'a>(entries: &'a JsonObject, key: &str) -> Option<&'a JsonValue> {
    entries.get(key)
}

struct JsonParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse(mut self) -> Result<JsonValue, TopologyError> {
        let value = self.parse_value()?;
        self.skip_ws();
        if self.peek().is_some() {
            return self.err("trailing input after JSON value");
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, TopologyError> {
        self.skip_ws();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('t') => {
                self.expect_word("true")?;
                Ok(JsonValue::Bool(true))
            }
            Some('f') => {
                self.expect_word("false")?;
                Ok(JsonValue::Bool(false))
            }
            Some('n') => {
                self.expect_word("null")?;
                Ok(JsonValue::Null)
            }
            Some('-' | '0'..='9') => self.parse_number().map(JsonValue::Number),
            Some(_) => self.err("unexpected JSON value"),
            None => self.err("expected JSON value"),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, TopologyError> {
        self.expect('{')?;
        let mut entries = JsonObject::new();
        loop {
            self.skip_ws();
            if self.consume('}') {
                break;
            }
            let key = self.parse_string()?;
            if entries.contains_key(&key) {
                return self.err(&format!("duplicate JSON field {key}"));
            }
            self.skip_ws();
            self.expect(':')?;
            let value = self.parse_value()?;
            entries.insert(key, value);
            self.skip_ws();
            if self.consume(',') {
                continue;
            }
            if self.consume('}') {
                break;
            }
            return self.err("expected ',' or '}'");
        }
        Ok(JsonValue::Object(entries))
    }

    fn parse_array(&mut self) -> Result<JsonValue, TopologyError> {
        self.expect('[')?;
        let mut values = Vec::new();
        loop {
            self.skip_ws();
            if self.consume(']') {
                break;
            }
            values.push(self.parse_value()?);
            self.skip_ws();
            if self.consume(',') {
                continue;
            }
            if self.consume(']') {
                break;
            }
            return self.err("expected ',' or ']'");
        }
        Ok(JsonValue::Array(values))
    }

    fn parse_string(&mut self) -> Result<String, TopologyError> {
        self.expect('"')?;
        let mut out = String::new();
        while let Some(ch) = self.next() {
            match ch {
                '"' => return Ok(out),
                '\\' => out.push(self.parse_escape()?),
                other => out.push(other),
            }
        }
        self.err("unterminated string")
    }

    fn parse_escape(&mut self) -> Result<char, TopologyError> {
        let Some(escaped) = self.next() else {
            return self.err("unterminated escape sequence");
        };
        match escaped {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '/' => Ok('/'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            other => self.err(&format!("unsupported escape sequence \\{other}")),
        }
    }

    fn parse_number(&mut self) -> Result<String, TopologyError> {
        let mut out = String::new();
        if self.consume('-') {
            out.push('-');
        }
        let mut digits = 0;
        while let Some(ch @ '0'..='9') = self.peek() {
            out.push(ch);
            self.next();
            digits += 1;
        }
        if digits == 0 {
            return self.err("invalid JSON number");
        }
        if self.consume('.') {
            out.push('.');
            let mut fraction_digits = 0;
            while let Some(ch @ '0'..='9') = self.peek() {
                out.push(ch);
                self.next();
                fraction_digits += 1;
            }
            if fraction_digits == 0 {
                return self.err("invalid JSON number fraction");
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            let exponent = self.next().expect("peeked exponent marker");
            out.push(exponent);
            if matches!(self.peek(), Some('+' | '-')) {
                let sign = self.next().expect("peeked exponent sign");
                out.push(sign);
            }
            let mut exponent_digits = 0;
            while let Some(ch @ '0'..='9') = self.peek() {
                out.push(ch);
                self.next();
                exponent_digits += 1;
            }
            if exponent_digits == 0 {
                return self.err("invalid JSON number exponent");
            }
        }
        Ok(out)
    }

    fn expect_word(&mut self, expected: &str) -> Result<(), TopologyError> {
        for expected_ch in expected.chars() {
            self.expect(expected_ch)?;
        }
        Ok(())
    }

    fn expect(&mut self, expected: char) -> Result<(), TopologyError> {
        if self.consume(expected) {
            Ok(())
        } else {
            self.err(&format!("expected '{expected}'"))
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.pos += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> Option<char> {
        let value = self.peek()?;
        self.pos += value.len_utf8();
        Some(value)
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos..)?.chars().next()
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.next();
        }
    }

    fn err<T>(&self, message: &str) -> Result<T, TopologyError> {
        Err(TopologyError::InvalidJson(format!(
            "{message} at byte {}",
            self.pos
        )))
    }
}

pub fn parse_peer_snapshot_fixture(input: &str) -> Result<PeerSnapshot, TopologyError> {
    if input.len() > MAX_PEER_SNAPSHOT_SIZE {
        return Err(TopologyError::InvalidLine { line: 0 });
    }

    let mut network_magic = None;
    let mut client_version = None;
    let mut point_hash = None;
    let mut point_slot = None;
    let mut priority_pools = Vec::new();
    let mut pools = Vec::new();

    for (line_no, raw_line) in input.lines().enumerate() {
        let line_no = line_no + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(value) = line.strip_prefix("network_magic=") {
            network_magic = Some(parse_u32_line(value, line_no)?);
        } else if let Some(value) = line.strip_prefix("client_version=") {
            client_version = Some(parse_u16_line(value, line_no)?);
        } else if let Some(value) = line.strip_prefix("point_hash=") {
            let value = value.trim();
            if value.is_empty() {
                return Err(TopologyError::InvalidLine { line: line_no });
            }
            point_hash = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("point_slot=") {
            point_slot = Some(parse_u64_line(value, line_no)?);
        } else if line.starts_with("priority_pool=") {
            priority_pools.push(parse_peer_pool_fixture(line, line_no)?);
        } else if line.starts_with("pool=") {
            pools.push(parse_peer_pool_fixture(line, line_no)?);
        } else {
            return Err(TopologyError::InvalidLine { line: line_no });
        }
    }

    Ok(PeerSnapshot {
        network_magic: network_magic.ok_or(TopologyError::InvalidLine { line: 0 })?,
        client_version: client_version.ok_or(TopologyError::InvalidLine { line: 0 })?,
        point: PeerSnapshotPoint {
            point_hash: point_hash.ok_or(TopologyError::InvalidLine { line: 0 })?,
            point_slot: point_slot.ok_or(TopologyError::InvalidLine { line: 0 })?,
        },
        priority_pools,
        pools,
    })
}

fn parse_peer_pool_fixture(line: &str, line_no: usize) -> Result<PeerPool, TopologyError> {
    let (relays, attrs) = parse_root_parts(line, line_no)?;
    Ok(PeerPool {
        accumulated_stake: parse_required_f64_attr(&attrs, "accumulated", line_no)?,
        relative_stake: parse_required_f64_attr(&attrs, "relative", line_no)?,
        relays,
    })
}

fn parse_local_root(line: &str, line_no: usize) -> Result<LocalRoot, TopologyError> {
    let (peers, attrs) = parse_root_parts(line, line_no)?;
    let valency = match parse_optional_u16_attr(&attrs, "valency", line_no)? {
        Some(value) => value,
        None => default_valency(&peers, line_no)?,
    };
    let warm_valency = parse_optional_u16_attr(&attrs, "warm", line_no)?
        .or(parse_optional_u16_attr(&attrs, "warm_valency", line_no)?)
        .unwrap_or(valency);
    Ok(LocalRoot {
        peers,
        advertise: parse_optional_bool_attr(&attrs, "advertise", line_no)?.unwrap_or(false),
        trustable: parse_optional_bool_attr(&attrs, "trustable", line_no)?.unwrap_or(true),
        valency,
        warm_valency,
    })
}

fn parse_public_root(line: &str, line_no: usize) -> Result<PublicRoot, TopologyError> {
    let (peers, attrs) = parse_root_parts(line, line_no)?;
    let valency = match parse_optional_u16_attr(&attrs, "valency", line_no)? {
        Some(value) => value,
        None => default_valency(&peers, line_no)?,
    };
    let warm_valency = parse_optional_u16_attr(&attrs, "warm", line_no)?
        .or(parse_optional_u16_attr(&attrs, "warm_valency", line_no)?)
        .unwrap_or(valency);
    Ok(PublicRoot {
        peers,
        advertise: parse_optional_bool_attr(&attrs, "advertise", line_no)?.unwrap_or(false),
        valency,
        warm_valency,
    })
}

type RootAttrs<'a> = Vec<(&'a str, &'a str)>;
type RootParts<'a> = (Vec<PeerAddress>, RootAttrs<'a>);

fn parse_root_parts(line: &str, line_no: usize) -> Result<RootParts<'_>, TopologyError> {
    let mut tokens = line.split_whitespace();
    let first = tokens
        .next()
        .ok_or(TopologyError::InvalidLine { line: line_no })?;
    let (_, peer_text) = first
        .split_once('=')
        .ok_or(TopologyError::InvalidLine { line: line_no })?;
    let peers = peer_text
        .split(',')
        .map(PeerAddress::parse)
        .collect::<Result<Vec<_>, _>>()?;
    let mut attrs = Vec::new();
    for token in tokens {
        let (key, value) = token
            .split_once('=')
            .ok_or(TopologyError::InvalidLine { line: line_no })?;
        attrs.push((key, value));
    }
    Ok((peers, attrs))
}

fn parse_optional_u16_attr(
    attrs: &[(&str, &str)],
    key: &str,
    line_no: usize,
) -> Result<Option<u16>, TopologyError> {
    let Some((_, value)) = attrs.iter().find(|(attr_key, _)| *attr_key == key) else {
        return Ok(None);
    };
    value
        .parse::<u16>()
        .map(Some)
        .map_err(|_| TopologyError::InvalidLine { line: line_no })
}

fn parse_required_f64_attr(
    attrs: &[(&str, &str)],
    key: &str,
    line_no: usize,
) -> Result<f64, TopologyError> {
    let Some((_, value)) = attrs.iter().find(|(attr_key, _)| *attr_key == key) else {
        return Err(TopologyError::InvalidLine { line: line_no });
    };
    value
        .parse::<f64>()
        .map_err(|_| TopologyError::InvalidLine { line: line_no })
}

fn parse_u16_line(value: &str, line_no: usize) -> Result<u16, TopologyError> {
    value
        .trim()
        .parse::<u16>()
        .map_err(|_| TopologyError::InvalidLine { line: line_no })
}

fn parse_u32_line(value: &str, line_no: usize) -> Result<u32, TopologyError> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|_| TopologyError::InvalidLine { line: line_no })
}

fn parse_u64_line(value: &str, line_no: usize) -> Result<u64, TopologyError> {
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| TopologyError::InvalidLine { line: line_no })
}

fn parse_optional_bool_attr(
    attrs: &[(&str, &str)],
    key: &str,
    line_no: usize,
) -> Result<Option<bool>, TopologyError> {
    let Some((_, value)) = attrs.iter().find(|(attr_key, _)| *attr_key == key) else {
        return Ok(None);
    };
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(Some(true)),
        "false" | "0" | "no" => Ok(Some(false)),
        _ => Err(TopologyError::InvalidLine { line: line_no }),
    }
}

fn validate_root_valency(
    root: &'static str,
    peer_count: usize,
    valency: u16,
    warm_valency: u16,
) -> Result<(), TopologyError> {
    if valency as usize > peer_count {
        return Err(TopologyError::InvalidValency {
            root,
            valency,
            peer_count,
        });
    }
    if warm_valency > valency {
        return Err(TopologyError::InvalidWarmValency {
            root,
            warm_valency,
            valency,
        });
    }
    Ok(())
}

fn default_valency(peers: &[PeerAddress], line_no: usize) -> Result<u16, TopologyError> {
    u16::try_from(peers.len()).map_err(|_| TopologyError::InvalidLine { line: line_no })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyError {
    InvalidPeer(String),
    InvalidJson(String),
    InvalidLine {
        line: usize,
    },
    InvalidValency {
        root: &'static str,
        valency: u16,
        peer_count: usize,
    },
    InvalidWarmValency {
        root: &'static str,
        warm_valency: u16,
        valency: u16,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PeerSnapshotError {
    NetworkMismatch { expected: u32, actual: u32 },
    EmptyPointHash,
    TooManyPools { max: usize, actual: usize },
    TooManyRelays { max: usize, actual: usize },
    InvalidStake,
    InvalidRelay(PeerAddress),
    DuplicateRelay(PeerAddress),
}

impl std::fmt::Display for PeerSnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkMismatch { expected, actual } => {
                write!(
                    f,
                    "peer snapshot network mismatch: expected={expected} actual={actual}"
                )
            }
            Self::EmptyPointHash => f.write_str("peer snapshot point hash is empty"),
            Self::TooManyPools { max, actual } => {
                write!(
                    f,
                    "peer snapshot has too many pools: max={max} actual={actual}"
                )
            }
            Self::TooManyRelays { max, actual } => {
                write!(
                    f,
                    "peer snapshot has too many relays: max={max} actual={actual}"
                )
            }
            Self::InvalidStake => f.write_str("peer snapshot stake values must be finite ratios"),
            Self::InvalidRelay(peer) => write!(
                f,
                "invalid peer snapshot relay {}:{}",
                peer.address, peer.port
            ),
            Self::DuplicateRelay(peer) => write!(
                f,
                "duplicate peer snapshot relay {}:{}",
                peer.address, peer.port
            ),
        }
    }
}

impl std::error::Error for PeerSnapshotError {}

impl std::fmt::Display for TopologyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPeer(value) => write!(f, "invalid peer address {value}"),
            Self::InvalidJson(value) => write!(f, "invalid topology JSON: {value}"),
            Self::InvalidLine { line } => write!(f, "invalid topology line {line}"),
            Self::InvalidValency {
                root,
                valency,
                peer_count,
            } => write!(
                f,
                "invalid {root} root valency: valency={valency} peers={peer_count}"
            ),
            Self::InvalidWarmValency {
                root,
                warm_valency,
                valency,
            } => write!(
                f,
                "invalid {root} root warm valency: warm={warm_valency} valency={valency}"
            ),
        }
    }
}

impl std::error::Error for TopologyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_relays_flatten_great_and_regular_pools() {
        let snapshot = PeerSnapshot {
            network_magic: 2,
            client_version: 16,
            point: PeerSnapshotPoint {
                point_hash: "hash".to_string(),
                point_slot: 10,
            },
            priority_pools: vec![PeerPool {
                accumulated_stake: 0.5,
                relative_stake: 0.5,
                relays: vec![PeerAddress::new("a.example", 3001)],
            }],
            pools: vec![PeerPool {
                accumulated_stake: 0.6,
                relative_stake: 0.1,
                relays: vec![PeerAddress::new("b.example", 3001)],
            }],
        };

        assert!(snapshot.has_relays());
        assert_eq!(snapshot.relay_peers().len(), 2);
        assert_eq!(
            snapshot.validate(&PeerSnapshotRules {
                network_magic: Some(2),
                max_pools: 2,
                max_relays: 2,
            }),
            Ok(())
        );
    }

    #[test]
    fn snapshot_validation_rejects_bad_relay_without_fetching() {
        let snapshot = PeerSnapshot {
            network_magic: 2,
            client_version: 16,
            point: PeerSnapshotPoint {
                point_hash: "hash".to_string(),
                point_slot: 10,
            },
            priority_pools: Vec::new(),
            pools: vec![PeerPool {
                accumulated_stake: 0.2,
                relative_stake: 0.2,
                relays: vec![PeerAddress::new("", 0)],
            }],
        };
        assert_eq!(
            snapshot.validate(&PeerSnapshotRules::default()),
            Err(PeerSnapshotError::InvalidRelay(PeerAddress::new("", 0)))
        );
    }

    #[test]
    fn snapshot_validation_rejects_duplicate_relays_without_fetching() {
        let duplicate = PeerAddress::new("dup.local", 3001);
        let snapshot = PeerSnapshot {
            network_magic: 2,
            client_version: 16,
            point: PeerSnapshotPoint {
                point_hash: "hash".to_string(),
                point_slot: 10,
            },
            priority_pools: vec![PeerPool {
                accumulated_stake: 0.5,
                relative_stake: 0.5,
                relays: vec![duplicate.clone()],
            }],
            pools: vec![PeerPool {
                accumulated_stake: 0.6,
                relative_stake: 0.1,
                relays: vec![duplicate.clone()],
            }],
        };

        assert_eq!(
            snapshot.validate(&PeerSnapshotRules::default()),
            Err(PeerSnapshotError::DuplicateRelay(duplicate))
        );
    }

    #[test]
    fn parse_seed_peers_accepts_local_fixture_text() {
        let peers =
            parse_seed_peers("# local fixture\npeer=alpha.local:3001\nseed=beta.local:3002\n")
                .unwrap();
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0], PeerAddress::new("alpha.local", 3001));
    }

    #[test]
    fn parse_topology_fixture_accepts_roots_seeds_and_metadata() {
        let topology = parse_topology_fixture(
            "# local fixture\n\
             local=alpha.local:3001,beta.local:3002 advertise=true trustable=true valency=2 warm=1\n\
             public=relay.local:3001 valency=1 warm=1\n\
             seed=seed.local:3003\n\
             bootstrap_after=42\n\
             snapshot_file=peers.local\n",
        )
        .unwrap();

        assert_eq!(topology.local_roots.len(), 1);
        assert_eq!(topology.local_roots[0].peers.len(), 2);
        assert!(topology.local_roots[0].advertise);
        assert!(topology.local_roots[0].trustable);
        assert_eq!(topology.local_roots[0].warm_valency, 1);
        assert_eq!(
            topology.public_roots[0].peers[0],
            PeerAddress::new("relay.local", 3001)
        );
        assert_eq!(
            topology.seed_peers,
            vec![PeerAddress::new("seed.local", 3003)]
        );
        assert_eq!(topology.use_bootstrap_after_slot, 42);
        assert_eq!(topology.peer_snapshot_file.as_deref(), Some("peers.local"));
        assert_eq!(
            topology.summary(),
            TopologySummary {
                local_roots: 1,
                public_roots: 1,
                local_peers: 2,
                public_peers: 1,
                bootstrap_peers: 1,
                unique_peers: 4,
                duplicate_peer_entries: 0,
                local_valency: 2,
                public_valency: 1,
                local_warm_valency: 1,
                public_warm_valency: 1,
                advertised_local_roots: 1,
                advertised_public_roots: 0,
                trustable_local_roots: 1,
                empty_local_roots: 0,
                empty_public_roots: 0,
                trustable_local_peers: 2,
                peer_snapshot_configured: true,
                ledger_peers_enabled: true,
                bootstrap_peers_configured: true,
                peer_snapshot_usable_by_topology: true,
                trusted_peer_source_configured: true,
            }
        );
    }

    #[test]
    fn parse_cardano_topology_json_accepts_p2p_fixture_shape() {
        let topology = parse_cardano_topology_json(
            r#"{
              "bootstrapPeers": [
                { "address": "backbone.cardano.iog.io", "port": 3001 },
                { "address": "backbone.mainnet.cardanofoundation.org", "port": 3001 },
                { "address": "backbone.mainnet.emurgornd.com", "port": 3001 }
              ],
              "localRoots": [
                { "accessPoints": [], "advertise": false, "trustable": false, "valency": 1 }
              ],
              "peerSnapshotFile": "mainnet-peer-snapshot.json",
              "publicRoots": [
                { "accessPoints": [], "advertise": false }
              ],
              "useLedgerAfterSlot": 185500763
            }"#,
        )
        .unwrap();

        assert_eq!(topology.seed_peers.len(), 3);
        assert_eq!(
            topology.seed_peers[0],
            PeerAddress::new("backbone.cardano.iog.io", 3001)
        );
        assert_eq!(topology.local_roots.len(), 1);
        assert_eq!(topology.local_roots[0].peers.len(), 0);
        assert_eq!(topology.local_roots[0].valency, 0);
        assert!(!topology.local_roots[0].trustable);
        assert_eq!(topology.public_roots.len(), 1);
        assert_eq!(topology.public_roots[0].valency, 0);
        assert_eq!(topology.use_bootstrap_after_slot, 185500763);
        assert_eq!(
            topology.peer_snapshot_file.as_deref(),
            Some("mainnet-peer-snapshot.json")
        );
        let summary = topology.summary();
        assert_eq!(summary.advertised_local_roots, 0);
        assert_eq!(summary.advertised_public_roots, 0);
        assert_eq!(summary.trustable_local_roots, 0);
        assert_eq!(summary.empty_local_roots, 1);
        assert_eq!(summary.empty_public_roots, 1);
        assert!(summary.ledger_peers_enabled);
        assert!(summary.bootstrap_peers_configured);
        assert!(summary.peer_snapshot_usable_by_topology);
        assert_eq!(summary.trustable_local_peers, 0);
        assert!(summary.trusted_peer_source_configured);
        assert_eq!(topology.validate_valencies(), Ok(()));
    }

    #[test]
    fn parse_cardano_topology_json_accepts_null_optional_fields() {
        let topology = parse_cardano_topology_json(
            r#"{
              "bootstrapPeers": null,
              "localRoots": null,
              "peerSnapshotFile": null,
              "publicRoots": null,
              "Producers": null,
              "useLedgerAfterSlot": null
            }"#,
        )
        .unwrap();

        assert!(topology.seed_peers.is_empty());
        assert!(topology.local_roots.is_empty());
        assert!(topology.public_roots.is_empty());
        assert_eq!(topology.peer_snapshot_file, None);
        assert_eq!(topology.use_bootstrap_after_slot, -1);
        let summary = topology.summary();
        assert!(!summary.bootstrap_peers_configured);
        assert!(!summary.peer_snapshot_configured);
        assert!(!summary.ledger_peers_enabled);
        assert!(!summary.trusted_peer_source_configured);
    }

    #[test]
    fn parse_cardano_topology_json_accepts_access_points() {
        let topology = parse_cardano_topology_json(
            r#"{
              "localRoots": [
                {
                  "accessPoints": [
                    { "address": "alpha.local", "port": 3001 },
                    { "address": "beta.local", "port": 3002 }
                  ],
                  "advertise": true,
                  "trustable": true,
                  "valency": 2
                }
              ],
              "publicRoots": [
                {
                  "accessPoints": [ { "address": "relay.local", "port": 3003 } ],
                  "advertise": false,
                  "valency": 1
                }
              ],
              "useLedgerAfterSlot": -1
            }"#,
        )
        .unwrap();

        assert_eq!(topology.local_roots[0].peers.len(), 2);
        assert_eq!(topology.local_roots[0].valency, 2);
        assert!(topology.local_roots[0].advertise);
        assert!(topology.local_roots[0].trustable);
        assert_eq!(
            topology.public_roots[0].peers[0],
            PeerAddress::new("relay.local", 3003)
        );
        assert_eq!(topology.public_roots[0].valency, 1);
        assert_eq!(
            topology.summary(),
            TopologySummary {
                local_roots: 1,
                public_roots: 1,
                local_peers: 2,
                public_peers: 1,
                bootstrap_peers: 0,
                unique_peers: 3,
                duplicate_peer_entries: 0,
                local_valency: 2,
                public_valency: 1,
                local_warm_valency: 2,
                public_warm_valency: 1,
                advertised_local_roots: 1,
                advertised_public_roots: 0,
                trustable_local_roots: 1,
                empty_local_roots: 0,
                empty_public_roots: 0,
                trustable_local_peers: 2,
                peer_snapshot_configured: false,
                ledger_peers_enabled: false,
                bootstrap_peers_configured: false,
                peer_snapshot_usable_by_topology: false,
                trusted_peer_source_configured: true,
            }
        );
    }

    #[test]
    fn topology_summary_counts_duplicate_peer_entries() {
        let topology = parse_cardano_topology_json(
            r#"{
              "bootstrapPeers": [ { "address": "dup.local", "port": 3001 } ],
              "localRoots": [
                {
                  "accessPoints": [ { "address": "dup.local", "port": 3001 } ],
                  "valency": 1
                }
              ],
              "publicRoots": [
                {
                  "accessPoints": [ { "address": "relay.local", "port": 3002 } ],
                  "valency": 1
                }
              ]
            }"#,
        )
        .unwrap();

        let summary = topology.summary();
        assert_eq!(summary.local_peers, 1);
        assert_eq!(summary.public_peers, 1);
        assert_eq!(summary.bootstrap_peers, 1);
        assert_eq!(summary.unique_peers, 2);
        assert_eq!(summary.duplicate_peer_entries, 1);
    }

    #[test]
    fn parse_cardano_topology_json_accepts_legacy_producers() {
        let topology = parse_cardano_topology_json(
            r#"{
              "Producers": [
                { "addr": "relays-new.cardano-mainnet.iohk.io", "port": 3001, "valency": 1 }
              ]
            }"#,
        )
        .unwrap();

        assert_eq!(topology.public_roots.len(), 1);
        assert_eq!(
            topology.public_roots[0].peers,
            vec![PeerAddress::new("relays-new.cardano-mainnet.iohk.io", 3001)]
        );
        assert_eq!(topology.public_roots[0].valency, 1);
    }

    #[test]
    fn parse_cardano_topology_json_rejects_impossible_nonempty_valency() {
        assert_eq!(
            parse_cardano_topology_json(
                r#"{
                  "publicRoots": [
                    {
                      "accessPoints": [ { "address": "relay.local", "port": 3001 } ],
                      "valency": 2
                    }
                  ]
                }"#,
            ),
            Err(TopologyError::InvalidValency {
                root: "public",
                valency: 2,
                peer_count: 1,
            })
        );
    }

    #[test]
    fn parse_cardano_topology_json_rejects_duplicate_fields() {
        let err = parse_cardano_topology_json(
            r#"{
              "publicRoots": [],
              "publicRoots": []
            }"#,
        )
        .unwrap_err();

        assert!(
            matches!(err, TopologyError::InvalidJson(message) if message.contains("duplicate JSON field publicRoots"))
        );
    }

    #[test]
    fn topology_valency_validation_rejects_impossible_roots() {
        assert_eq!(
            parse_topology_fixture("local=alpha.local:3001 valency=2 warm=1"),
            Err(TopologyError::InvalidValency {
                root: "local",
                valency: 2,
                peer_count: 1,
            })
        );
        assert_eq!(
            parse_topology_fixture("public=relay.local:3001 valency=1 warm=2"),
            Err(TopologyError::InvalidWarmValency {
                root: "public",
                warm_valency: 2,
                valency: 1,
            })
        );
    }

    #[test]
    fn parse_topology_fixture_rejects_oversized_input() {
        let input = "x".repeat(MAX_TOPOLOGY_SIZE + 1);
        assert_eq!(
            parse_topology_fixture(&input),
            Err(TopologyError::InvalidLine { line: 0 })
        );
    }

    #[test]
    fn topology_can_attach_peer_snapshot_fixture_without_io() {
        let topology = parse_topology_fixture("local=alpha.local:3001 valency=1 warm=1")
            .unwrap()
            .with_peer_snapshot_fixture(
                "network_magic=2\n\
                 client_version=16\n\
                 point_hash=abc123\n\
                 point_slot=42\n\
                 pool=beta.local:3002 accumulated=0.5 relative=0.5\n",
            )
            .unwrap();

        let snapshot = topology.peer_snapshot.as_ref().unwrap();
        assert_eq!(snapshot.network_magic, 2);
        assert_eq!(
            snapshot.relay_peers(),
            vec![PeerAddress::new("beta.local", 3002)]
        );
    }

    #[test]
    fn parse_peer_snapshot_fixture_accepts_local_text() {
        let snapshot = parse_peer_snapshot_fixture(
            "# local peer snapshot\n\
             network_magic=2\n\
             client_version=16\n\
             point_hash=abc123\n\
             point_slot=42\n\
             priority_pool=alpha.local:3001 accumulated=0.5 relative=0.5\n\
             pool=beta.local:3002,gamma.local:3003 accumulated=0.75 relative=0.25\n",
        )
        .unwrap();

        assert_eq!(snapshot.network_magic, 2);
        assert_eq!(snapshot.client_version, 16);
        assert_eq!(snapshot.point.point_hash, "abc123");
        assert_eq!(snapshot.point.point_slot, 42);
        assert_eq!(snapshot.priority_pools.len(), 1);
        assert_eq!(snapshot.pools.len(), 1);
        assert_eq!(snapshot.relay_peers().len(), 3);
        assert_eq!(snapshot.validate(&PeerSnapshotRules::default()), Ok(()));
    }

    #[test]
    fn parse_cardano_peer_snapshot_json_accepts_snapshot_shape_without_io() {
        let snapshot = parse_cardano_peer_snapshot_json(
            r#"{
              "NetworkMagic": 1,
              "NodeToClientVersion": 23,
              "Point": {
                "blockPointHash": "abc",
                "blockPointSlot": 42
              },
              "bigLedgerPools": [
                {
                  "relativeStake": 0.5,
                  "accumulatedStake": 0.5,
                  "relays": [
                    {"address": "relay.example.com", "port": 3001}
                  ]
                }
              ]
            }"#,
        )
        .unwrap();

        assert_eq!(snapshot.network_magic, 1);
        assert_eq!(snapshot.client_version, 23);
        assert_eq!(snapshot.point.point_hash, "abc");
        assert_eq!(snapshot.point.point_slot, 42);
        assert_eq!(snapshot.priority_pools.len(), 1);
        assert!(snapshot.pools.is_empty());
        assert_eq!(
            snapshot.relay_peers(),
            vec![PeerAddress::new("relay.example.com", 3001)]
        );
        assert_eq!(
            snapshot.validate(&PeerSnapshotRules {
                network_magic: Some(1),
                max_pools: 1,
                max_relays: 1,
            }),
            Ok(())
        );
    }

    #[test]
    fn parse_cardano_peer_snapshot_json_preserves_ledger_pools_and_missing_ports() {
        let snapshot = parse_cardano_peer_snapshot_json(
            r#"{
              "NetworkMagic": 2,
              "NodeToClientVersion": 23,
              "Point": {
                "blockPointHash": "def",
                "blockPointSlot": 77
              },
              "bigLedgerPools": [
                {
                  "relays": [ {"address": "big.example", "port": 3001} ]
                }
              ],
              "ledgerPools": [
                {
                  "accumulatedStake": 0.75,
                  "relativeStake": 0.25,
                  "relays": [ {"address": "relay-no-port.example"} ]
                }
              ]
            }"#,
        )
        .unwrap();

        assert_eq!(snapshot.priority_pools.len(), 1);
        assert_eq!(snapshot.pools.len(), 1);
        assert_eq!(snapshot.relay_peers().len(), 2);
        assert_eq!(
            snapshot.relay_peers()[1],
            PeerAddress::new("relay-no-port.example", 0)
        );
        assert_eq!(
            snapshot.validate(&PeerSnapshotRules::default()),
            Err(PeerSnapshotError::InvalidRelay(PeerAddress::new(
                "relay-no-port.example",
                0
            )))
        );
    }

    #[test]
    fn parse_peer_snapshot_fixture_rejects_missing_required_fields() {
        assert_eq!(
            parse_peer_snapshot_fixture("network_magic=2\nclient_version=16\npoint_hash=abc"),
            Err(TopologyError::InvalidLine { line: 0 })
        );
        assert_eq!(
            parse_peer_snapshot_fixture(
                "network_magic=2\nclient_version=16\npoint_hash=abc\npoint_slot=1\npool=relay.local:3001 relative=0.1"
            ),
            Err(TopologyError::InvalidLine { line: 5 })
        );
    }

    #[test]
    fn parse_peer_snapshot_fixture_rejects_oversized_input() {
        let input = "x".repeat(MAX_PEER_SNAPSHOT_SIZE + 1);
        assert_eq!(
            parse_peer_snapshot_fixture(&input),
            Err(TopologyError::InvalidLine { line: 0 })
        );
    }
}
