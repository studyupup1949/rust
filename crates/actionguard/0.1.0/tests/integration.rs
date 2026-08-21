use actionguard::policies::{AllowList, ArgMatchesRegex, CustomAsyncPolicy, DenyList};
use actionguard::{AsyncPolicySet, Decision, PolicySet, ToolCall, Vote};

#[test]
fn fails_closed_when_nothing_allows_the_call() {
    let policies = PolicySet::new();
    let call = ToolCall::new("anything", serde_json::json!({}));
    assert_eq!(
        policies.check(&call),
        Decision::Deny("no policy allowed this call (fail-closed default)".to_string())
    );
}

#[test]
fn allow_list_permits_listed_tools_only() {
    let policies = PolicySet::new().with(AllowList::new(["read_file", "search"]));
    assert_eq!(
        policies.check(&ToolCall::new("read_file", serde_json::json!({}))),
        Decision::Allow
    );
    assert!(matches!(
        policies.check(&ToolCall::new("shell_exec", serde_json::json!({}))),
        Decision::Deny(_)
    ));
}

#[test]
fn deny_overrides_beats_an_allow_list() {
    let policies = PolicySet::new()
        .with(AllowList::new(["shell_exec"]))
        .with(DenyList::new(["shell_exec"]));

    let decision = policies.check(&ToolCall::new("shell_exec", serde_json::json!({})));
    assert!(matches!(decision, Decision::Deny(_)));
}

#[test]
fn arg_matches_regex_scopes_a_path_even_when_the_tool_is_allowed() {
    let policies = PolicySet::new()
        .with(AllowList::new(["read_file"]))
        .with(ArgMatchesRegex::new("read_file", "path", r"^/workspace/.*").unwrap());

    let inside = ToolCall::new(
        "read_file",
        serde_json::json!({"path": "/workspace/notes.txt"}),
    );
    let outside = ToolCall::new("read_file", serde_json::json!({"path": "/etc/passwd"}));

    assert_eq!(policies.check(&inside), Decision::Allow);
    assert!(matches!(policies.check(&outside), Decision::Deny(_)));
}

#[test]
fn arg_matches_regex_abstains_for_other_tools() {
    let policies = PolicySet::new()
        .with(AllowList::new(["search"]))
        .with(ArgMatchesRegex::new("read_file", "path", r"^/workspace/.*").unwrap());

    // search isn't read_file, so the regex policy abstains and only AllowList votes
    assert_eq!(
        policies.check(&ToolCall::new("search", serde_json::json!({"q": "rust"}))),
        Decision::Allow
    );
}

#[tokio::test]
async fn async_policy_only_runs_after_sync_deny_short_circuits() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let called = Arc::new(AtomicBool::new(false));
    let called_in_closure = called.clone();

    let policies = AsyncPolicySet::from_sync(PolicySet::new().with(DenyList::new(["shell_exec"])))
        .with_async(CustomAsyncPolicy::new("would_call_llm", move |_call| {
            let called = called_in_closure.clone();
            async move {
                called.store(true, Ordering::SeqCst);
                Vote::Allow
            }
        }));

    let decision = policies
        .check(&ToolCall::new("shell_exec", serde_json::json!({})))
        .await;

    assert!(matches!(decision, Decision::Deny(_)));
    assert!(!called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn async_policy_can_deny_based_on_intent() {
    let policies = AsyncPolicySet::new()
        .with(AllowList::new(["send_email"]))
        .with_async(CustomAsyncPolicy::new(
            "matches_intent",
            |call| async move {
                let to = call.argument_str("to").unwrap_or_default();
                if to == "boss@company.com" {
                    Vote::Allow
                } else {
                    Vote::Deny("recipient wasn't mentioned in the user's request".to_string())
                }
            },
        ));

    let expected = ToolCall::new("send_email", serde_json::json!({"to": "boss@company.com"}));
    let unexpected = ToolCall::new(
        "send_email",
        serde_json::json!({"to": "randomperson@example.com"}),
    );

    assert_eq!(policies.check(&expected).await, Decision::Allow);
    assert!(matches!(
        policies.check(&unexpected).await,
        Decision::Deny(_)
    ));
}
