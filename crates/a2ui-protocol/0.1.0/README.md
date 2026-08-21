# a2ui-protocol

> A2UI protocol in Rust. How agents talk to interfaces.

## What This Does

This crate defines the **Agent-to-UI wire protocol** — binary packets for sending rendering-agnostic UI state between processes:

- **Version headers** — protocol versioning for forward compatibility
- **Type discriminators** — packet type identification (state update, render request, control action, heartbeat)
- **Length-prefixed payloads** — bincode-serialized data with explicit size
- **CRC32 integrity checks** — IEEE polynomial with compile-time lookup table

One packet format, many payloads:

```
┌──────────┬──────┬───────────┬───────────────┬──────────┐
│ version  │ type │ len (u32) │   payload     │  CRC32   │
│  (1 byte)│(1 B) │  (4 LE)   │   (N bytes)   │ (4 LE)   │
└──────────┴──────┴───────────┴───────────────┴──────────┘
```

## Install

```bash
cargo add a2ui-protocol
```

## Quick Start

```rust
use a2ui_protocol::*;

// Build state
let state = sample_ui_state();

// Serialize to bincode
let payload = state.serialize().unwrap();

// Wrap in a packet (auto-computes CRC32)
let packet = A2UIPacket::new(PacketType::StateUpdate, payload);

// Encode to wire bytes
let wire = packet.encode();

// Decode from wire bytes
let decoded = A2UIPacket::decode(&wire).unwrap();
assert_eq!(packet, decoded);

// Recover state
let recovered = UIState::deserialize(&decoded.payload).unwrap();
assert_eq!(state, recovered);
```

## Core Types

- **`UIState`** — Canonical rendering-agnostic UI state (rooms, agents, notifications, metrics)
- **`A2UIPacket`** — Wire-format packet with version, type, payload, and CRC32 checksum
- **`RenderRequest`** — Request to render state for a specific target (Terminal, Telegram, Dashboard, GameEngine, Voice, JSON, A2A)
- **`RoomState`** / **`AgentState`** / **`Notification`** / **`Control`** — State components
- **`Viewport`** / **`RenderOptions`** — Client dimensions and render flags

## Testing

```bash
cargo test
```

## License

MIT OR Apache-2.0
