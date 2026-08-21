use crate::{AsyncPolicy, Policy, ToolCall, Vote};
use std::collections::HashSet;
use std::future::Future;

/// Votes `Allow` for calls whose tool name is in the list, `Abstain` otherwise —
/// abstaining (not denying) on the rest lets this compose with other policies
/// instead of being the sole word on every call.
pub struct AllowList {
    names: HashSet<String>,
}

impl AllowList {
    pub fn new(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            names: names.into_iter().map(Into::into).collect(),
        }
    }
}

impl Policy for AllowList {
    fn name(&self) -> &str {
        "allow_list"
    }

    fn vote(&self, call: &ToolCall) -> Vote {
        if self.names.contains(&call.name) {
            Vote::Allow
        } else {
            Vote::Abstain
        }
    }
}

/// Votes `Deny` for calls whose tool name is in the list, `Abstain` otherwise.
/// A `Deny` here wins regardless of what any [`AllowList`] says.
pub struct DenyList {
    names: HashSet<String>,
}

impl DenyList {
    pub fn new(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            names: names.into_iter().map(Into::into).collect(),
        }
    }
}

impl Policy for DenyList {
    fn name(&self) -> &str {
        "deny_list"
    }

    fn vote(&self, call: &ToolCall) -> Vote {
        if self.names.contains(&call.name) {
            Vote::Deny(format!("{} is on the deny list", call.name))
        } else {
            Vote::Abstain
        }
    }
}

/// For calls to `tool`, requires the string argument `argument` to match `pattern`.
/// Abstains for other tools. For `tool`, votes `Allow` on a match and `Deny` on a
/// missing/non-string/non-matching argument — e.g. keeping a `read_file` call
/// inside `/workspace` regardless of what an [`AllowList`] says about the tool name.
pub struct ArgMatchesRegex {
    tool: String,
    argument: String,
    pattern: regex::Regex,
}

impl ArgMatchesRegex {
    pub fn new(
        tool: impl Into<String>,
        argument: impl Into<String>,
        pattern: &str,
    ) -> Result<Self, regex::Error> {
        Ok(Self {
            tool: tool.into(),
            argument: argument.into(),
            pattern: regex::Regex::new(pattern)?,
        })
    }
}

impl Policy for ArgMatchesRegex {
    fn name(&self) -> &str {
        "arg_matches_regex"
    }

    fn vote(&self, call: &ToolCall) -> Vote {
        if call.name != self.tool {
            return Vote::Abstain;
        }
        match call.argument_str(&self.argument) {
            Some(value) if self.pattern.is_match(value) => Vote::Allow,
            Some(value) => Vote::Deny(format!(
                "{}={value:?} does not match /{}/",
                self.argument,
                self.pattern.as_str()
            )),
            None => Vote::Deny(format!(
                "missing or non-string argument {:?}",
                self.argument
            )),
        }
    }
}

/// Wraps an async closure as an [`AsyncPolicy`] — the escape hatch for checks this
/// crate can't sensibly hardcode: an LLM-as-judge asking whether an action matches
/// the user's stated intent, a call to an external policy service.
pub struct CustomAsyncPolicy<F> {
    name: String,
    vote: F,
}

impl<F, Fut> CustomAsyncPolicy<F>
where
    F: Fn(ToolCall) -> Fut + Send + Sync,
    Fut: Future<Output = Vote> + Send,
{
    pub fn new(name: impl Into<String>, vote: F) -> Self {
        Self {
            name: name.into(),
            vote,
        }
    }
}

#[async_trait::async_trait]
impl<F, Fut> AsyncPolicy for CustomAsyncPolicy<F>
where
    F: Fn(ToolCall) -> Fut + Send + Sync,
    Fut: Future<Output = Vote> + Send,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn vote(&self, call: &ToolCall) -> Vote {
        (self.vote)(call.clone()).await
    }
}
