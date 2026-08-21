# AATP: Autonomous Agent Transport Protocol

<p align="center">
  <a href="https://github.com/MerlijnW70/aatp/actions"><img src="https://github.com/MerlijnW70/aatp/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT">
  <img src="https://img.shields.io/badge/no__std-compatible-green.svg" alt="no_std">
  <img src="https://img.shields.io/badge/unsafe-forbidden-success.svg" alt="forbid(unsafe)">
</p>

AATP is a zero-dependency, `forbid(unsafe)` binary transport protocol engineered specifically for autonomous agent communication. It prioritizes deterministic performance, memory safety, and verifiable integrity.

## Design Philosophy
AATP is built on the principle of **Structural Assurance**. Rather than relying on complex, black-box macros or heavy dependencies, AATP provides a transparent, audited interface.

- **Audited Integrity:** 100% mutation-tested coverage (0 survivors).
- **Embedded Ready:** `no_std` compatible, verified on `thumbv7em-none-eabihf`.
- **Zero-Dependency:** No external serialization crates or bloat.
- **Fail-Closed:** Designed for high-assurance environments where memory corruption is not an option.

## Integrity Verification
AATP has undergone an adversarial deep-audit by autonomous agents and exhaustive mutation testing.

| Finding | Severity | Fix |
|:---|:---|:---|
| Comparative speed claim (unbacked) | High | Softened to structural argument |
| Dead repository URL | Med | Removed / Updated |
| Macro E0392 (all-scalar struct) | Med | Added no-lifetime macro arm |
| Macro hollow coverage (u16 arm) | Low-Med | Added dedicated test case |
| Dictionary pinning typo | Low-Med | Implemented snapshot tests |
| Target/ build artifact cruft | Low | Cleaned & .gitignore fixed |
| CRC vs. Authentication scope | Low | Added explicit security caveats |

*Verified empirically: every 1-bit corruption in the dictionary, XOR key, or boundary offsets triggers an immediate test failure.* The `tests/fuzz.rs` harness backs this with a deterministic property/fuzz pass — thousands of random inputs per run assert no parser panics and that every round-trip is an identity.

## Quick Start
Define your agent messages with the `aatp_message!` macro and get an allocation-free codec for free:

```rust
use aatp::aatp_message;

aatp_message! {
    pub struct Ping {
        seq: u64,
        code: u16,
    }
}

let ping = Ping { seq: 42, code: 7 };
let mut buf = [0u8; 32];
let n = ping.encode(&mut buf).unwrap();
assert_eq!(Ping::decode(&buf[..n]).unwrap(), ping);
```

String and byte fields are **decoded zero-copy** — they borrow from the input buffer, no allocation:

```rust
use aatp::aatp_message;

aatp_message! {
    pub struct ToolCall<'a> {
        id: u64,
        tool: str,
        args: str,
    }
}

let call = ToolCall { id: 7, tool: "web_search", args: "{\"q\":\"aatp\"}" };
let mut buf = [0u8; 128];
let n = call.encode(&mut buf).unwrap();
assert_eq!(ToolCall::decode(&buf[..n]).unwrap(), call); // `tool`/`args` borrow `buf`
```

Or frame an arbitrary payload for the wire — magic, version, length, and CRC32C all verified on decode:

```rust
let mut frame = vec![0u8; aatp::OVERHEAD + payload.len()];
let n = aatp::encode(0x02, &payload, &mut frame)?;   // [Magic][Ver][Type][Len][Payload][CRC32C]
let pkt = aatp::decode(&frame[..n])?;                // magic, version, length, CRC all checked
```

## What's in the box

| Module | What |
| --- | --- |
| `lib` | Frame codec — `[Magic][Version][Type][Length][Payload][CRC32C]`, table-driven CRC32C |
| `codec` + `aatp_message!` | Safe typed `Writer`/`Reader` cursors and a macro that generates a struct's codec |
| `message` | A ready `AgentMessage` payload (id, role, name, content, tokens) |
| `state` | Session machine `Handshake → Active → Closed`; refuses out-of-order frames |
| `dict` | Compile-time dictionary compression — recompile with your vocabulary |
| `shadow` | Reversible XOR keystream — **obfuscation, see the warning** |

## Install

```toml
[dependencies]
aatp = "0.1"
```

## On performance — an argument from structure, not a benchmark

AATP's edge is *structural*: it skips a text format's tokenizing and per-string allocation entirely and decodes by borrowing fields in place, so on streams of many small structured messages (tool calls, status, turns) there is simply less work per message — while *also* carrying an integrity checksum a text format never does. On bulk, paragraph-sized payloads that edge narrows: a checksummed binary format is byte-throughput-bound. This crate ships **no cross-format benchmark**; `benches/deser.rs` reports AATP's own throughput. The comparison above is an argument from structure, not a measured claim.

## ⚠️ `shadow` is obfuscation, not encryption

`shadow` XORs the payload with a keystream. It is exactly reversible and hides bytes from a casual glance, but a static-key XOR stream is **trivially broken** (known-plaintext recovers the keystream; two messages under one key reveal each other) and is **unauthenticated**. **Do not use it for confidentiality.** For real secrecy use a vetted authenticated-encryption (AEAD) construction. Its 100% mutation coverage proves it is *correct and reversible* — **not** *secret*.

Likewise, the frame's **CRC32C detects accidental corruption, not a malicious sender** — an attacker can recompute it. Pair AATP with a MAC/AEAD if you need to *trust* the peer, not merely detect line noise.

## On the testing claim

"100% mutation coverage" means **zero survivors under the mutation operator set** used to audit it — comparison/boolean/arithmetic swaps, condition forcing, and unary-negation removal. It is a strong signal, not a proof of total correctness: the operator set does not, for example, mutate integer literals, so wide constant tables (the CRC32C table, the SplitMix64 keystream, the dictionary) are additionally pinned by explicit known-answer, snapshot, and digest tests. A green build means the tested behaviour is genuinely pinned — it does not mean "provably bug-free."

## License

MIT.
