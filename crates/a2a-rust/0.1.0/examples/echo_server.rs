use a2a_rust::A2AError;
use a2a_rust::server::{A2AHandler, router};
use a2a_rust::types::{
    AgentCapabilities, AgentCard, AgentInterface, Message, Part, Role, SendMessageRequest,
    SendMessageResponse,
};

#[derive(Clone)]
struct EchoAgent;

#[async_trait::async_trait]
impl A2AHandler for EchoAgent {
    async fn get_agent_card(&self) -> Result<AgentCard, A2AError> {
        Ok(AgentCard {
            name: "Echo Agent".to_owned(),
            description: "Minimal example A2A agent".to_owned(),
            supported_interfaces: vec![
                AgentInterface {
                    url: "/rpc".to_owned(),
                    protocol_binding: "JSONRPC".to_owned(),
                    tenant: None,
                    protocol_version: "1.0".to_owned(),
                },
                AgentInterface {
                    url: "/".to_owned(),
                    protocol_binding: "HTTP+JSON".to_owned(),
                    tenant: None,
                    protocol_version: "1.0".to_owned(),
                },
            ],
            provider: None,
            version: "1.0.0".to_owned(),
            documentation_url: None,
            capabilities: AgentCapabilities {
                streaming: Some(false),
                push_notifications: Some(false),
                extensions: Vec::new(),
                extended_agent_card: Some(false),
            },
            security_schemes: Default::default(),
            security_requirements: Vec::new(),
            default_input_modes: vec!["text/plain".to_owned()],
            default_output_modes: vec!["text/plain".to_owned()],
            skills: Vec::new(),
            signatures: Vec::new(),
            icon_url: None,
        })
    }

    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, A2AError> {
        let reply = request
            .message
            .parts
            .iter()
            .find_map(|part| part.text.as_deref())
            .unwrap_or("hello");

        Ok(SendMessageResponse::Message(Message {
            message_id: "msg-echo-1".to_owned(),
            context_id: request.message.context_id,
            task_id: None,
            role: Role::Agent,
            parts: vec![Part {
                text: Some(format!("echo: {reply}")),
                raw: None,
                url: None,
                data: None,
                metadata: None,
                filename: None,
                media_type: None,
            }],
            metadata: None,
            extensions: Vec::new(),
            reference_task_ids: Vec::new(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("echo server listening on http://127.0.0.1:3000");
    axum::serve(listener, router(EchoAgent)).await?;
    Ok(())
}
