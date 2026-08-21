# a3s-flow

**Workflow engine for agentic platforms** — Execute JSON-defined DAGs with concurrent wave scheduling, pluggable node types, and full lifecycle control.

```rust
let engine = FlowEngine::new(NodeRegistry::with_defaults());
let id = engine.start(&definition, variables).await?;
engine.pause(id).await?;
engine.resume(id).await?;
println!("{:?}", engine.state(id).await?);
```

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/a3s-flow.svg)](https://crates.io/crates/a3s-flow)

---

## Why a3s-flow?

- **JSON-native** — workflows are plain JSON objects (`{ nodes, edges }`); no YAML, no DSL, no visual editor required to run them
- **Correct by construction** — cycle detection and reference validation happen at parse time, before a single node executes
- **Concurrent by default** — nodes with no mutual dependency run in the same wave via Tokio's `JoinSet`, not one-by-one
- **Full lifecycle control** — pause at wave boundaries, resume, or cancel mid-execution via `FlowEngine`
- **Extend without forking** — implement the `Node` trait to add any node type: LLM prompt, HTTP call, condition branch, sub-flow, script
- **A3S ecosystem integration** — designed to sit beside `a3s-power` (LLM inference), `a3s-event` (pub/sub hooks), and `a3s-lane` (priority scheduling)

---

## Architecture

```
  External Caller
  ┌─────────────────────────────────────────────────────────────┐
  │                       FlowEngine                            │
  │                                                             │
  │  node_types() → Vec<String>   list registered node types    │
  │  start(def, vars) → Uuid      parse DAG, spawn task, return │
  │  pause(id)                    signal pause at wave boundary  │
  │  resume(id)                   unblock a paused execution     │
  │  terminate(id)                cancel via CancellationToken   │
  │  state(id) → ExecutionState   snapshot current state        │
  │                                                             │
  │  ┌───────────────────────────────────────────────────────┐  │
  │  │               ExecutionState (per Uuid)               │  │
  │  │                                                       │  │
  │  │            ┌──────────┐                               │  │
  │  │   start() ►│ Running  │◄─── resume()                  │  │
  │  │            └────┬─────┘                               │  │
  │  │    pause() ─────┤  terminate()                        │  │
  │  │                 │  ├──────────────► Terminated        │  │
  │  │                 ▼  │  node error ► Failed(msg)        │  │
  │  │            ┌────────┴─┐  all done ► Completed(result) │  │
  │  │            │  Paused  │                               │  │
  │  │            └──────────┘                               │  │
  │  └───────────────────────────────────────────────────────┘  │
  └────────────────────────────┬────────────────────────────────┘
                               │  spawns background Tokio task
                               │  (watch::Receiver + CancellationToken)
                               ▼
  ┌─────────────────────────────────────────────────────────────┐
  │                       FlowRunner                            │
  │                (one task per execution)                     │
  │                                                             │
  │  run_controlled(execution_id, vars, signal_rx, cancel)      │
  │                                                             │
  │  ┌──────────────────────────────────────────────────────┐  │
  │  │  Between waves: check signal_rx + CancellationToken  │  │
  │  │                                                      │  │
  │  │   signal = Run  ─────────────────────► execute wave  │  │
  │  │   signal = Pause ──► wait for Run/cancel             │  │
  │  │   cancel.is_cancelled() ────────────► Terminated     │  │
  │  └──────────────────────────────────────────────────────┘  │
  │                                                             │
  │  Wave 1 │ fetch                    no deps → run now        │
  │         │  └─ outputs["fetch"]                             │
  │  Wave 2 │ summarize                fetch done → run now     │
  │         │  └─ outputs["summarize"]                         │
  │  Wave 3 │ branch_a  branch_b       both ready → concurrent  │
  │         │  └─ outputs["branch_a"], outputs["branch_b"]     │
  │  Wave 4 │ notify                   fan-in join → run now    │
  │         │  └─ outputs["notify"]                            │
  │                                                             │
  │  ┌──────────────────────────────────────────────────────┐  │
  │  │  Within wave (JoinSet drain): tokio::select!         │  │
  │  │                                                      │  │
  │  │   cancel.cancelled() ───────────────► Terminated     │  │
  │  │   join_next() → node result ────────► store output   │  │
  │  │   join_next() → node error  ────────► fail-fast      │  │
  │  └──────────────────────────────────────────────────────┘  │
  └──────────────────────┬──────────────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          │  parse definition (once)    │  resolve type per node
          ▼                             ▼
  ┌────────────────┐          ┌──────────────────────────────┐
  │    DagGraph    │          │        NodeRegistry           │
  │                │          │                              │
  │  1. Parse JSON │          │  "noop"                 (✓)  │
  │  2. Validate   │          │  "http-request"         (✓)  │
  │  3. Cycle det. │          │  "if-else"              (✓)  │
  │  4. Topo sort  │          │  "template-transform"   (✓)  │
  │                │          │  "variable-aggregator"  (✓)  │
  │                │          │  "code"                 (✓)  │
  │                │          │  "iteration"            (✓)  │
  │                │          │  "llm"                  (✓)  │
  │                │          │  "question-classifier"  (✓)  │
  │                │          │  "assign"               (✓)  │
  │                │          │  "parameter-extractor"  (✓)  │
  │                │          │  "loop"                 (✓)  │
  │                │          │  "list-operator"        (✓)  │
  │                │          │  "mcp"                  (✓)  │
  │                │          │  <any>     → CustomNode      │
  └────────────────┘          └──────────────────────────────┘
                                         │  execute(ExecContext)
                                         ▼
                              ┌──────────────────────────────┐
                              │         ExecContext          │
                              │                              │
                              │  data      — node config     │
                              │  inputs    — upstream output │
                              │  variables — global vars     │
                              └──────────────┬───────────────┘
                                             │  Result<Value>
                                             ▼
                              ┌──────────────────────────────┐
                              │         FlowResult           │
                              │                              │
                              │  execution_id : Uuid         │
                              │  outputs : Map<id, Value>    │
                              └──────────────────────────────┘
```

### Module map

```
src/
├── lib.rs                  — public re-exports
├── error.rs                — FlowError enum, Result<T> alias
├── condition.rs            — Condition, CondOp, Case, LogicalOp
├── engine.rs               — FlowEngine: lifecycle API
├── execution.rs            — ExecutionState, ExecutionHandle (internal)
├── graph.rs                — DagGraph: parse, validate, topo sort
├── node.rs                 — Node trait, ExecContext
├── registry.rs             — NodeRegistry: type string → Arc<dyn Node>
├── runner.rs               — FlowRunner: wave-based execution
└── nodes/
    ├── mod.rs
    ├── noop.rs             — "noop"
    ├── http.rs             — "http-request"
    ├── cond.rs             — "if-else"
    ├── template_transform.rs — "template-transform"
    ├── variable_aggregator.rs — "variable-aggregator"
    ├── code.rs             — "code" (Rhai)
    ├── iteration.rs        — "iteration" (concurrent sub-flow loop)
    ├── llm.rs              — "llm"
    ├── question_classifier.rs — "question-classifier"
    ├── assign.rs           — "assign"
    ├── parameter_extractor.rs — "parameter-extractor"
    ├── loop_node.rs        — "loop"
    ├── list_operator.rs    — "list-operator"
    └── mcp.rs              — "mcp" (Model Context Protocol client)
```

---

## Flow definition format

A flow is a JSON object with two arrays — **`nodes`** and **`edges`** — mirroring Dify's workflow format:

```json
{
  "nodes": [
    {
      "id":   "fetch_data",
      "type": "http-request",
      "data": { "url": "https://api.example.com/items", "method": "GET" }
    },
    {
      "id":   "check_ok",
      "type": "if-else",
      "data": { "cases": [{ "id": "ok", "conditions": [{ "from": "fetch_data", "path": "status", "op": "eq", "value": 200 }] }] }
    },
    {
      "id":   "notify",
      "type": "http-request",
      "data": {
        "url":    "https://hooks.example.com/done",
        "method": "POST",
        "run_if": { "from": "check_ok", "path": "branch", "op": "eq", "value": "ok" }
      }
    }
  ],
  "edges": [
    { "source": "fetch_data", "target": "check_ok" },
    { "source": "check_ok",   "target": "notify" }
  ]
}
```

**Node fields:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `id` | `string` | ✓ | Unique node identifier within the flow |
| `type` | `string` | ✓ | Node type — looked up in `NodeRegistry` |
| `data` | `object` | | Static node configuration (prompt, URL, script body, …) |

**Edge fields:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `source` | `string` | ✓ | ID of the upstream node |
| `target` | `string` | ✓ | ID of the downstream node |

**`run_if` guard** — place inside `data` to conditionally skip the node:

```json
"data": {
  "run_if": { "from": "upstream_id", "path": "branch", "op": "eq", "value": "ok" }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `from` | `string` | Upstream node ID to read from |
| `path` | `string` | Dot-separated path into the output (e.g. `"body.count"`; `""` for root) |
| `op` | `string` | `eq` \| `ne` \| `gt` \| `lt` \| `gte` \| `lte` \| `contains` |
| `value` | any JSON | Right-hand side of the comparison |

Skip propagates automatically: if a node is skipped, any downstream node whose `run_if.from` points to it will also be skipped.

**Validation rules** (enforced at `DagGraph::from_json` time, before execution):

- At least one node must be present
- All node IDs must be unique
- Every ID listed in an edge's `source`/`target` must reference a node defined in `nodes`
- The graph must be acyclic

---

## Quick start

```toml
# Cargo.toml
[dependencies]
a3s-flow  = "0.1"
tokio     = { version = "1", features = ["full"] }
serde_json = "1"
```

### Via `FlowEngine` (recommended)

`FlowEngine` is the primary API. It owns the node registry and all running executions.

```rust
use a3s_flow::{ExecutionState, FlowEngine, NodeRegistry};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> a3s_flow::Result<()> {
    let engine = FlowEngine::new(NodeRegistry::with_defaults());

    // Discover registered node types.
    println!("available nodes: {:?}", engine.node_types()); // ["noop"]

    let definition = json!({
        "nodes": [
            { "id": "start",   "type": "noop" },
            { "id": "process", "type": "noop" },
            { "id": "end",     "type": "noop" }
        ],
        "edges": [
            { "source": "start",   "target": "process" },
            { "source": "process", "target": "end" }
        ]
    });

    // Start: validates the DAG, spawns a background task, returns immediately.
    let id = engine.start(&definition, HashMap::new()).await?;

    // Pause at the next wave boundary.
    engine.pause(id).await?;

    // Resume.
    engine.resume(id).await?;

    // Query state at any time.
    match engine.state(id).await? {
        ExecutionState::Completed(result) => {
            println!("done — outputs: {:#?}", result.outputs);
        }
        ExecutionState::Running => println!("still running"),
        other => println!("state: {}", other.as_str()),
    }

    Ok(())
}
```

### Via `FlowRunner` (direct, no lifecycle control)

Use `FlowRunner` when you need a simple fire-and-forget execution with no
pause / resume / terminate support.

```rust
use a3s_flow::{DagGraph, FlowRunner, NodeRegistry};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> a3s_flow::Result<()> {
    let dag    = DagGraph::from_json(&json!({ "nodes": [{ "id": "a", "type": "noop" }], "edges": [] }))?;
    let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
    let result = runner.run(HashMap::new()).await?;
    println!("{:#?}", result.outputs);
    Ok(())
}
```

---

## Adding a custom node

Implement `Node` and register it before creating the engine or runner:

```rust
use a3s_flow::{ExecContext, FlowEngine, FlowError, Node, NodeRegistry};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

struct HttpGetNode;

#[async_trait]
impl Node for HttpGetNode {
    fn node_type(&self) -> &str { "http_get" }

    async fn execute(&self, ctx: ExecContext) -> Result<Value, FlowError> {
        let url = ctx.data["url"]
            .as_str()
            .ok_or_else(|| FlowError::InvalidDefinition("missing data.url".into()))?;

        let body = reqwest::get(url)
            .await
            .map_err(|e| FlowError::Internal(e.to_string()))?
            .text()
            .await
            .map_err(|e| FlowError::Internal(e.to_string()))?;

        Ok(serde_json::json!({ "body": body }))
    }
}

let mut registry = NodeRegistry::with_defaults();
registry.register(Arc::new(HttpGetNode));

let engine = FlowEngine::new(registry);
```

---

## Built-in nodes (Dify-compatible)

| Type string | Dify equivalent | Key config fields | Output |
|-------------|----------------|-------------------|--------|
| `"noop"` | — | — | Merged upstream inputs |
| `"start"` | Start | `inputs[]` (name, type, default) | Resolved input variables |
| `"end"` | End | `outputs` (name → JSON pointer) | Named output values |
| `"http-request"` | HTTP Request | `url`✱, `method`, `headers`, `body` | `{ status, ok, body }` |
| `"if-else"` | IF/ELSE | `cases[]` (id, conditions, logical_operator) | `{ branch: "case_id"\|"else" }` |
| `"template-transform"` | Template | `template` (Jinja2 string)✱ | `{ output: string }` |
| `"variable-aggregator"` | Variable Aggregator | `inputs` (ordered key list, optional) | `{ output: first_non_null }` |
| `"code"` | Code | `language` (`"rhai"`), `code`✱ | Map or `{ output: value }` |
| `"iteration"` | Iteration | `input_selector`✱, `output_selector`✱, `flow`✱, `mode` (`"parallel"`/`"sequential"`) | `{ output: [value, ...] }` |
| `"llm"` | LLM | `model`✱, `user_prompt`✱, `system_prompt`, `api_base`, `api_key`, `temperature`, `max_tokens` | `{ text, model, finish_reason, usage }` |
| `"question-classifier"` | Question Classifier | `model`✱, `question`✱, `classes[]`✱ (id, name, description), `api_base`, `api_key` | `{ branch: "class_id" }` |
| `"assign"` | Variable Assigner | `assigns`✱ (name → Jinja2 template or literal value) | Assigned key-value map |
| `"parameter-extractor"` | Parameter Extractor | `model`✱, `query`✱, `parameters[]`✱ (name, type, description, required), `api_base`, `api_key` | Extracted JSON object |
| `"loop"` | Loop | `flow`✱, `output_selector`✱, `max_iterations` (default 10), `break_condition` | `{ output, iterations }` |
| `"list-operator"` | List Operator | `input_selector`✱, `filter`, `sort_by`, `sort_order`, `deduplicate_by`, `limit` | `{ output: [...] }` |
| `"mcp"` | MCP Tool Call | `server_url`✱, `tool_name`✱, `transport` (default `"sse"`), `arguments` | `{ text, content, is_error }` |

✱ = required field

### `"if-else"` — conditional routing

```json
{
  "id": "route",
  "type": "if-else",
  "data": {
    "cases": [
      {
        "id": "is_ok",
        "logical_operator": "and",
        "conditions": [
          { "from": "fetch", "path": "status", "op": "eq", "value": 200 }
        ]
      },
      {
        "id": "is_error",
        "conditions": [
          { "from": "fetch", "path": "status", "op": "gte", "value": 500 }
        ]
      }
    ]
  }
}
```

Output: `{ "branch": "is_ok" }` | `{ "branch": "is_error" }` | `{ "branch": "else" }`

Downstream nodes use `run_if` inside `data` to select their path:
```json
{ "data": { "run_if": { "from": "route", "path": "branch", "op": "eq", "value": "is_ok" } } }
```

### `"template-transform"` — Jinja2 rendering

Upstream node outputs are available by node ID; global variables by their key:

```json
{ "template": "Hello {{ user.name }}! Status: {{ fetch.status }}" }
```

### `"code"` — Rhai scripting

`inputs` (upstream outputs by node ID) and `variables` are injected into scope:

```rhai
// Returns an object map → becomes output directly
#{
  ok:    inputs.fetch.status == 200,
  count: inputs.fetch.body.items.len()
}
```

### `"iteration"` — loop over an array

Runs an inline sub-flow for every element of an input array. Each iteration receives two extra flow variables: `variables.item` (the current element) and `variables.index` (0-based position). Iterations execute concurrently; results are returned in the original array order.

```json
{
  "id": "process_all",
  "type": "iteration",
  "data": {
    "input_selector":  "fetch.body.items",
    "output_selector": "summarize.output",
    "flow": {
      "nodes": [
        {
          "id": "summarize",
          "type": "code",
          "data": {
            "language": "rhai",
            "code": "#{ output: variables.item.name + \" processed\" }"
          }
        }
      ],
      "edges": []
    }
  }
}
```

Output: `{ "output": ["item0 processed", "item1 processed", ...] }`

---

## Reliability features (Phase 3)

### Per-node retry policy

Add a `retry` object to any node's `data` field to enable automatic retries on failure:

```json
{
  "id": "fetch",
  "type": "http-request",
  "data": {
    "url": "https://api.example.com/items",
    "retry": { "max_attempts": 3, "backoff_ms": 500 }
  }
}
```

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `max_attempts` | `u32` | ✓ | Total attempts including the first (minimum: 1) |
| `backoff_ms` | `u64` | | Base delay between retries in ms. Each retry waits `base * 2^(n-1)` (capped at `base * 64`). Default: `0` (no delay) |

All errors (including transient network failures) count as an attempt. The last error is propagated as `FlowError::NodeFailed` if all attempts are exhausted.

### Per-node timeout

Add `timeout_ms` to any node's `data` field to limit its execution time:

```json
{
  "id": "fetch",
  "type": "http-request",
  "data": {
    "url": "https://api.example.com/items",
    "timeout_ms": 5000
  }
}
```

If the node does not complete within the specified duration, the attempt fails with `"timed out after Xms"`. Combine with `retry` to retry timed-out nodes.

### Partial execution resume

`FlowRunner::resume_from` continues a flow from a prior (partial or complete) `FlowResult`, skipping any nodes that already have recorded outputs:

```rust
// First run (possibly interrupted or partial).
let partial: FlowResult = runner.run(variables.clone()).await?;

