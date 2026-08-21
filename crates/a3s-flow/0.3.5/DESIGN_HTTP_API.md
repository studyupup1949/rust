# a3s-flow HTTP API Design

This document defines a REST-style HTTP surface for exposing `a3s-flow`
through an Axum server.

Goals:

- preserve the progressive API layering already present in `FlowEngine`
- keep transport payloads close to the Rust API names
- make dynamic node type management explicit and safe
- support both editor-style clients and hosted workflow platforms
- align the public route shape with the existing `a3s-box` API style

## Box-aligned design principles

`a3s-box` exposes its UI/backend API as resource families under `/api/box/*`,
for example:

- `/api/box/containers`
- `/api/box/images`
- `/api/box/networks`
- `/api/box/volumes`
- `/api/box/info`
- action routes such as `/api/box/stop/:id` and `/api/box/pull/:ref`

`a3s-flow` should follow the same transport philosophy:

- use `/api/flow/*`, not a separate RPC-like namespace
- group routes by resource family, not by abstract capability layer
- use collection routes for snapshots and listings
- use small action routes for lifecycle transitions
- keep route names short and UI-friendly

The progressive aspect should exist in the resource model itself:

1. catalog and validation
2. execution creation and inspection
3. event observation
4. runtime control
5. shared context mutation
6. definition-backed execution

## Base path

Recommended base path:

```text
/api/flow
```

Examples below assume that prefix.

## Error model

All non-2xx responses should use a stable JSON envelope:

```json
{
  "error": {
    "code": "protected_node_type",
    "message": "node type is protected and cannot be removed: noop"
  }
}
```

Suggested error code mapping:

| FlowError | HTTP | `error.code` |
|----------|------|--------------|
| `InvalidDefinition` | `400` | `invalid_definition` |
| `UnknownNode` | `400` | `unknown_node` |
| `ExecutionNotFound` | `404` | `execution_not_found` |
| `FlowNotFound` | `404` | `flow_not_found` |
| `InvalidTransition` | `409` | `invalid_transition` |
| `ProtectedNodeType` | `409` | `protected_node_type` |
| `Internal` | `500` | `internal` |
| `Terminated` | `409` | `terminated` |

## Common response shapes

Execution state:

```json
{
  "execution_id": "e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905",
  "state": "running"
}
```

Terminal execution state:

```json
{
  "execution_id": "e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905",
  "state": "completed",
  "result": {
    "execution_id": "e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905",
    "outputs": {
      "end": {
        "answer": "ok"
      }
    }
  }
}
```

Validation response:

```json
{
  "valid": false,
  "issues": [
    {
      "node_id": "b",
      "message": "unknown node type 'does-not-exist'"
    }
  ]
}
```

## Resource families

To stay consistent with `a3s-box`, the HTTP surface should be organized into
these top-level families:

| Family | Purpose |
|--------|---------|
| `/api/flow/info` | engine-level metadata and summary |
| `/api/flow/nodes` | node catalog and runtime node type management |
| `/api/flow/validate` | pre-flight validation |
| `/api/flow/executions` | execution creation and snapshot lookup |
| `/api/flow/events` | event streaming |
| `/api/flow/context` | shared mutable execution context |
| `/api/flow/definitions` | named/stored flow entry points |

This gives `a3s-flow` the same “tabbed resource” feel as `a3s-box`.

## L0 Discovery

### `GET /api/flow/info`

Returns engine-level summary information plus the capabilities document.

Response `200`:

```json
{
  "engine": "a3s-flow",
  "version": "2026-03-22",
  "progressive_disclosure": true,
  "summary": "A3S Flow exposes a discoverable catalog of workflow node capabilities.",
  "node_count": 16,
  "capabilities": {
    "version": "2026-03-22",
    "progressive_disclosure": true,
    "summary": "A3S Flow exposes a discoverable catalog of workflow node capabilities.",
    "nodes": []
  }
}
```

### `GET /api/flow/capabilities`

Returns `engine.capabilities()`.

Response `200`:

```json
{
  "version": "2026-03-22",
  "progressive_disclosure": true,
  "summary": "A3S Flow exposes a discoverable catalog of workflow node capabilities.",
  "nodes": []
}
```

### `GET /api/flow/node-types`

Returns a light-weight list of node type strings.

Response `200`:

```json
{
  "node_types": ["assign", "http-request", "llm", "noop"]
}
```

### `GET /api/flow/nodes`

Returns `engine.node_descriptors()`.

Response `200`:

```json
{
  "nodes": [
    {
      "node_type": "http-request",
      "display_name": "HTTP Request",
      "category": "integration",
      "summary": "Calls external HTTP APIs with configurable method, headers, and body.",
      "default_data": {
        "method": "GET",
        "url": "",
        "headers": {}
      },
      "fields": [
        {
          "key": "method",
          "kind": "string",
          "required": true,
          "description": "HTTP method."
        }
      ]
    }
  ]
}
```

## L0.5 Runtime node type management

These endpoints control the engine's runtime node registry. They only affect
future validations and executions.

Built-in node types are protected from deletion.

### `POST /api/flow/node-types`

Registers or replaces a custom node type.

Because a `dyn Node` cannot be transported over HTTP directly, this endpoint is
only suitable when the server supports a fixed set of server-side factories or
plugins. The request identifies which server-known implementation to bind.

Request:

```json
{
  "factory": "slow-test-node",
  "descriptor": {
    "display_name": "Slow Node",
    "category": "testing",
    "summary": "Sleeps briefly during tests.",
    "default_data": {
      "delay_ms": 10
    },
    "fields": [
      {
        "key": "delay_ms",
        "kind": "number",
        "required": false,
        "description": "Sleep duration in milliseconds."
      }
    ]
  }
}
```

