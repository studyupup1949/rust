use activitystreams_vocabulary::Context;

/// Represents the canonical [ForgeFed](https://forgefed.org/spec) namespace URI.
pub const FORGEFED_CONTEXT: &str = "https://forgefed.org/ns";

/// Creates the default [ForgeFed](https://forgefed.org/spec) context.
///
/// Includes the base [ActivityStreams](https://www.w3.org/TR/activitystreams-vocabulary) context,
/// and the [ForgeFed](https://forgefed.org/spec) namespace URI.
#[inline]
pub fn forgefed_context() -> Context {
    Context::array([
        serde_json::Value::String(Context::URI.into()),
        serde_json::Value::String(FORGEFED_CONTEXT.into()),
    ])
}