// Resume: nodes listed in partial.completed_nodes are not re-executed.
let full: FlowResult = runner.resume_from(&partial, variables).await?;
```

`FlowResult` now exposes two additional fields to support this:

| Field | Type | Description |
|-------|------|-------------|
| `completed_nodes` | `HashSet<String>` | All nodes that finished (including skipped ones) |
| `skipped_nodes` | `HashSet<String>` | Nodes whose `run_if` guard was false |

Use `skipped_nodes` to distinguish a node that genuinely produced `null` from one that was conditionally skipped.

---

## Extension points

| Type / Trait | Purpose | Default |
|---|---|---|
| `Node` | Custom node execution logic | 7 built-in types |
| `NodeRegistry` | Maps type strings to `Arc<dyn Node>` | Ships with all built-ins |
| `Condition` / `Case` | Shared condition type for `run_if` + `"if-else"` | — |
| `ExecContext` | Per-node runtime data (data + inputs + variables) | — |
| `FlowEngine` | Lifecycle orchestrator — owns registry + execution map | — |
| `ExecutionStore` | Persist execution history and replay | `MemoryExecutionStore` |
| `FlowStore` | Load and save named flow definitions | `MemoryFlowStore` |
| `EventEmitter` | Node and flow lifecycle events | `NoopEventEmitter` |
| `FlowEvent` | Cloneable event enum for broadcast streaming | — |
| `StartNode` | Dify-compatible input declaration + defaults | built-in |
| `EndNode` | Output collection via JSON pointer paths | built-in |

---

## Persistence & observability (Phase 4)

### ExecutionStore — persist completed results

`FlowEngine` automatically saves every successfully completed `FlowResult` to an `ExecutionStore` when one is configured:

```rust
use a3s_flow::{FlowEngine, MemoryExecutionStore, NodeRegistry};
use std::sync::Arc;

