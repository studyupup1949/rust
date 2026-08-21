pub mod client;
pub mod errors;
pub mod events;
pub mod jobs;
pub mod signals;
pub mod subscription;

pub mod proto {
    tonic::include_proto!("ada.v1");
}

pub use client::{
    AdaClient, ClientConfig, Principal, PrincipalData, PrincipalDocuments, PrincipalSummary,
};
pub use errors::AdaError;
pub use events::PrincipalEvents;
pub use jobs::PrincipalJobs;
pub use signals::PrincipalSignals;
pub use subscription::{
    ConnectedInfo, ConnectingInfo, CursorInfo, DisconnectedInfo, Lifecycle, ListenerErrorInfo,
    ProtocolErrorInfo, ReconnectInfo, StreamConfig, StreamKind, SubscriptionOptions, TerminalInfo,
    Unsubscribe,
};
