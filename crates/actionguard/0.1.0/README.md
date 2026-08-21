# actionguard

[![crates.io](https://img.shields.io/crates/v/actionguard.svg?style=flat)](https://crates.io/crates/actionguard)
[![docs.rs](https://docs.rs/actionguard/badge.svg?style=flat)](https://docs.rs/actionguard)
[![CI](https://github.com/thaicn1712/actionguard/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/thaicn1712/actionguard/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/actionguard.svg?style=flat)](LICENSE)

Policy-as-code for AI agent tool calls, in Rust — the pattern security teams already use for cloud infrastructure (OPA/Rego, AWS Cedar: a policy decision point in front of every action, deny-overrides, fail-closed by default) applied to agent tool calls instead of API requests. `guardflow` validates what an agent *says*; `actionguard` validates what it's about to *do*, before it does it.

## Install

```bash
cargo add actionguard
```

## Usage

```rust,ignore
use actionguard::{PolicySet, ToolCall};
use actionguard::policies::{AllowList, DenyList, ArgMatchesRegex};

let policies = PolicySet::new()
    .with(AllowList::new(["read_file", "search", "send_email"]))
    .with(DenyList::new(["rm_rf", "drop_table", "shell_exec"]))
    .with(ArgMatchesRegex::new("read_file", "path", r"^/workspace/.*"));

let call = ToolCall::new("read_file", serde_json::json!({ "path": "/workspace/notes.txt" }));

match policies.check(&call) {
    actionguard::Decision::Allow => run_the_tool(call),
    actionguard::Decision::Deny(reason) => println!("blocked: {reason}"),
}
```

**Fail-closed by default**: if no policy explicitly allows a call, it's denied — the same default OPA and every serious authorization system ships with, and the opposite of what most hand-rolled "if command contains rm" checks do.

**Deny-overrides**: any policy voting `Deny` blocks the call outright, even if another policy voted `Allow` — you can't accidentally allowlist your way past an explicit deny rule.

## Async policies

For checks that need a model call — "does this action match what the user actually asked for" (see [Intent-Governed Tool Authorization](https://arxiv.org/abs/2606.22916)) — `AsyncPolicy` wraps an async closure, evaluated only after every sync policy has already voted:

```rust,ignore
use actionguard::AsyncPolicySet;
use actionguard::policies::CustomAsyncPolicy;

let policies = AsyncPolicySet::from_sync(policies).with_async(
    CustomAsyncPolicy::new("matches_intent", |call| async move {
        if call_is_consistent_with(&user_request, &call).await {
            actionguard::Vote::Allow
        } else {
            actionguard::Vote::Deny("not consistent with the user's request".into())
        }
    }),
);
```

## Examples

```bash
cargo run --example agent_dispatcher   # sync path-scoping + async intent check, full story
```

## Benchmarks

`cargo bench` ([`benches/overhead.rs`](benches/overhead.rs)):

| Scenario | Time |
|---|---|
| `PolicySet::check`, 3 policies, allowed | ~104 ns |

## License

MIT