let store = Arc::new(MemoryExecutionStore::new());
let engine = FlowEngine::new(NodeRegistry::with_defaults())
    .with_execution_store(Arc::clone(&store) as Arc<dyn a3s_flow::ExecutionStore>);

let id = engine.start(&definition, variables).await?;
// After the flow completes, the result is available in the store.
let result = store.load(id).await?.unwrap();
```

Implement `ExecutionStore` to persist to a database, S3, or any backend:

```rust
#[async_trait]
impl ExecutionStore for MyStore {
    async fn save(&self, result: &FlowResult) -> Result<()> { /* ... */ }
    async fn load(&self, id: Uuid) -> Result<Option<FlowResult>> { /* ... */ }
    async fn list(&self) -> Result<Vec<Uuid>> { /* ... */ }
    async fn delete(&self, id: Uuid) -> Result<()> { /* ... */ }
}
```

### FlowStore — named flow definition storage

`FlowStore` is a stand-alone utility for storing and retrieving named flow definitions:

```rust
use a3s_flow::{MemoryFlowStore, FlowStore, FlowEngine, NodeRegistry};
use std::sync::Arc;

let flow_store = MemoryFlowStore::new();
flow_store.save("daily-report", &definition).await?;

// Later: load by name and start.
let engine = FlowEngine::new(NodeRegistry::with_defaults());
if let Some(def) = flow_store.load("daily-report").await? {
    engine.start(&def, variables).await?;
}
```

### EventEmitter — lifecycle event hooks

Implement `EventEmitter` to receive flow and node lifecycle events. Register it on `FlowEngine` or `FlowRunner`:

```rust
use a3s_flow::{EventEmitter, FlowEngine, FlowResult, NodeRegistry};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

