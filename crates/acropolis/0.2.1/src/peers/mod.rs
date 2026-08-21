use crate::events::{Event, EventPayload};
use crate::topology::{PeerAddress, PeerSnapshotRules, TopologyConfig};

pub const PEER_CHURN_EVENT: &str = "peers.churn";
pub const PEER_CONNECTION_BLOCKED_EVENT: &str = "peers.connection.blocked";
pub const PEER_CONNECTION_PLAN_EVENT: &str = "peers.connection_plan";
pub const PEER_DISCOVER_EVENT: &str = "peers.discover";
pub const PEER_DISCOVERY_PLAN_EVENT: &str = "peers.discovery_plan";
pub const PEER_LIFECYCLE_PLAN_EVENT: &str = "peers.lifecycle_plan";
pub const PEER_PROMOTE_EVENT: &str = "peers.promote";
pub const PEER_PRUNE_EVENT: &str = "peers.prune";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerTargets {
    pub known: i32,
    pub established: i32,
    pub active: i32,
}

impl Default for PeerTargets {
    fn default() -> Self {
        Self {
            known: 100,
            established: 20,
            active: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerPlan {
    pub targets: PeerTargets,
    pub peers: Vec<PeerAddress>,
    pub bootstrap_peers_enabled: bool,
}

impl PeerPlan {
    pub fn new(targets: PeerTargets) -> Self {
        Self {
            targets,
            peers: Vec::new(),
            bootstrap_peers_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerDiscoverySource {
    LocalRoot,
    PublicRoot,
    Seed,
    Snapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerDiscoveryEntry {
    pub peer: PeerAddress,
    pub source: PeerDiscoverySource,
    pub advertise: bool,
    pub trustable: bool,
}

impl PeerDiscoveryEntry {
    pub fn score_bias(&self) -> i64 {
        let source_bias = match self.source {
            PeerDiscoverySource::LocalRoot => 30,
            PeerDiscoverySource::PublicRoot => 20,
            PeerDiscoverySource::Seed => 10,
            PeerDiscoverySource::Snapshot => 5,
        };
        source_bias + if self.trustable { 20 } else { 0 } + if self.advertise { 5 } else { 0 }
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            PEER_DISCOVER_EVENT,
            EventPayload::Text(format!(
                "peer={}:{} source={:?} advertise={} trustable={}",
                self.peer.address, self.peer.port, self.source, self.advertise, self.trustable
            )),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerDiscoveryPlan {
    pub entries: Vec<PeerDiscoveryEntry>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PeerDiscoveryCounts {
    pub local_roots: usize,
    pub public_roots: usize,
    pub seed_peers: usize,
    pub snapshot_relays: usize,
}

impl PeerDiscoveryCounts {
    pub fn total(self) -> usize {
        self.local_roots + self.public_roots + self.seed_peers + self.snapshot_relays
    }
}

impl PeerDiscoveryPlan {
    pub fn from_topology(
        topology: &TopologyConfig,
        snapshot_rules: &PeerSnapshotRules,
    ) -> Result<Self, crate::topology::PeerSnapshotError> {
        if let Some(snapshot) = &topology.peer_snapshot {
            snapshot.validate(snapshot_rules)?;
        }
        let mut entries = Vec::new();
        for root in &topology.local_roots {
            for peer in &root.peers {
                push_unique(
                    &mut entries,
                    PeerDiscoveryEntry {
                        peer: peer.clone(),
                        source: PeerDiscoverySource::LocalRoot,
                        advertise: root.advertise,
                        trustable: root.trustable,
                    },
                );
            }
        }
        for root in &topology.public_roots {
            for peer in &root.peers {
                push_unique(
                    &mut entries,
                    PeerDiscoveryEntry {
                        peer: peer.clone(),
                        source: PeerDiscoverySource::PublicRoot,
                        advertise: root.advertise,
                        trustable: false,
                    },
                );
            }
        }
        for peer in &topology.seed_peers {
            push_unique(
                &mut entries,
                PeerDiscoveryEntry {
                    peer: peer.clone(),
                    source: PeerDiscoverySource::Seed,
                    advertise: false,
                    trustable: false,
                },
            );
        }
        if let Some(snapshot) = &topology.peer_snapshot {
            for peer in snapshot.relay_peers() {
                push_unique(
                    &mut entries,
                    PeerDiscoveryEntry {
                        peer,
                        source: PeerDiscoverySource::Snapshot,
                        advertise: false,
                        trustable: false,
                    },
                );
            }
        }
        Ok(Self { entries })
    }

    pub fn events(&self) -> Vec<Event> {
        self.entries
            .iter()
            .map(PeerDiscoveryEntry::to_event)
            .collect()
    }

    pub fn summary_line(&self) -> String {
        let counts = self.source_counts();
        format!(
            "peer_discovery entries={} local_roots={} public_roots={} seed_peers={} snapshot_relays={}",
            counts.total(),
            counts.local_roots,
            counts.public_roots,
            counts.seed_peers,
            counts.snapshot_relays,
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            PEER_DISCOVERY_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        let mut events = vec![self.to_event()];
        events.extend(self.events());
        events
    }

    pub fn source_counts(&self) -> PeerDiscoveryCounts {
        let mut counts = PeerDiscoveryCounts::default();
        for entry in &self.entries {
            match entry.source {
                PeerDiscoverySource::LocalRoot => counts.local_roots += 1,
                PeerDiscoverySource::PublicRoot => counts.public_roots += 1,
                PeerDiscoverySource::Seed => counts.seed_peers += 1,
                PeerDiscoverySource::Snapshot => counts.snapshot_relays += 1,
            }
        }
        counts
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerLifecyclePlan {
    pub added: Vec<PeerAddress>,
    pub promoted: Vec<(PeerAddress, PeerState)>,
    pub pruned: Vec<PeerAddress>,
}

impl PeerLifecyclePlan {
    pub fn summary_line(&self) -> String {
        format!(
            "peer_lifecycle added={} promoted={} pruned={}",
            self.added.len(),
            self.promoted.len(),
            self.pruned.len()
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            PEER_LIFECYCLE_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        let mut events = vec![self.to_event()];
        events.extend(self.events());
        events
    }

    pub fn events(&self) -> Vec<Event> {
        let mut events = Vec::new();
        for peer in &self.added {
            events.push(Event::new(
                PEER_CHURN_EVENT,
                EventPayload::Text(format!("added={}:{}", peer.address, peer.port)),
            ));
        }
        for (peer, state) in &self.promoted {
            events.push(Event::new(
                PEER_PROMOTE_EVENT,
                EventPayload::Text(format!(
                    "peer={}:{} state={state:?}",
                    peer.address, peer.port
                )),
            ));
        }
        for peer in &self.pruned {
            events.push(Event::new(
                PEER_PRUNE_EVENT,
                EventPayload::Text(format!("peer={}:{}", peer.address, peer.port)),
            ));
        }
        events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerConnectionPlan {
    pub warm: Vec<PeerAddress>,
    pub hot: Vec<PeerAddress>,
    pub open_paths: bool,
    pub blocked_reason: Option<String>,
}

impl PeerConnectionPlan {
    pub fn summary_line(&self) -> String {
        format!(
            "peer_connection warm={} hot={} open_paths={} blocked={}",
            self.warm.len(),
            self.hot.len(),
            self.open_paths,
            self.blocked_reason.is_some()
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            PEER_CONNECTION_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        let mut events = vec![self.to_event()];
        events.extend(self.events());
        events
    }

    pub fn events(&self) -> Vec<Event> {
        let Some(reason) = &self.blocked_reason else {
            return Vec::new();
        };
        vec![Event::new(
            PEER_CONNECTION_BLOCKED_EVENT,
            EventPayload::Text(format!(
                "warm={} hot={} open_paths={} reason={reason}",
                self.warm.len(),
                self.hot.len(),
                self.open_paths
            )),
        )]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    Known,
    Warm,
    Hot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerRecord {
    pub peer: PeerAddress,
    pub state: PeerState,
    pub score: i64,
}

impl PeerRecord {
    pub fn new(peer: PeerAddress) -> Self {
        Self {
            peer,
            state: PeerState::Known,
            score: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeerSet {
    targets: PeerTargets,
    peers: Vec<PeerRecord>,
}

impl PeerSet {
    pub fn new(targets: PeerTargets) -> Self {
        Self {
            targets,
            peers: Vec::new(),
        }
    }

    pub fn add_peer(&mut self, peer: PeerAddress) -> bool {
        if self.peers.iter().any(|record| record.peer == peer) {
            return false;
        }
        self.peers.push(PeerRecord::new(peer));
        true
    }

    pub fn apply_discovery_plan(&mut self, plan: &PeerDiscoveryPlan) -> PeerLifecyclePlan {
        let mut added = Vec::new();
        for entry in &plan.entries {
            if self.add_peer(entry.peer.clone()) {
                added.push(entry.peer.clone());
            }
            self.bump_score(&entry.peer, entry.score_bias());
        }
        let promoted = self.promote_to_targets();
        let pruned = self.prune_to_targets();
        PeerLifecyclePlan {
            added,
            promoted,
            pruned,
        }
    }

    pub fn connection_plan(&self, allow_paths: bool) -> PeerConnectionPlan {
        let mut warm = self
            .peers
            .iter()
            .filter(|record| matches!(record.state, PeerState::Warm | PeerState::Hot))
            .map(|record| record.peer.clone())
            .collect::<Vec<_>>();
        let mut hot = self
            .peers
            .iter()
            .filter(|record| record.state == PeerState::Hot)
            .map(|record| record.peer.clone())
            .collect::<Vec<_>>();
        warm.truncate(self.targets.established.max(0) as usize);
        hot.truncate(self.targets.active.max(0) as usize);
        let blocked_reason = if allow_paths || (warm.is_empty() && hot.is_empty()) {
            None
        } else {
            Some("peer paths are blocked by safety config".to_string())
        };
        PeerConnectionPlan {
            warm,
            hot,
            open_paths: allow_paths,
            blocked_reason,
        }
    }

    pub fn promote(&mut self, peer: &PeerAddress, state: PeerState) -> bool {
        let Some(record) = self.peers.iter_mut().find(|record| &record.peer == peer) else {
            return false;
        };
        record.state = state;
        true
    }

    pub fn bump_score(&mut self, peer: &PeerAddress, delta: i64) -> bool {
        let Some(record) = self.peers.iter_mut().find(|record| &record.peer == peer) else {
            return false;
        };
        record.score = record.score.saturating_add(delta);
        true
    }

    pub fn counts(&self) -> PeerCounts {
        PeerCounts {
            known: self.peers.len(),
            warm: self
                .peers
                .iter()
                .filter(|record| matches!(record.state, PeerState::Warm | PeerState::Hot))
                .count(),
            hot: self
                .peers
                .iter()
                .filter(|record| record.state == PeerState::Hot)
                .count(),
        }
    }

    pub fn prune_to_targets(&mut self) -> Vec<PeerAddress> {
        let target = self.targets.known.max(0) as usize;
        if self.peers.len() <= target {
            return Vec::new();
        }
        self.peers.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.peer.address.cmp(&right.peer.address))
                .then_with(|| left.peer.port.cmp(&right.peer.port))
        });
        self.peers
            .split_off(target)
            .into_iter()
            .map(|record| record.peer)
            .collect()
    }

    pub fn records(&self) -> &[PeerRecord] {
        &self.peers
    }

    fn promote_to_targets(&mut self) -> Vec<(PeerAddress, PeerState)> {
        self.peers.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.peer.address.cmp(&right.peer.address))
                .then_with(|| left.peer.port.cmp(&right.peer.port))
        });
        let hot_target = self.targets.active.max(0) as usize;
        let warm_target = self.targets.established.max(0) as usize;
        let mut promoted = Vec::new();
        for (index, record) in self.peers.iter_mut().enumerate() {
            let next_state = if index < hot_target {
                PeerState::Hot
            } else if index < warm_target {
                PeerState::Warm
            } else {
                PeerState::Known
            };
            if record.state != next_state {
                record.state = next_state;
                promoted.push((record.peer.clone(), next_state));
            }
        }
        promoted
    }
}

fn push_unique(entries: &mut Vec<PeerDiscoveryEntry>, entry: PeerDiscoveryEntry) {
    if !entries.iter().any(|existing| existing.peer == entry.peer) {
        entries.push(entry);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerCounts {
    pub known: usize,
    pub warm: usize,
    pub hot: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::{
        parse_topology_fixture, LocalRoot, PeerPool, PeerSnapshot, PeerSnapshotPoint, PublicRoot,
    };

    #[test]
    fn peer_set_promotes_and_counts_peers() {
        let peer = PeerAddress::new("peer.local", 3001);
        let mut peers = PeerSet::new(PeerTargets::default());
        assert!(peers.add_peer(peer.clone()));
        assert!(!peers.add_peer(peer.clone()));
        assert!(peers.promote(&peer, PeerState::Hot));
        assert_eq!(
            peers.counts(),
            PeerCounts {
                known: 1,
                warm: 1,
                hot: 1
            }
        );
    }

    #[test]
    fn peer_set_prunes_lowest_scored_peers() {
        let keep = PeerAddress::new("keep.local", 1);
        let drop = PeerAddress::new("drop.local", 1);
        let mut peers = PeerSet::new(PeerTargets {
            known: 1,
            established: 0,
            active: 0,
        });
        peers.add_peer(drop.clone());
        peers.add_peer(keep.clone());
        peers.bump_score(&keep, 10);
        assert_eq!(peers.prune_to_targets(), vec![drop]);
        assert_eq!(peers.records()[0].peer, keep);
    }

    #[test]
    fn discovery_plan_deduplicates_topology_sources() {
        let duplicate = PeerAddress::new("dup.local", 3001);
        let topology = TopologyConfig {
            local_roots: vec![LocalRoot {
                peers: vec![duplicate.clone()],
                advertise: true,
                trustable: true,
                valency: 1,
                warm_valency: 1,
            }],
            public_roots: vec![PublicRoot {
                peers: vec![duplicate.clone(), PeerAddress::new("public.local", 3001)],
                advertise: false,
                valency: 1,
                warm_valency: 1,
            }],
            seed_peers: vec![PeerAddress::new("seed.local", 3001)],
            use_bootstrap_after_slot: -1,
            peer_snapshot_file: None,
            peer_snapshot: Some(PeerSnapshot {
                network_magic: 2,
                client_version: 1,
                point: PeerSnapshotPoint {
                    point_hash: "hash".to_string(),
                    point_slot: 42,
                },
                priority_pools: vec![PeerPool {
                    accumulated_stake: 0.5,
                    relative_stake: 0.5,
                    relays: vec![PeerAddress::new("snapshot.local", 3001)],
                }],
                pools: Vec::new(),
            }),
        };
        let plan = PeerDiscoveryPlan::from_topology(
            &topology,
            &PeerSnapshotRules {
                network_magic: Some(2),
                max_pools: 1,
                max_relays: 1,
            },
        )
        .unwrap();
        assert_eq!(plan.entries.len(), 4);
        assert_eq!(plan.entries[0].source, PeerDiscoverySource::LocalRoot);
        assert_eq!(plan.events().len(), 4);
        assert_eq!(plan.event_batch().len(), 5);
        assert_eq!(
            plan.event_batch()[0].name.as_str(),
            PEER_DISCOVERY_PLAN_EVENT
        );
        let counts = plan.source_counts();
        assert_eq!(counts.local_roots, 1);
        assert_eq!(counts.public_roots, 1);
        assert_eq!(counts.seed_peers, 1);
        assert_eq!(counts.snapshot_relays, 1);
        assert_eq!(counts.total(), 4);
        assert_eq!(
            plan.summary_line(),
            "peer_discovery entries=4 local_roots=1 public_roots=1 seed_peers=1 snapshot_relays=1"
        );
    }

    #[test]
    fn discovery_plan_uses_parsed_snapshot_fixture_without_dialing() {
        let topology = parse_topology_fixture("local=trusted.local:3001 valency=1 warm=1")
            .unwrap()
            .with_peer_snapshot_fixture(
                "network_magic=2\n\
                 client_version=16\n\
                 point_hash=abc123\n\
                 point_slot=42\n\
                 pool=snapshot.local:3002 accumulated=0.5 relative=0.5\n",
            )
            .unwrap();

        let plan = PeerDiscoveryPlan::from_topology(
            &topology,
            &PeerSnapshotRules {
                network_magic: Some(2),
                max_pools: 1,
                max_relays: 1,
            },
        )
        .unwrap();

        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.entries[1].source, PeerDiscoverySource::Snapshot);
        assert_eq!(
            plan.entries[1].peer,
            PeerAddress::new("snapshot.local", 3002)
        );
    }

    #[test]
    fn discovery_plan_rejects_duplicate_snapshot_fixture_relays() {
        let topology = parse_topology_fixture("local=trusted.local:3001 valency=1 warm=1")
            .unwrap()
            .with_peer_snapshot_fixture(
                "network_magic=2\n\
                 client_version=16\n\
                 point_hash=abc123\n\
                 point_slot=42\n\
                 priority_pool=dup.local:3002 accumulated=0.5 relative=0.5\n\
                 pool=dup.local:3002 accumulated=0.6 relative=0.1\n",
            )
            .unwrap();

        assert_eq!(
            PeerDiscoveryPlan::from_topology(&topology, &PeerSnapshotRules::default()),
            Err(crate::topology::PeerSnapshotError::DuplicateRelay(
                PeerAddress::new("dup.local", 3002)
            ))
        );
    }

    #[test]
    fn peer_set_applies_discovery_and_plans_connections_without_dialing() {
        let topology = TopologyConfig {
            local_roots: vec![LocalRoot {
                peers: vec![PeerAddress::new("trusted.local", 3001)],
                advertise: true,
                trustable: true,
                valency: 1,
                warm_valency: 1,
            }],
            public_roots: vec![PublicRoot {
                peers: vec![PeerAddress::new("public.local", 3001)],
                advertise: false,
                valency: 1,
                warm_valency: 1,
            }],
            seed_peers: vec![PeerAddress::new("seed.local", 3001)],
            use_bootstrap_after_slot: -1,
            peer_snapshot_file: None,
            peer_snapshot: None,
        };
        let discovery =
            PeerDiscoveryPlan::from_topology(&topology, &PeerSnapshotRules::default()).unwrap();
        let mut peers = PeerSet::new(PeerTargets {
            known: 3,
            established: 2,
            active: 1,
        });
        let lifecycle = peers.apply_discovery_plan(&discovery);
        assert_eq!(lifecycle.added.len(), 3);
        assert_eq!(peers.counts().hot, 1);
        assert_eq!(peers.counts().warm, 2);
        assert_eq!(
            lifecycle.events().len(),
            lifecycle.added.len() + lifecycle.promoted.len()
        );
        assert_eq!(
            lifecycle.event_batch().len(),
            1 + lifecycle.added.len() + lifecycle.promoted.len()
        );
        assert_eq!(
            lifecycle.event_batch()[0].name.as_str(),
            PEER_LIFECYCLE_PLAN_EVENT
        );
        assert_eq!(
            lifecycle.summary_line(),
            "peer_lifecycle added=3 promoted=2 pruned=0"
        );

        let connection_plan = peers.connection_plan(false);
        assert!(!connection_plan.open_paths);
        assert_eq!(connection_plan.hot.len(), 1);
        assert_eq!(connection_plan.warm.len(), 2);
        assert!(connection_plan.blocked_reason.is_some());
        let events = connection_plan.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), PEER_CONNECTION_BLOCKED_EVENT);
        assert_eq!(
            events[0].payload,
            EventPayload::Text(
                "warm=2 hot=1 open_paths=false reason=peer paths are blocked by safety config"
                    .to_string()
            )
        );
        let event_batch = connection_plan.event_batch();
        assert_eq!(event_batch.len(), 2);
        assert_eq!(event_batch[0].name.as_str(), PEER_CONNECTION_PLAN_EVENT);
        assert_eq!(
            connection_plan.summary_line(),
            "peer_connection warm=2 hot=1 open_paths=false blocked=true"
        );
    }
}