Response `201`:

```json
{
  "node_type": "slow",
  "registered": true,
  "replaced": false
}
```

Notes:

- `factory` is a transport-facing identifier resolved by the server
- the server decides the actual Rust `Node` implementation and final `node_type`
- if `descriptor` is omitted, the server may use the node's built-in metadata

### `DELETE /api/flow/node-type/:node_type`

Deletes a runtime-registered node type.

Response `200`:

```json
{
  "node_type": "slow",
  "removed": true
}
```

Response `409` when attempting to delete a built-in type:

```json
{
  "error": {
    "code": "protected_node_type",
    "message": "node type is protected and cannot be removed: noop"
  }
}
```

Response `404` when the type does not exist:

```json
{
  "error": {
    "code": "node_type_not_found",
    "message": "node type not found: slow"
  }
}
```

Recommendation:

- prefer `404` instead of `200 { removed: false }` for HTTP clients
- keep the Rust API permissive, but make the HTTP contract explicit

This follows the same singular-resource delete style that `a3s-box` uses with
routes like `/api/box/container/:id` and `/api/box/image/:ref`.

## L1 Pre-flight

### `POST /api/flow/validate`

Request:

```json
{
  "definition": {
    "nodes": [
      { "id": "a", "type": "noop" }
    ],
    "edges": []
  }
}
```

Response `200`:

```json
{
  "valid": true,
  "issues": []
}
```

## L2 Execution

### `POST /api/flow/executions`

Starts an inline flow definition.

Request:

```json
{
  "definition": {
    "nodes": [
      { "id": "a", "type": "noop" }
    ],
    "edges": []
  },
  "variables": {
    "user_id": "u_123"
  }
}
```

Response `202`:

```json
{
  "execution_id": "e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905",
  "state": "running"
}
```

### `GET /api/flow/execution/:id`

Returns the latest execution snapshot.

Response `200`:

```json
{
  "execution_id": "e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905",
  "state": "paused"
}
```

For box-style consistency, the collection route is plural and the single-item
snapshot route is singular.

## L3 Streaming

### `GET /api/flow/events/:id`

Streams `FlowEvent` values.

Recommended transport:

- Server-Sent Events for browser and CLI clients
- WebSocket optional if bidirectional interaction is needed later

SSE event examples:

```text
event: flow.started
data: {"execution_id":"e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905"}
```

```text
event: node.completed
data: {"execution_id":"e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905","node_id":"a","output":{}}
```

```text
event: flow.completed
data: {"execution_id":"e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905","result":{"outputs":{"a":{}}}}
```

This keeps events as their own resource family instead of nesting them under
execution resources. That matches the flat, UI-oriented style used by the
existing `a3s-box` API.

## L4 Runtime control

### `POST /api/flow/pause/:id`

Response `200`:

```json
{
  "execution_id": "e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905",
  "state": "paused"
}
```

### `POST /api/flow/resume/:id`

Response `200`:

```json
{
  "execution_id": "e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905",
  "state": "running"
}
```

### `POST /api/flow/terminate/:id`

Response `202`:

```json
{
  "execution_id": "e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905",
  "state": "terminating"
}
```

Action routes are intentionally short and command-like here because that is how
`a3s-box` exposes lifecycle operations such as `/api/box/stop/:id`.

## L5 Shared context

### `GET /api/flow/context/:id`

Response `200`:

```json
{
  "context": {
    "approval": "granted"
  }
}
```

### `PUT /api/flow/context/:id/:key`

Request:

```json
{
  "value": "granted"
}
```

Response `200`:

```json
{
  "key": "approval",
  "updated": true
}
```

### `DELETE /api/flow/context/:id/:key`

Response `200`:

```json
{
  "key": "approval",
  "removed": true
}
```

## L6 Named flows

### `POST /api/flow/run/:name`

Request:

```json
{
  "variables": {
    "topic": "ai infra"
  }
}
```

Response `202`:

```json
{
  "execution_id": "e6b7e7cf-3534-4d1b-b0d1-4e2c2788f905",
  "state": "running",
  "flow_name": "daily-briefing"
}
```

This mirrors `a3s-box` action naming more closely than
`/definitions/:name/executions`.

## Handler mapping

Suggested internal handler-to-engine mapping:

| HTTP handler | FlowEngine call |
|-------------|-----------------|
| `GET /info` | `capabilities()` + local summary |
| `GET /capabilities` | `capabilities()` |
| `GET /node-types` | `node_types()` |
| `GET /nodes` | `node_descriptors()` |
| `POST /node-types` | `register_node_type[_with_descriptor]()` via server plugin registry |
| `DELETE /node-type/:node_type` | `unregister_node_type()` |
| `POST /validate` | `validate()` |
| `POST /executions` | `start()` |
| `GET /execution/:id` | `state()` |
| `GET /events/:id` | `start_streaming()` or server-side subscription bridge |
| `POST /pause/:id` | `pause()` |
| `POST /resume/:id` | `resume()` |
| `POST /terminate/:id` | `terminate()` |
| `GET /context/:id` | `get_context()` |
| `PUT /context/:id/:key` | `set_context_entry()` |
| `DELETE /context/:id/:key` | `delete_context_entry()` |
| `POST /run/:name` | `start_named()` |

## Open questions

1. Runtime registration over HTTP requires a server-side plugin/factory registry because arbitrary Rust `Node` trait objects cannot be uploaded directly as JSON.
2. Event streaming for an existing execution may require the server to keep a live subscription registry if clients attach after `start()`.
3. If named-flow storage becomes mutable over HTTP later, that should likely be a separate resource family:

```text
/v1/flow/definitions
```

4. Authentication and multi-tenant isolation should be applied at the router layer, not embedded into flow payloads.
