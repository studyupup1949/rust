use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use pin_project_lite::pin_project;

use crate::error::RouterError;
use crate::types::message::ContentPart;
use crate::types::response::{ChatResponse, FinishReason, StreamEvent, Usage};


pin_project! {
    /// A stream of chat completion events from any provider.
    ///
    /// Wraps a provider-specific stream into a unified async stream of `StreamEvent`.
    /// Can be consumed event-by-event or collected into a full `ChatResponse`.
    pub struct ChatStream {
        #[pin]
        inner: Pin<Box<dyn Stream<Item = Result<StreamEvent, RouterError>> + Send>>,
    }
}

impl ChatStream {
    /// Create a new ChatStream from any async stream of StreamEvents.
    pub fn new(stream: impl Stream<Item = Result<StreamEvent, RouterError>> + Send + 'static) -> Self {
        Self {
            inner: Box::pin(stream),
        }
    }

    /// Consume the entire stream and assemble a full ChatResponse.
    pub async fn collect(self) -> Result<ChatResponse, RouterError> {
        use futures::StreamExt;

        let mut state = StreamCollector::default();
        let mut stream = self;

        while let Some(event) = stream.next().await {
            let event = event?;
            state.process_event(event);
        }

        Ok(state.into_response())
    }
}

impl Stream for ChatStream {
    type Item = Result<StreamEvent, RouterError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        this.inner.as_mut().poll_next(cx)
    }
}

/// Internal state for assembling a ChatResponse from stream events.
#[derive(Default)]
struct StreamCollector {
    text: String,
    thinking: String,
    tool_calls: Vec<ToolCallAccumulator>,
    usage: Option<Usage>,
    finish_reason: Option<FinishReason>,
}

#[derive(Default)]
struct ToolCallAccumulator {
    id: String,
    name: String,
    arguments_json: String,
}

impl StreamCollector {
    fn process_event(&mut self, event: StreamEvent) {
        match event {
            StreamEvent::Delta { content } => {
                self.text.push_str(&content);
            }
            StreamEvent::ThinkingDelta { content } => {
                self.thinking.push_str(&content);
            }
            StreamEvent::ToolCallDelta {
                index,
                id,
                name,
                arguments_delta,
            } => {
                while self.tool_calls.len() <= index {
                    self.tool_calls.push(ToolCallAccumulator::default());
                }
                let tc = &mut self.tool_calls[index];
                if let Some(id) = id {
                    tc.id = id;
                }
                if let Some(name) = name {
                    tc.name = name;
                }
                tc.arguments_json.push_str(&arguments_delta);
            }
            StreamEvent::ToolCallComplete(tc) => {
                // If we get a complete tool call, add it directly
                let acc = ToolCallAccumulator {
                    id: tc.id,
                    name: tc.name,
                    arguments_json: tc.arguments.to_string(),
                };
                self.tool_calls.push(acc);
            }
            StreamEvent::Usage(usage) => {
                self.usage = Some(usage);
            }
            StreamEvent::Done { finish_reason } => {
                self.finish_reason = Some(finish_reason);
            }
        }
    }

    fn into_response(self) -> ChatResponse {
        let mut content = Vec::new();

        if !self.text.is_empty() {
            content.push(ContentPart::Text {
                text: self.text,
            });
        }

        for tc in self.tool_calls {
            let arguments = serde_json::from_str(&tc.arguments_json)
                .unwrap_or(serde_json::Value::String(tc.arguments_json));
            content.push(ContentPart::ToolUse {
                id: tc.id,
                name: tc.name,
                arguments,
            });
        }

        let thinking = if self.thinking.is_empty() {
            None
        } else {
            Some(self.thinking)
        };

        ChatResponse {
            id: String::new(),
            model: String::new(),
            content,
            finish_reason: self.finish_reason.unwrap_or(FinishReason::Stop),
            usage: self.usage.unwrap_or_default(),
            thinking,
            raw: serde_json::Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    #[tokio::test]
    async fn test_collect_text_stream() {
        let events = vec![
            Ok(StreamEvent::Delta {
                content: "Hello".into(),
            }),
            Ok(StreamEvent::Delta {
                content: " world".into(),
            }),
            Ok(StreamEvent::Usage(Usage {
                prompt_tokens: 10,
                completion_tokens: 2,
                total_tokens: 12,
                ..Default::default()
            })),
            Ok(StreamEvent::Done {
                finish_reason: FinishReason::Stop,
            }),
        ];

        let chat_stream = ChatStream::new(stream::iter(events));
        let response = chat_stream.collect().await.unwrap();

        assert_eq!(response.text(), "Hello world");
        assert_eq!(response.finish_reason, FinishReason::Stop);
        assert_eq!(response.usage.total_tokens, 12);
    }

    #[tokio::test]
    async fn test_collect_tool_call_stream() {
        let events = vec![
            Ok(StreamEvent::ToolCallDelta {
                index: 0,
                id: Some("call_1".into()),
                name: Some("get_weather".into()),
                arguments_delta: r#"{"lo"#.into(),
            }),
            Ok(StreamEvent::ToolCallDelta {
                index: 0,
                id: None,
                name: None,
                arguments_delta: r#"cation":"NYC"}"#.into(),
            }),
            Ok(StreamEvent::Done {
                finish_reason: FinishReason::ToolUse,
            }),
        ];

        let chat_stream = ChatStream::new(stream::iter(events));
        let response = chat_stream.collect().await.unwrap();

        let tool_calls = response.tool_calls();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "get_weather");
        assert_eq!(tool_calls[0].arguments["location"], "NYC");
    }
}