struct MyEmitter;

#[async_trait]
impl EventEmitter for MyEmitter {
    async fn on_flow_started(&self, id: Uuid) { /* ... */ }
    async fn on_flow_completed(&self, id: Uuid, result: &FlowResult) { /* ... */ }
    async fn on_flow_failed(&self, id: Uuid, reason: &str) { /* ... */ }
    async fn on_flow_terminated(&self, id: Uuid) { /* ... */ }
    async fn on_node_started(&self, id: Uuid, node_id: &str, node_type: &str) { /* ... */ }
    async fn on_node_completed(&self, id: Uuid, node_id: &str, output: &Value) { /* ... */ }
    async fn on_node_skipped(&self, id: Uuid, node_id: &str) { /* ... */ }
    async fn on_node_failed(&self, id: Uuid, node_id: &str, reason: &str) { /* ... */ }
}

let engine = FlowEngine::new(NodeRegistry::with_defaults())
    .with_event_emitter(Arc::new(MyEmitter) as Arc<dyn EventEmitter>);
```

### OpenTelemetry-compatible tracing spans

Every node execution is wrapped in a `tracing::info_span!` with `node_id`, `node_type`, and `execution_id` as structured fields:

```
node.execute{node_id="fetch_data", node_type="http-request", execution_id="..."}
```

Attach a `tracing-opentelemetry` subscriber to export these spans to any OTel-compatible backend (Jaeger, OTLP, etc.). No additional configuration is needed in `a3s-flow`.

---

## Error handling

All errors are variants of `FlowError`:

| Variant | When |
|---|---|
| `InvalidDefinition(String)` | Bad JSON shape, empty flow, duplicate node ID, unknown node type |
| `CyclicGraph` | The DAG contains a cycle |
| `UnknownNode(String)` | An edge `source`/`target` references a non-existent node ID |
| `NodeFailed { node_id, execution_id, reason }` | A node's `execute` returned an error |
| `Terminated` | The execution was stopped by `terminate()` |
| `ExecutionNotFound(Uuid)` | No execution exists for the given ID |
| `InvalidTransition { action, from }` | State transition not allowed (e.g. pause a completed flow) |
| `Json(serde_json::Error)` | JSON deserialization failure |
| `Internal(String)` | Unexpected engine-level error |

---

## Roadmap

**Phase 1 — Core engine** ✅

- [x] JSON DAG parsing and validation
- [x] Cycle detection (petgraph topological sort)
- [x] Wave-based concurrent execution (Tokio `JoinSet`)
- [x] Pluggable `Node` trait and `NodeRegistry`
- [x] Built-in `noop` node
- [x] `ExecContext`: config + upstream inputs + global variables
- [x] `FlowResult` with per-node outputs and execution UUID
- [x] `FlowEngine`: start, pause, resume, terminate, state query
- [x] `ExecutionState` machine: Running → Paused / Completed / Failed / Terminated
- [x] Cancel-aware `JoinSet` drain via `tokio::select!` (fast termination mid-wave)

**Phase 2 — Built-in nodes (Dify-compatible)** ✅

- [x] `"http-request"` — HTTP request node (GET / POST / PUT / DELETE / PATCH)
- [x] `"if-else"` — multi-case conditional routing, output `{ "branch": "case_id" | "else" }`
- [x] `"template-transform"` — Jinja2 string rendering (minijinja)
- [x] `"variable-aggregator"` — first non-null fan-in after branch merge
- [x] `"code"` — sandboxed Rhai script execution
- [x] `run_if` — per-node guard condition with automatic skip propagation
- [x] `Case` + `LogicalOp` — multi-condition AND/OR within a branch
- [x] `"iteration"` — concurrent sub-flow loop over an array; `item` + `index` injected as variables; results collected in original order

**Phase 3 — Reliability** ✅

- [x] Per-node retry policy (max attempts, exponential backoff) — `data["retry"]`
- [x] Per-node timeout — `data["timeout_ms"]`
- [x] Partial execution resume — `FlowRunner::resume_from(&prior, vars)` skips already-completed nodes

**Phase 4 — Persistence & observability** ✅

- [x] `ExecutionStore` trait + `MemoryExecutionStore` — persist execution history; auto-saved by `FlowEngine`
- [x] `FlowStore` trait + `MemoryFlowStore` — load / save named flow definitions
- [x] `EventEmitter` trait + `NoopEventEmitter` — node and flow lifecycle events (integrates with `a3s-event`)
- [x] OpenTelemetry-compatible `info_span!("node.execute", node_id, node_type, execution_id)` per node

**Phase 5 — Streaming & sub-flow composition** ✅

- [x] `FlowEvent` enum — `Clone`-able snapshot of every lifecycle event (8 variants covering flow + node start/complete/skip/fail)
- [x] `FlowEngine::start_streaming` — returns `(Uuid, broadcast::Receiver<FlowEvent>)`; receiver is created before spawn so zero events are lost; composable with any existing `EventEmitter`
- [x] `"sub-flow"` built-in node — executes a named flow inline as a single step; inherits parent registry and variables; `data["variables"]` extends/overrides them; output is the sub-flow's per-node outputs map
- [x] `flow_store` propagation — `FlowRunner` and `ExecContext` now carry the engine's `FlowStore`, enabling `"sub-flow"` and future nodes to load named definitions at execution time

**Phase 6 — Error recovery & concurrency controls** ✅

- [x] `continue_on_error` per-node flag — a failed node produces `{"__error__": "reason"}` as output instead of halting the flow; downstream nodes run normally; `EventEmitter` still receives `on_node_completed` with the error output
- [x] `max_concurrency` on `FlowRunner` / `FlowEngine` — Tokio `Semaphore` limits the number of nodes executing simultaneously within a wave; unlimited by default; builder-pattern API (`with_max_concurrency(n)`)
- [x] `"start"` node — Dify-compatible entry point; declares typed flow inputs with optional defaults; validates type at execution time; passes resolved variables to downstream nodes
- [x] `"end"` node — Dify-compatible output collector; gathers values from upstream nodes using JSON pointer paths (`/node_id/field`); missing paths resolve to `null`

**Phase 7 — LLM nodes** ✅

- [x] `"llm"` node — OpenAI-compatible chat completion; system and user prompts rendered as Jinja2 templates; outputs `text`, `model`, `finish_reason`, and token usage; works with any `/v1/chat/completions` endpoint (OpenAI, Ollama, LM Studio, vLLM, Together AI, Anthropic proxy, etc.)
- [x] `"question-classifier"` node — LLM-powered routing; classifies input into one of N user-defined classes; outputs `{ "branch": "class_id" }` (same shape as `"if-else"`); fallback strategy: exact match → substring match → first class

**Phase 8 — State mutation & validation** ✅

- [x] `"assign"` node — writes key-value pairs into the live flow variable scope; string values rendered as Jinja2 templates, non-string values used as-is; runner automatically merges output into `ctx.variables` between waves so downstream nodes see updated values
- [x] Sequential iteration mode — `"iteration"` node gains `data["mode"]` field: `"parallel"` (default, existing behaviour) or `"sequential"` (items processed one-at-a-time in order; `prev_output` variable injected for each step)
- [x] `FlowEngine::validate` — synchronous pre-flight check returning `Vec<ValidationIssue>`; checks: DAG structural validity, all node types registered, `run_if.from` references existing nodes; zero-cost — does not start an execution

**Phase 9 — Dify parity: parameter-extractor, loop, list-operator** ✅

- [x] `"parameter-extractor"` node — LLM-powered structured extraction from natural language; `query` rendered as Jinja2 template; `parameters[]` declares names, types, descriptions, and required flag; LLM response parsed as JSON (markdown fences stripped automatically); required parameters that cannot be found surface as errors
- [x] `"loop"` node — while-loop over inline sub-flow; runs until `break_condition` (same `Condition` schema as `run_if`) is true or `max_iterations` is reached; injects `iteration_index` and `loop_output` (previous iteration result) as variables each round; outputs `{ output, iterations }`
- [x] `"list-operator"` node — pure in-process JSON array pipeline: filter (eq/ne/gt/lt/gte/lte/contains on a dot-path field) → sort (by dot-path field, asc/desc, numerics/strings/nulls) → deduplicate (by dot-path key or full equality) → limit (first N); all operations optional; zero network calls

**Phase 10 — MCP integration** ✅

- [x] `"mcp"` node — call a tool on a Model Context Protocol server; supports SSE (2024-11-05) and Streamable HTTP (2025-03-26) transports; `arguments` rendered as Jinja2 templates; outputs `{ text, content, is_error }` where `text` is concatenated text content and `content` is the full MCP content array; enables external tool integration (document extraction, knowledge retrieval, etc.) without embedding those capabilities in the engine

---

## Streaming execution (Phase 5)

### `start_streaming` — pull-based event subscription

`FlowEngine::start_streaming` is an alternative to `start` that also returns a live `broadcast::Receiver<FlowEvent>`. Because the receiver is created **before** the execution task is spawned, the first event (`FlowStarted`) is never missed.

```rust
use a3s_flow::{FlowEngine, FlowEvent, NodeRegistry};
use serde_json::json;
use std::collections::HashMap;

