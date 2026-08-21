use a2a_rust::types::{
    AgentCard, Role, SendMessageRequest, SendMessageResponse, StreamResponse, TaskState,
};

#[test]
fn agent_card_deserializes_spec_discovery_example() {
    let json = serde_json::json!({
        "name": "Research Agent",
        "description": "AI assistant specialized in academic and technical research with comprehensive source citation",
        "supportedInterfaces": [
            {
                "url": "https://research-agent.example.com/a2a/v1",
                "protocolBinding": "HTTP+JSON",
                "protocolVersion": "1.0"
            }
        ],
        "version": "1.2.0",
        "capabilities": {
            "streaming": false,
            "pushNotifications": false,
            "extensions": [
                {
                    "uri": "https://example.com/extensions/citations/v1",
                    "description": "Returns citation metadata alongside answers",
                    "required": false
                }
            ]
        },
        "defaultInputModes": ["text/plain"],
        "defaultOutputModes": ["text/plain"],
        "skills": [
            {
                "id": "academic-research",
                "name": "Academic Research Assistant",
                "description": "Provides research assistance with citations and source verification",
                "tags": ["research", "citations", "academic"]
            }
        ]
    });

    let card: AgentCard = serde_json::from_value(json).expect("agent card should deserialize");
    let serialized = serde_json::to_value(&card).expect("agent card should serialize");

    assert_eq!(card.name, "Research Agent");
    assert_eq!(card.supported_interfaces[0].protocol_binding, "HTTP+JSON");
    assert_eq!(card.skills[0].id, "academic-research");
    assert_eq!(
        card.capabilities.extensions[0].uri,
        "https://example.com/extensions/citations/v1"
    );
    assert_eq!(serialized["name"], "Research Agent");
    assert_eq!(
        serialized["supportedInterfaces"][0]["protocolBinding"],
        "HTTP+JSON"
    );
}

#[test]
fn send_message_request_deserializes_spec_booking_example() {
    let json = serde_json::json!({
        "message": {
            "messageId": "msg-1",
            "role": "ROLE_USER",
            "parts": [
                {
                    "text": "Book me a flight from San Francisco to London next Friday."
                }
            ]
        }
    });

    let request: SendMessageRequest =
        serde_json::from_value(json).expect("request should deserialize");
    request.validate().expect("request should validate");
    let serialized = serde_json::to_value(&request).expect("request should serialize");

    assert_eq!(request.message.message_id, "msg-1");
    assert!(matches!(request.message.role, Role::User));
    assert_eq!(
        request.message.parts[0].text.as_deref(),
        Some("Book me a flight from San Francisco to London next Friday.")
    );
    assert_eq!(serialized["message"]["role"], "ROLE_USER");
    assert_eq!(
        serialized["message"]["parts"][0]["text"],
        "Book me a flight from San Francisco to London next Friday."
    );
}

#[test]
fn send_message_response_deserializes_proto_first_input_required_example() {
    let json = serde_json::json!({
        "task": {
            "id": "task-123",
            "contextId": "ctx-123",
            "status": {
                "state": "TASK_STATE_INPUT_REQUIRED",
                "message": {
                    "messageId": "msg-2",
                    "contextId": "ctx-123",
                    "taskId": "task-123",
                    "role": "ROLE_AGENT",
                    "parts": [
                        {
                            "text": "I need more details. Where would you like to fly from and to?"
                        }
                    ]
                }
            }
        }
    });

    let response: SendMessageResponse =
        serde_json::from_value(json).expect("response should deserialize");
    response.validate().expect("response should validate");
    let serialized = serde_json::to_value(&response).expect("response should serialize");

    match response {
        SendMessageResponse::Task(task) => {
            assert_eq!(task.id, "task-123");
            assert_eq!(task.context_id.as_deref(), Some("ctx-123"));
            assert_eq!(task.status.state, TaskState::InputRequired);
        }
        SendMessageResponse::Message(_) => panic!("expected task response"),
    }
    assert_eq!(
        serialized["task"]["status"]["state"],
        "TASK_STATE_INPUT_REQUIRED"
    );
}

#[test]
fn stream_response_deserializes_status_update_example() {
    let json = serde_json::json!({
        "statusUpdate": {
            "taskId": "task-123",
            "contextId": "ctx-123",
            "status": {
                "state": "TASK_STATE_WORKING",
                "message": {
                    "messageId": "msg-3",
                    "contextId": "ctx-123",
                    "taskId": "task-123",
                    "role": "ROLE_AGENT",
                    "parts": [
                        {
                            "text": "Still searching for flights..."
                        }
                    ]
                }
            }
        }
    });

    let response: StreamResponse = serde_json::from_value(json).expect("stream should deserialize");
    response.validate().expect("stream should validate");
    let serialized = serde_json::to_value(&response).expect("stream should serialize");

    match response {
        StreamResponse::StatusUpdate(update) => {
            assert_eq!(update.task_id, "task-123");
            assert_eq!(update.context_id, "ctx-123");
            assert_eq!(update.status.state, TaskState::Working);
        }
        _ => panic!("expected status update"),
    }
    assert_eq!(
        serialized["statusUpdate"]["status"]["state"],
        "TASK_STATE_WORKING"
    );
}

#[test]
fn stream_response_deserializes_artifact_update_example() {
    let json = serde_json::json!({
        "artifactUpdate": {
            "taskId": "task-123",
            "contextId": "ctx-123",
            "artifact": {
                "artifactId": "artifact-1",
                "parts": [
                    {
                        "text": "Partial itinerary"
                    }
                ]
            },
            "append": true,
            "lastChunk": false
        }
    });

    let response: StreamResponse = serde_json::from_value(json).expect("stream should deserialize");
    response.validate().expect("stream should validate");
    let serialized = serde_json::to_value(&response).expect("stream should serialize");

    match response {
        StreamResponse::ArtifactUpdate(update) => {
            assert_eq!(update.task_id, "task-123");
            assert!(update.append);
            assert!(!update.last_chunk);
        }
        _ => panic!("expected artifact update"),
    }
    assert_eq!(serialized["artifactUpdate"]["append"], true);
}
