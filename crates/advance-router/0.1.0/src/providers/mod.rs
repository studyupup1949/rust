#[cfg(feature = "openai")]
pub mod openai;

#[cfg(any(
    feature = "openai",
    feature = "deepseek",
    feature = "grok",
    feature = "minimax",
    feature = "glm",
    feature = "qwen"
))]
pub mod openai_compat;

#[cfg(feature = "deepseek")]
pub mod deepseek;

#[cfg(feature = "grok")]
pub mod grok;

#[cfg(feature = "minimax")]
pub mod minimax;

#[cfg(feature = "glm")]
pub mod glm;

#[cfg(feature = "qwen")]
pub mod qwen;

#[cfg(feature = "anthropic")]
pub mod anthropic;

#[cfg(feature = "gemini")]
pub mod gemini;
