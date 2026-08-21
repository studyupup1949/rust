use actionguard::policies::{AllowList, ArgMatchesRegex, DenyList};
use actionguard::{Decision, PolicySet, ToolCall};
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_three_policies_allow(c: &mut Criterion) {
    let policies = PolicySet::new()
        .with(AllowList::new(["read_file", "search", "send_email"]))
        .with(DenyList::new(["rm_rf", "drop_table", "shell_exec"]))
        .with(ArgMatchesRegex::new("read_file", "path", r"^/workspace/.*").unwrap());

    let call = ToolCall::new(
        "read_file",
        serde_json::json!({ "path": "/workspace/notes.txt" }),
    );

    c.bench_function("PolicySet::check, 3 policies, allowed", |b| {
        b.iter(|| {
            let decision = policies.check(&call);
            assert_eq!(decision, Decision::Allow);
        });
    });
}

criterion_group!(benches, bench_three_policies_allow);
criterion_main!(benches);