let engine = FlowEngine::new(NodeRegistry::with_defaults());
let def = json!({
    "nodes": [{ "id": "a", "type": "noop" }, { "id": "b", "type": "noop" }],
    "edges": [{ "source": "a", "target": "b" }]
});

let (id, mut rx) = engine.start_streaming(&def, HashMap::new()).await?;

while let Ok(event) = rx.recv().await {
    match event {
        FlowEvent::NodeCompleted { node_id, output, .. } => {
            println!("✓ {node_id}: {output}");
        }
        FlowEvent::FlowCompleted { result, .. } => {
            println!("flow done — {} nodes", result.completed_nodes.len());
            break;
        }
        FlowEvent::FlowFailed { reason, .. } => {
            eprintln!("flow failed: {reason}");
            break;
        }
        _ => {}
    }
}
```

Multiple subscribers are supported via [`broadcast::Receiver::resubscribe`]. If a custom `EventEmitter` is also attached (via `with_event_emitter`), both receive every event.

### `FlowEvent` variants

| Variant | When |
|---------|------|
| `FlowStarted { execution_id }` | Execution begins |
| `NodeStarted { execution_id, node_id, node_type }` | Before first attempt |
| `NodeCompleted { execution_id, node_id, output }` | Node succeeded |
| `NodeSkipped { execution_id, node_id }` | `run_if` guard was false |
| `NodeFailed { execution_id, node_id, reason }` | All retries exhausted |
| `FlowCompleted { execution_id, result }` | All nodes done |
| `FlowFailed { execution_id, reason }` | A node failed and halted the flow |
| `FlowTerminated { execution_id }` | `terminate()` was called |

---

## Sub-flow composition (Phase 5)

### `"sub-flow"` node — reuse named flows as steps

The `"sub-flow"` node loads a named flow definition from the engine's `FlowStore` and executes it synchronously as part of the parent wave. The parent's node registry is shared — all custom node types are available inside the sub-flow. Variables are inherited from the parent and can be overridden per invocation.

The node output is a JSON object whose keys are the sub-flow's node IDs and values are those nodes' outputs, identical in shape to `FlowResult::outputs`.

```rust
use a3s_flow::{FlowEngine, FlowStore, MemoryFlowStore, NodeRegistry};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};

