//! Agent gRPC server implementation.

use crate::agent::{Agent, LogSender};
use crate::proto::{self, agent_service_server};
use crate::types::{Context, prost_value_to_json};
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

/// Agent gRPC server.
///
/// Wraps an `Agent` implementation and exposes it as a gRPC service.
pub struct AgentServer<A: Agent> {
    agent: Arc<A>,
}

impl<A: Agent> AgentServer<A> {
    /// Create a new agent server.
    pub fn new(agent: A) -> Self {
        Self {
            agent: Arc::new(agent),
        }
    }

    /// Start serving on the given address.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// AgentServer::new(MyAgent)
    ///     .serve("[::1]:50051")
    ///     .await
    ///     .unwrap();
    /// ```
    pub async fn serve(self, addr: impl ToSocketAddrs) -> Result<(), Box<dyn std::error::Error>> {
        let addr = addr.to_socket_addrs()?.next().ok_or("Invalid address")?;

        let svc =
            agent_service_server::AgentServiceServer::new(AgentServiceImpl { agent: self.agent });

        tonic::transport::Server::builder()
            .add_service(svc)
            .serve(addr)
            .await?;

        Ok(())
    }

    /// Get the gRPC service for custom server configuration.
    pub fn into_service(self) -> agent_service_server::AgentServiceServer<AgentServiceImpl<A>> {
        agent_service_server::AgentServiceServer::new(AgentServiceImpl { agent: self.agent })
    }
}

/// Internal gRPC service implementation.
pub struct AgentServiceImpl<A: Agent> {
    agent: Arc<A>,
}

#[tonic::async_trait]
impl<A: Agent> agent_service_server::AgentService for AgentServiceImpl<A> {
    type RunStream = ReceiverStream<Result<proto::AgentUpdate, Status>>;

    async fn run(
        &self,
        request: Request<proto::RunRequest>,
    ) -> Result<Response<Self::RunStream>, Status> {
        let req = request.into_inner();

        let nid = req.nid;
        let ctx: Context = req.ctx.unwrap_or_default().into();
        let inputs = req
            .inputs
            .map(prost_value_to_json)
            .unwrap_or(serde_json::Value::Null);

        let (tx, rx) = mpsc::channel(32);
        let (log_tx, mut log_rx) = mpsc::channel::<String>(1024);

        let agent = self.agent.clone();
        let log_sender = LogSender::new(log_tx);

        // Spawn task to forward logs
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Some(log) = log_rx.recv().await {
                let update = proto::AgentUpdate {
                    relay_message: Some(proto::agent_update::RelayMessage::Log(log)),
                };
                if tx_clone.send(Ok(update)).await.is_err() {
                    break;
                }
            }
        });

        // Spawn task to run agent
        tokio::spawn(async move {
            let output = agent.run(nid, ctx, inputs, log_sender).await;
            let update = proto::AgentUpdate {
                relay_message: Some(proto::agent_update::RelayMessage::Output(output.into())),
            };
            let _ = tx.send(Ok(update)).await;
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn shutdown(
        &self,
        _request: Request<proto::Empty>,
    ) -> Result<Response<proto::Empty>, Status> {
        self.agent.shutdown().await;
        Ok(Response::new(proto::Empty {}))
    }
}
