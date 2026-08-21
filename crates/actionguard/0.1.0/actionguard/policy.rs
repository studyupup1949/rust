#![doc = include_str!("../README.md")]

use serde_json::Value;

pub mod policies;

/// A tool/action an agent is about to take.
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

impl ToolCall {
    pub fn new(name: impl Into<String>, arguments: Value) -> Self {
        Self {
            name: name.into(),
            arguments,
        }
    }

    pub fn argument(&self, key: &str) -> Option<&Value> {
        self.arguments.get(key)
    }

    pub fn argument_str(&self, key: &str) -> Option<&str> {
        self.arguments.get(key).and_then(|v| v.as_str())
    }
}

/// One policy's vote on a [`ToolCall`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vote {
    Allow,
    Deny(String),
    Abstain,
}

/// The outcome of [`PolicySet::check`] / [`AsyncPolicySet::check`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny(String),
}

/// A sync check that votes on whether a [`ToolCall`] should proceed.
pub trait Policy: Send + Sync {
    fn name(&self) -> &str;
    fn vote(&self, call: &ToolCall) -> Vote;
}

/// A check that needs a network call — an LLM-as-judge asking whether an action
/// matches the user's actual intent, a call to an external policy service.
#[async_trait::async_trait]
pub trait AsyncPolicy: Send + Sync {
    fn name(&self) -> &str;
    async fn vote(&self, call: &ToolCall) -> Vote;
}

/// A set of [`Policy`]s evaluated deny-overrides, fail-closed by default: any
/// `Deny` vote wins outright regardless of `Allow` votes elsewhere; if nothing
/// explicitly `Allow`s (every policy abstained), the call is denied.
#[derive(Default)]
pub struct PolicySet {
    policies: Vec<Box<dyn Policy>>,
}

impl PolicySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, policy: impl Policy + 'static) -> Self {
        self.policies.push(Box::new(policy));
        self
    }

    pub fn check(&self, call: &ToolCall) -> Decision {
        decide(self.policies.iter().map(|p| (p.name(), p.vote(call))))
    }

    fn into_policies(self) -> Vec<Box<dyn Policy>> {
        self.policies
    }
}

/// Like [`PolicySet`], but also runs [`AsyncPolicy`]s. Sync policies run first
/// (cheap); an explicit sync `Deny` short-circuits before any network call.
#[derive(Default)]
pub struct AsyncPolicySet {
    sync: Vec<Box<dyn Policy>>,
    r#async: Vec<Box<dyn AsyncPolicy>>,
}

impl AsyncPolicySet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start from an existing [`PolicySet`]'s sync policies.
    pub fn from_sync(set: PolicySet) -> Self {
        Self {
            sync: set.into_policies(),
            r#async: Vec::new(),
        }
    }

    pub fn with(mut self, policy: impl Policy + 'static) -> Self {
        self.sync.push(Box::new(policy));
        self
    }

    pub fn with_async(mut self, policy: impl AsyncPolicy + 'static) -> Self {
        self.r#async.push(Box::new(policy));
        self
    }

    pub async fn check(&self, call: &ToolCall) -> Decision {
        let sync_votes: Vec<(&str, Vote)> =
            self.sync.iter().map(|p| (p.name(), p.vote(call))).collect();
        if let Some(deny) = find_deny(&sync_votes) {
            return deny;
        }

        let mut all_votes = sync_votes;
        for p in &self.r#async {
            let vote = p.vote(call).await;
            if let Vote::Deny(reason) = &vote {
                return Decision::Deny(format!("{}: {reason}", p.name()));
            }
            all_votes.push((p.name(), vote));
        }

        decide(all_votes)
    }
}

fn find_deny(votes: &[(&str, Vote)]) -> Option<Decision> {
    votes.iter().find_map(|(name, vote)| match vote {
        Vote::Deny(reason) => Some(Decision::Deny(format!("{name}: {reason}"))),
        _ => None,
    })
}

fn decide<'a>(votes: impl IntoIterator<Item = (&'a str, Vote)>) -> Decision {
    let mut allowed = false;
    for (name, vote) in votes {
        match vote {
            Vote::Deny(reason) => return Decision::Deny(format!("{name}: {reason}")),
            Vote::Allow => allowed = true,
            Vote::Abstain => {}
        }
    }
    if allowed {
        Decision::Allow
    } else {
        Decision::Deny("no policy allowed this call (fail-closed default)".to_string())
    }
}
