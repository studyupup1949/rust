use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
        ChatCompletionRequestFunctionMessage, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        ChatCompletionTool, ChatCompletionToolChoiceOption, ChatCompletionToolType,
        CreateChatCompletionRequest, FunctionObject,
    },
};
use async_trait::async_trait;

use crate::{error::AgentError, tool::Tool, types::RunContext};

/// Trait for language models that can be used by agents
#[async_trait]
pub trait Model: Send + Sync {
    /// Generate a response based on the context and available tools
    async fn generate_response(
        &self,
        context: &mut RunContext,
        tools: &[&dyn Tool],
    ) -> Result<String, AgentError>;
}

/// OpenAI model implementation
pub struct OpenAI {
    client: Client<OpenAIConfig>,
    /// See the [model endpoint compatibility](https://platform.openai.com/docs/models#model-endpoint-compatibility) table for details on which models work with the Chat API.
    model: String,
}

impl OpenAI {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        Self {
            client,
            model: model.into(),
        }
    }

    fn create_messages(&self, context: &RunContext) -> Vec<ChatCompletionRequestMessage> {
        context
            .messages
            .iter()
            .map(|msg| match msg.role.as_str() {
                "system" => {
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content: ChatCompletionRequestSystemMessageContent::Text(
                            msg.content.clone(),
                        ),
                        name: msg.tool_name.clone(),
                    })
                }
                "assistant" =>
                {
                    #[allow(deprecated)]
                    ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
                        content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                            msg.content.clone(),
                        )),
                        name: msg.tool_name.clone(),
                        tool_calls: None,
                        function_call: None,
                        audio: None,
                        refusal: None,
                    })
                }
                "tool" => {
                    ChatCompletionRequestMessage::Function(ChatCompletionRequestFunctionMessage {
                        content: Some(msg.content.clone()),
                        name: msg.tool_name.clone().unwrap_or_default(),
                    })
                }
                _ => ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                    name: msg.tool_name.clone(),
                }),
            })
            .collect()
    }

    fn create_tools(&self, tools: &[&dyn Tool]) -> Vec<ChatCompletionTool> {
        tools
            .iter()
            .map(|tool| ChatCompletionTool {
                r#type: ChatCompletionToolType::Function,
                function: FunctionObject {
                    name: tool.name().to_string(),
                    description: Some(tool.description().to_string()),
                    parameters: Some(tool.parameters_schema()),
                    strict: None,
                },
            })
            .collect()
    }
}

#[async_trait]
impl Model for OpenAI {
    async fn generate_response(
        &self,
        context: &mut RunContext,
        tools: &[&dyn Tool],
    ) -> Result<String, AgentError> {
        let messages = self.create_messages(context);
        let openai_tools = self.create_tools(tools);

        let request = CreateChatCompletionRequest {
            model: self.model.clone(),
            messages,
            tools: Some(openai_tools),
            tool_choice: Some(ChatCompletionToolChoiceOption::Auto),
            temperature: Some(0.7),
            ..Default::default()
        };

        let response =
            self.client.chat().create(request).await.map_err(|e| {
                AgentError::ModelError(format!("Failed to generate response: {}", e))
            })?;

        let message = response.choices[0].message.clone();

        if let Some(tool_calls) = message.tool_calls {
            if let Some(tool_call) = tool_calls.first() {
                let tool = tools
                    .iter()
                    .find(|t| t.name() == tool_call.function.name)
                    .ok_or_else(|| {
                        AgentError::ToolError(format!(
                            "Tool not found: {}",
                            tool_call.function.name
                        ))
                    })?;

                let result = tool.execute(context, &tool_call.function.arguments).await?;
                context.add_tool_message(result.tool_name, result.output);
                self.generate_response(context, tools).await
            } else {
                Ok(message.content.unwrap_or_default())
            }
        } else {
            Ok(message.content.unwrap_or_default())
        }
    }
}