// Register the sub-flow definition.
let store = Arc::new(MemoryFlowStore::new());
let summarizer_def = json!({
    "nodes": [{ "id": "summarize", "type": "noop" }],
    "edges": []
});
store.save("summarizer", &summarizer_def).await?;

// Build the parent flow that calls the sub-flow.
let parent_def = json!({
    "nodes": [
        { "id": "fetch", "type": "noop" },
        {
            "id": "summarize",
            "type": "sub-flow",
            "data": {
                "name": "summarizer",
                "variables": { "max_tokens": 256 }
            }
        }
    ],
    "edges": [{ "source": "fetch", "target": "summarize" }]
});

let engine = FlowEngine::new(NodeRegistry::with_defaults())
    .with_flow_store(store as Arc<dyn FlowStore>);

let id = engine.start(&parent_def, HashMap::new()).await?;
```

**`data` fields for `"sub-flow"`:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | ✅ | Name of the flow definition in the `FlowStore` |
| `variables` | object | — | Extra variables merged on top of the parent's variables |

---

## Error recovery & concurrency controls (Phase 6)

### `continue_on_error` — absorb node failures

Set `data["continue_on_error"]: true` on any node to prevent its failure from halting the flow. Instead of propagating a `NodeFailed` error, the node outputs `{"__error__": "reason"}` and downstream nodes execute as normal.

```json
{
  "nodes": [
    {
      "id": "fetch",
      "type": "http-request",
      "data": {
        "url": "https://api.example.com/data",
        "continue_on_error": true
      }
    },
    {
      "id": "fallback",
      "type": "variable-aggregator",
      "data": { "inputs": ["fetch"] }
    }
  ],
  "edges": [{ "source": "fetch", "target": "fallback" }]
}
```

Downstream nodes receive `inputs["fetch"] = {"__error__": "..."}` and can branch on it via an `"if-else"` node or a `"code"` node.

### `max_concurrency` — rate-limit parallel execution

By default all nodes in a wave run simultaneously. Use `with_max_concurrency(n)` to cap this:

```rust
use a3s_flow::{FlowEngine, NodeRegistry};
use std::collections::HashMap;

let engine = FlowEngine::new(NodeRegistry::with_defaults())
    .with_max_concurrency(4);  // at most 4 nodes run at once

