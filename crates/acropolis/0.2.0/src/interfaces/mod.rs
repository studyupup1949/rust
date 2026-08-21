use crate::config::{DataMode, InterfaceConfig};
use crate::events::{Event, EventPayload};
use std::fmt;

pub const INTERFACE_BLOCKED_EVENT: &str = "interfaces.blocked";
pub const INTERFACE_PLAN_EVENT: &str = "interfaces.plan";
pub const INTERFACE_REQUEST_PLAN_EVENT: &str = "interfaces.request_plan";
pub const INTERFACE_ROUTE_PLAN_EVENT: &str = "interfaces.route_plan";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfacePlan {
    pub transactions_enabled: bool,
    pub state_enabled: bool,
    pub batches_enabled: bool,
    pub archive_enabled: bool,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceRequest {
    pub name: String,
    pub requested: bool,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceRequestPlan {
    pub requests: Vec<InterfaceRequest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceKind {
    Transactions,
    State,
    Batches,
    Archive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceRoute {
    pub kind: InterfaceKind,
    pub name: String,
    pub path: String,
    pub port: u16,
    pub state: InterfaceState,
    pub max_body_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceRoutePlan {
    pub routes: Vec<InterfaceRoute>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InterfaceRouteCounts {
    pub closed: usize,
    pub planned: usize,
    pub open: usize,
}

impl InterfaceRouteCounts {
    pub fn total(self) -> usize {
        self.closed + self.planned + self.open
    }
}

impl InterfaceRequestPlan {
    pub fn requested_count(&self) -> usize {
        self.requests
            .iter()
            .filter(|request| request.requested)
            .count()
    }

    pub fn summary_line(&self) -> String {
        let requested = self.requested_count();
        format!(
            "interface_requests total={} requested={} unrequested={}",
            self.requests.len(),
            requested,
            self.requests.len().saturating_sub(requested)
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            INTERFACE_REQUEST_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl InterfacePlan {
    pub fn from_config(interfaces: &InterfaceConfig, data_mode: DataMode) -> Self {
        let any_public_interface = interfaces.transaction_port != 0
            || interfaces.state_port != 0
            || interfaces.batch_port != 0;
        let blocked_reason = if any_public_interface && data_mode != DataMode::Full {
            Some("public interfaces require ACROPOLIS_DATA_MODE=full".to_string())
        } else {
            None
        };
        Self {
            transactions_enabled: interfaces.transaction_port != 0 && blocked_reason.is_none(),
            state_enabled: interfaces.state_port != 0 && blocked_reason.is_none(),
            batches_enabled: interfaces.batch_port != 0 && blocked_reason.is_none(),
            archive_enabled: interfaces.archive_port != 0,
            blocked_reason,
        }
    }

    pub fn request_plan(config: &InterfaceConfig) -> InterfaceRequestPlan {
        InterfaceRequestPlan {
            requests: vec![
                InterfaceRequest {
                    name: "transactions".to_string(),
                    requested: config.transaction_port != 0,
                    port: config.transaction_port,
                },
                InterfaceRequest {
                    name: "state".to_string(),
                    requested: config.state_port != 0,
                    port: config.state_port,
                },
                InterfaceRequest {
                    name: "batches".to_string(),
                    requested: config.batch_port != 0,
                    port: config.batch_port,
                },
                InterfaceRequest {
                    name: "archive".to_string(),
                    requested: config.archive_port != 0,
                    port: config.archive_port,
                },
            ],
        }
    }

    pub fn route_plan(config: &InterfaceConfig, data_mode: DataMode) -> InterfaceRoutePlan {
        let plan = Self::from_config(config, data_mode);
        InterfaceRoutePlan {
            routes: vec![
                InterfaceRoute::new(
                    InterfaceKind::Transactions,
                    "transactions",
                    "/transactions",
                    config.transaction_port,
                    plan.transactions_enabled,
                ),
                InterfaceRoute::new(
                    InterfaceKind::State,
                    "state",
                    "/state",
                    config.state_port,
                    plan.state_enabled,
                ),
                InterfaceRoute::new(
                    InterfaceKind::Batches,
                    "batches",
                    "/batches",
                    config.batch_port,
                    plan.batches_enabled,
                ),
                InterfaceRoute::new(
                    InterfaceKind::Archive,
                    "archive",
                    "/archive",
                    config.archive_port,
                    plan.archive_enabled,
                ),
            ],
        }
    }

    pub fn events(&self) -> Vec<Event> {
        let Some(reason) = &self.blocked_reason else {
            return Vec::new();
        };
        vec![Event::new(
            INTERFACE_BLOCKED_EVENT,
            EventPayload::Text(format!(
                "transactions={} state={} batches={} archive={} reason={reason}",
                self.transactions_enabled,
                self.state_enabled,
                self.batches_enabled,
                self.archive_enabled
            )),
        )]
    }

    pub fn summary_line(&self) -> String {
        format!(
            "interface_plan transactions={} state={} batches={} archive={} blocked={}",
            self.transactions_enabled,
            self.state_enabled,
            self.batches_enabled,
            self.archive_enabled,
            self.blocked_reason.is_some()
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            INTERFACE_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        let mut events = vec![self.to_event()];
        events.extend(self.events());
        events
    }
}

impl InterfaceRoute {
    pub fn new(
        kind: InterfaceKind,
        name: impl Into<String>,
        path: impl Into<String>,
        port: u16,
        enabled: bool,
    ) -> Self {
        Self {
            kind,
            name: name.into(),
            path: path.into(),
            port,
            state: if enabled {
                InterfaceState::Planned
            } else {
                InterfaceState::Closed
            },
            max_body_bytes: match kind {
                InterfaceKind::Transactions => 16 * 1024,
                InterfaceKind::State => 1024 * 1024,
                InterfaceKind::Batches => 64 * 1024,
                InterfaceKind::Archive => 4 * 1024 * 1024,
            },
        }
    }
}

impl InterfaceRoutePlan {
    pub fn summary_line(&self) -> String {
        let counts = self.state_counts();
        format!(
            "interface_routes total={} closed={} planned={} open={}",
            counts.total(),
            counts.closed,
            counts.planned,
            counts.open
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            INTERFACE_ROUTE_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }

    pub fn planned(&self) -> Vec<&InterfaceRoute> {
        self.routes
            .iter()
            .filter(|route| route.state == InterfaceState::Planned)
            .collect()
    }

    pub fn state_counts(&self) -> InterfaceRouteCounts {
        let mut counts = InterfaceRouteCounts::default();
        for route in &self.routes {
            match route.state {
                InterfaceState::Closed => counts.closed += 1,
                InterfaceState::Planned => counts.planned += 1,
                InterfaceState::Open => counts.open += 1,
            }
        }
        counts
    }

    pub fn handle_local(
        &self,
        kind: InterfaceKind,
        body: &[u8],
    ) -> Result<InterfaceResponse, InterfaceError> {
        let route = self
            .routes
            .iter()
            .find(|route| route.kind == kind)
            .ok_or(InterfaceError::UnknownRoute(kind))?;
        route.handle_local(body)
    }
}

impl InterfaceRoute {
    pub fn handle_local(&self, body: &[u8]) -> Result<InterfaceResponse, InterfaceError> {
        if self.state == InterfaceState::Closed {
            return Err(InterfaceError::RouteClosed(self.name.clone()));
        }
        if body.len() > self.max_body_bytes {
            return Err(InterfaceError::BodyTooLarge {
                name: self.name.clone(),
                max: self.max_body_bytes,
                actual: body.len(),
            });
        }
        if matches!(
            self.kind,
            InterfaceKind::Transactions | InterfaceKind::Batches
        ) && body.is_empty()
        {
            return Err(InterfaceError::EmptyBody(self.name.clone()));
        }
        let status_code = match self.kind {
            InterfaceKind::Transactions | InterfaceKind::Batches => 202,
            InterfaceKind::State | InterfaceKind::Archive => 200,
        };
        Ok(InterfaceResponse {
            kind: self.kind,
            status_code,
            body: format!(
                "{} handled locally at {} bytes={}",
                self.name,
                self.path,
                body.len()
            ),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceResponse {
    pub kind: InterfaceKind,
    pub status_code: u16,
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceState {
    Closed,
    Planned,
    Open,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceLatch {
    pub name: String,
    pub state: InterfaceState,
}

impl InterfaceLatch {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: InterfaceState::Closed,
        }
    }

    pub fn plan(&mut self) {
        if self.state == InterfaceState::Closed {
            self.state = InterfaceState::Planned;
        }
    }

    pub fn open(&mut self, allow_paths: bool) -> Result<(), InterfaceError> {
        if !allow_paths {
            return Err(InterfaceError::SafetyClosed(self.name.clone()));
        }
        self.state = InterfaceState::Open;
        Ok(())
    }

    pub fn close(&mut self) {
        self.state = InterfaceState::Closed;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceError {
    SafetyClosed(String),
    RouteClosed(String),
    UnknownRoute(InterfaceKind),
    EmptyBody(String),
    BodyTooLarge {
        name: String,
        max: usize,
        actual: usize,
    },
}

impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SafetyClosed(name) => write!(f, "interface {name} blocked by safety config"),
            Self::RouteClosed(name) => write!(f, "interface route {name} is closed"),
            Self::UnknownRoute(kind) => write!(f, "unknown interface route {kind:?}"),
            Self::EmptyBody(name) => write!(f, "interface route {name} requires a body"),
            Self::BodyTooLarge { name, max, actual } => write!(
                f,
                "interface route {name} body too large: max={max} actual={actual}"
            ),
        }
    }
}

impl std::error::Error for InterfaceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interface_latch_stays_closed_without_safety_permission() {
        let mut latch = InterfaceLatch::new("transactions");
        latch.plan();
        assert_eq!(latch.state, InterfaceState::Planned);
        assert_eq!(
            latch.open(false),
            Err(InterfaceError::SafetyClosed("transactions".to_string()))
        );
        assert_eq!(latch.state, InterfaceState::Planned);
        latch.close();
        assert_eq!(latch.state, InterfaceState::Closed);
    }

    #[test]
    fn request_plan_lists_closed_interface_requests() {
        let config = InterfaceConfig {
            transaction_port: 10,
            state_port: 0,
            batch_port: 0,
            archive_port: 20,
            archive_base_url: None,
        };
        let plan = InterfacePlan::request_plan(&config);
        assert_eq!(plan.requested_count(), 2);
        assert_eq!(plan.event_batch().len(), 1);
        assert_eq!(
            plan.event_batch()[0].name.as_str(),
            INTERFACE_REQUEST_PLAN_EVENT
        );
        assert_eq!(
            plan.summary_line(),
            "interface_requests total=4 requested=2 unrequested=2"
        );
    }

    #[test]
    fn route_plan_keeps_public_interfaces_closed_without_full_data() {
        let config = InterfaceConfig {
            transaction_port: 10,
            state_port: 11,
            batch_port: 12,
            archive_port: 13,
            archive_base_url: None,
        };
        let plan = InterfacePlan::route_plan(&config, DataMode::Core);
        assert_eq!(plan.planned().len(), 1);
        assert_eq!(plan.planned()[0].kind, InterfaceKind::Archive);
        assert_eq!(plan.routes[0].state, InterfaceState::Closed);
        assert_eq!(
            plan.state_counts(),
            InterfaceRouteCounts {
                closed: 3,
                planned: 1,
                open: 0,
            }
        );
        assert_eq!(plan.state_counts().total(), 4);
        assert_eq!(plan.event_batch().len(), 1);
        assert_eq!(
            plan.event_batch()[0].name.as_str(),
            INTERFACE_ROUTE_PLAN_EVENT
        );
        assert_eq!(
            plan.summary_line(),
            "interface_routes total=4 closed=3 planned=1 open=0"
        );
    }

    #[test]
    fn interface_plan_emits_blocked_event_without_opening_paths() {
        let config = InterfaceConfig {
            transaction_port: 10,
            state_port: 11,
            batch_port: 12,
            archive_port: 13,
            archive_base_url: None,
        };
        let plan = InterfacePlan::from_config(&config, DataMode::Core);
        let events = plan.events();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), INTERFACE_BLOCKED_EVENT);
        assert_eq!(
            events[0].payload,
            EventPayload::Text(
                "transactions=false state=false batches=false archive=true reason=public interfaces require ACROPOLIS_DATA_MODE=full"
                    .to_string()
            )
        );
        let event_batch = plan.event_batch();
        assert_eq!(event_batch.len(), 2);
        assert_eq!(event_batch[0].name.as_str(), INTERFACE_PLAN_EVENT);
        assert_eq!(
            plan.summary_line(),
            "interface_plan transactions=false state=false batches=false archive=true blocked=true"
        );
    }

    #[test]
    fn local_interface_handler_accepts_planned_transaction_without_opening_paths() {
        let config = InterfaceConfig {
            transaction_port: 10,
            state_port: 11,
            batch_port: 0,
            archive_port: 0,
            archive_base_url: None,
        };
        let plan = InterfacePlan::route_plan(&config, DataMode::Full);

        let response = plan
            .handle_local(InterfaceKind::Transactions, b"transaction-bytes")
            .unwrap();
        assert_eq!(response.kind, InterfaceKind::Transactions);
        assert_eq!(response.status_code, 202);
        assert!(response.body.contains("handled locally"));
    }

    #[test]
    fn local_interface_handler_rejects_closed_route() {
        let config = InterfaceConfig {
            transaction_port: 10,
            state_port: 0,
            batch_port: 0,
            archive_port: 0,
            archive_base_url: None,
        };
        let plan = InterfacePlan::route_plan(&config, DataMode::Core);

        assert_eq!(
            plan.handle_local(InterfaceKind::Transactions, b"transaction-bytes"),
            Err(InterfaceError::RouteClosed("transactions".to_string()))
        );
    }

    #[test]
    fn local_interface_handler_rejects_oversized_body() {
        let route =
            InterfaceRoute::new(InterfaceKind::Transactions, "transactions", "/tx", 10, true);
        let body = vec![0; route.max_body_bytes + 1];

        assert_eq!(
            route.handle_local(&body),
            Err(InterfaceError::BodyTooLarge {
                name: "transactions".to_string(),
                max: 16 * 1024,
                actual: 16 * 1024 + 1,
            })
        );
    }
}
