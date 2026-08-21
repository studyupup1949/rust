use a2a_rust::client::A2AClient;
use a2a_rust::types::{Message, Part, Role, SendMessageRequest, SendMessageResponse};

#[tokio::main]
async fn main() -> Result<(), a2a_rust::A2AError> {
    let client = A2AClient::new("http://127.0.0.1:3000")?;
    let card = client.discover_agent_card().await?;
    println!("discovered agent: {}", card.name);

    let response = client
        .send_message(SendMessageRequest {
            message: Message {
                message_id: "msg-1".to_owned(),
                context_id: Some("ctx-1".to_owned()),
                task_id: None,
                role: Role::User,
                parts: vec![Part {
                    text: Some("ping".to_owned()),
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
            },
            configuration: None,
            metadata: None,
            tenant: None,
        })
        .await?;

    match response {
        SendMessageResponse::Message(message) => {
            let reply = message
                .parts
                .iter()
                .find_map(|part| part.text.as_deref())
                .unwrap_or("<no text part>");
            println!("reply: {reply}");
        }
        SendMessageResponse::Task(task) => {
            println!("task created: {}", task.id);
        }
    }

    Ok(())
}