let id = engine.start(&definition, HashMap::new()).await?;
```

This is implemented with a Tokio `Semaphore` acquired *inside* each spawned task — all tasks are spawned immediately (preserving correct wave ordering) but at most `n` proceed concurrently.

### `"start"` node — declare and validate flow inputs

```json
{
  "id": "start",
  "type": "start",
  "data": {
    "inputs": [
      { "name": "query",      "type": "string" },
      { "name": "max_tokens", "type": "number", "default": 256 },
      { "name": "verbose",    "type": "bool",   "default": false }
    ]
  }
}
```

The `"start"` node resolves each declared input from the flow's `variables` map (applying defaults for absent ones) and validates that types match. Its output is `{ "query": "...", "max_tokens": 256, "verbose": false }`. Downstream nodes access these via `ctx.inputs["start"]["query"]`.

**`inputs[n]` fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | ✅ | Variable name |
| `type` | `"string"` \| `"number"` \| `"bool"` \| `"object"` \| `"array"` | — | Expected type (validated at runtime) |
| `default` | any | — | Fallback when the variable is absent; omitting it makes the input required |

### `"end"` node — collect flow outputs

```json
{
  "id": "end",
  "type": "end",
  "data": {
    "outputs": {
      "answer":       "/llm/text",
      "total_tokens": "/llm/usage/total_tokens",
      "raw":          "/transform"
    }
  }
}
```

Paths are JSON pointers resolved against the set of upstream node outputs. `/llm/text` resolves to `ctx.inputs["llm"]["text"]`. Missing paths resolve to `null`. Omitting `outputs` returns all upstream inputs as-is.

**Complete start → process → end flow:**

```rust
use a3s_flow::{FlowEngine, NodeRegistry};
use serde_json::json;
use std::collections::HashMap;

let def = json!({
    "nodes": [
        {
            "id": "start",
            "type": "start",
            "data": { "inputs": [{ "name": "query", "type": "string" }] }
        },
        { "id": "process", "type": "noop" },
        {
            "id": "end",
            "type": "end",
            "data": { "outputs": { "result": "/process" } }
        }
    ],
    "edges": [
        { "source": "start",   "target": "process" },
        { "source": "process", "target": "end" }
    ]
});

let engine = FlowEngine::new(NodeRegistry::with_defaults());
let mut vars = HashMap::new();
vars.insert("query".into(), json!("What is 2+2?"));
let id = engine.start(&def, vars).await?;
```

---

## LLM nodes (Phase 7)

### `"llm"` — OpenAI-compatible chat completion

Renders system and user prompts as Jinja2 templates, calls any `/v1/chat/completions` endpoint, and returns the assistant's reply with token-usage statistics.

**Config schema:**

| Field | Type | Required | Default | Description |
|-------|------|:--------:|---------|-------------|
| `model` | string | ✅ | — | Model identifier |
| `user_prompt` | string | ✅ | — | User turn — rendered as Jinja2 template |
| `system_prompt` | string | — | _(none)_ | System turn — rendered as Jinja2 template |
| `api_base` | string | — | `https://api.openai.com/v1` | Base URL (no trailing slash) |
| `api_key` | string | — | `""` | Bearer token; may be empty for local models |
| `temperature` | number | — | `0.7` | Sampling temperature `[0, 2]` |
| `max_tokens` | integer | — | _(none)_ | Max completion tokens |

**Template context:** Both prompts are Jinja2 templates. The rendering context contains all global flow `variables` (by key) and all upstream node outputs (by node ID). Upstream inputs shadow variables with the same key.

**Output schema:**

```json
{
  "text":          "The answer is 42.",
  "model":         "gpt-4o-mini",
  "finish_reason": "stop",
  "usage": {
    "prompt_tokens":     15,
    "completion_tokens":  8,
    "total_tokens":      23
  }
}
```

**Example — question answering with a `"start"` node:**

```json
{
  "nodes": [
    {
      "id": "start",
      "type": "start",
      "data": { "inputs": [{ "name": "query", "type": "string" }] }
    },
    {
      "id": "llm",
      "type": "llm",
      "data": {
        "model":         "gpt-4o-mini",
        "api_key":       "sk-...",
        "system_prompt": "You are a helpful assistant. Answer concisely.",
        "user_prompt":   "{{ query }}"
      }
    },
    {
      "id": "end",
      "type": "end",
      "data": { "outputs": { "answer": "/llm/text" } }
    }
  ],
  "edges": [
    { "source": "start", "target": "llm" },
    { "source": "llm",   "target": "end" }
  ]
}
```

**Example — local Ollama model:**

```json
{
  "id": "llm",
  "type": "llm",
  "data": {
    "model":    "llama3.2",
    "api_base": "http://localhost:11434/v1",
    "api_key":  "",
    "user_prompt": "Summarise this text: {{ fetch.body }}"
  }
}
```

---

### `"question-classifier"` — LLM-powered routing

Classifies an input question into one of several user-defined classes using an LLM, then outputs `{ "branch": "class_id" }` — the same shape as `"if-else"`, so `run_if` conditions work identically.

**Config schema:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `model` | string | ✅ | Model identifier |
| `question` | string | ✅ | Question to classify — rendered as Jinja2 template |
| `classes` | array | ✅ | At least 2 classes; each requires `id` and `name` |
| `classes[].id` | string | ✅ | Unique identifier returned as `branch` |
| `classes[].name` | string | ✅ | Human-readable class name |
| `classes[].description` | string | — | Optional extra guidance for the LLM |
| `api_base`, `api_key`, `temperature`, `max_tokens` | | — | Same as `"llm"` node |

**Output schema:**

```json
{ "branch": "technical" }
```

If the LLM response does not match any declared class ID (case-insensitive), the node falls back to the first class.

**Example — three-way routing:**

```json
{
  "nodes": [
    {
      "id": "start",
      "type": "start",
      "data": { "inputs": [{ "name": "user_input", "type": "string" }] }
    },
    {
      "id": "classifier",
      "type": "question-classifier",
      "data": {
        "model":    "gpt-4o-mini",
        "api_key":  "sk-...",
        "question": "{{ user_input }}",
        "classes": [
          { "id": "technical", "name": "Technical question",
            "description": "Questions about code, APIs, or system behaviour" },
          { "id": "billing",   "name": "Billing question" },
          { "id": "general",   "name": "General question" }
        ]
      }
    },
    {
      "id": "tech-answer",
      "type": "llm",
      "data": {
        "model": "gpt-4o-mini", "api_key": "sk-...",
        "user_prompt": "Answer this technical question: {{ user_input }}"
      },
      "run_if": { "from": "classifier", "path": "branch", "op": "eq", "value": "technical" }
    },
    {
      "id": "billing-answer",
      "type": "llm",
      "data": {
        "model": "gpt-4o-mini", "api_key": "sk-...",
        "user_prompt": "Help with this billing question: {{ user_input }}"
      },
      "run_if": { "from": "classifier", "path": "branch", "op": "eq", "value": "billing" }
    },
    {
      "id": "general-answer",
      "type": "llm",
      "data": {
        "model": "gpt-4o-mini", "api_key": "sk-...",
        "user_prompt": "Answer this question: {{ user_input }}"
      },
      "run_if": { "from": "classifier", "path": "branch", "op": "eq", "value": "general" }
    }
  ],
  "edges": [
    { "source": "start",      "target": "classifier" },
    { "source": "classifier", "target": "tech-answer" },
    { "source": "classifier", "target": "billing-answer" },
    { "source": "classifier", "target": "general-answer" }
  ]
}
```

---

## Phase 8 — State mutation & validation

### `"assign"` node — write to the variable scope

The `"assign"` node is the only built-in node that mutates the flow's live variable map. Its output is merged into `ctx.variables` immediately after the wave completes, so every downstream node sees the new values without any special wiring.

**Config schema:**

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `assigns` | object | ✅ | Map of variable names to Jinja2 templates (strings) or literal JSON values |

**Template context:** same as the `"llm"` node — all global variables + upstream node outputs (inputs shadow same-name variables).

**Output:** the resolved assignment map (identical to what is merged into `ctx.variables`).

**Example — initialise counters:**

```json
{
  "id": "init",
  "type": "assign",
  "data": {
    "assigns": {
      "attempt":  1,
      "user_msg": "{{ start.message }}",
      "tags":     ["default"]
    }
  }
}
```

**Example — update variable mid-flow:**

```json
{
  "nodes": [
    { "id": "start",  "type": "start",  "data": { "inputs": [{ "name": "name", "type": "string" }] } },
    { "id": "greet",  "type": "assign", "data": { "assigns": { "greeting": "Hello, {{ name }}!" } } },
    { "id": "render", "type": "template-transform", "data": { "template": "{{ greeting }}" } }
  ],
  "edges": [
    { "source": "start",  "target": "greet" },
    { "source": "greet",  "target": "render" }
  ]
}
```

**Behaviour notes:**
- If a wave contains multiple `"assign"` nodes, all their outputs are merged after the wave (order between concurrent assigns within one wave is not guaranteed — avoid conflicting keys in the same wave).
- If an `"assign"` node fails and has `continue_on_error: true`, the error output (`{ "__error__": "..." }`) is **not** merged into variables.
- Skipped `"assign"` nodes (via `run_if`) do not affect the variable scope.

---

### Sequential iteration mode

The `"iteration"` node now supports a `"mode"` field:

| Value | Behaviour |
|-------|-----------|
| `"parallel"` | *(default)* All items run concurrently via Tokio tasks |
| `"sequential"` | Items run one-at-a-time in order; each item receives the previous item's collected output as the `prev_output` variable |

**Additional variable injected in sequential mode:**

| Variable | Value |
|----------|-------|
| `prev_output` | The previous iteration's `output_selector` result (`null` for the first item) |

**Example — sequential summarisation pipeline:**

```json
{
  "id": "pipeline",
  "type": "iteration",
  "data": {
    "input_selector":  "fetch.body.chapters",
    "output_selector": "summarize.text",
    "mode":            "sequential",
    "flow": {
      "nodes": [
        {
          "id": "summarize",
          "type": "llm",
          "data": {
            "model":       "gpt-4o-mini",
            "api_key":     "sk-...",
            "user_prompt": "Previous summary: {{ prev_output }}\n\nSummarise this chapter: {{ item }}"
          }
        }
      ],
      "edges": []
    }
  }
}
```

---

### `FlowEngine::validate` — pre-flight validation

Validate a flow definition before executing it. Returns a `Vec<ValidationIssue>` — an empty list means the flow is structurally valid.

```rust
use a3s_flow::{FlowEngine, NodeRegistry, ValidationIssue};
use serde_json::json;

let engine = FlowEngine::new(NodeRegistry::with_defaults());

let issues = engine.validate(&json!({
    "nodes": [
        { "id": "a", "type": "noop" },
        { "id": "b", "type": "does-not-exist" }
    ],
    "edges": []
}));

for issue in &issues {
    println!("{issue}");  // "node 'b': unknown node type 'does-not-exist'"
}
assert_eq!(issues.len(), 1);
```

**Checks performed:**

| Check | Error location |
|-------|----------------|
| DAG parse failure (cycle, unknown edge ref, duplicate ID) | `node_id: None` |
| Unregistered node type | `node_id: Some("node_id")` |
| `run_if.from` references a node not in the graph | `node_id: Some("node_id")` |

`ValidationIssue` implements `Display` for human-readable messages:
- Flow-level: `"cyclic dependency detected"`
- Node-level: `"node 'b': unknown node type 'does-not-exist'"`

---

## License

MIT — see [LICENSE](./LICENSE).

---

<p align="center">
  Part of the <a href="https://github.com/A3S-Lab/a3s">A3S</a> ecosystem · Built by <a href="https://github.com/A3S-Lab">A3S Lab</a>
</p>
