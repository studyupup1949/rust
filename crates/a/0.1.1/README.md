# a

A Rust library that implements the **protocol layer** of China MEE **HJ 212** (ASCII messages).

This crate focuses on reusable building blocks:

- **Receiving**: extract frames from a TCP/serial byte stream (sticky packets) → parse a frame → get a structured `Hj212Packet`
- **Sending**: build `CP=&&...&&` and payload via builders → wrap into an HJ212 frame (`##{LEN}{PAYLOAD}{CRC}`)

## Scope

Included:

- Framing: `##{LEN}{PAYLOAD}{CRC16_HEX}` (HJ 212—2025 ANSI CRC16)
- Parse/build: `parse_frame` / `build_frame`
- Streaming framer: `Framer`
- Builders: `CpBuilder` / `PayloadBuilder`
- Helpers: CRC functions, DataTime parsing
- Standard-alignment helpers:
  - `build_frame_standard` (uppercase CRC + trailing `\r\n`)
  - `parse_frame_strict` (requires 4-digit length + CRC + `\r\n`)
- Appendix C helpers: `PNUM/PNO` support and common ACK payload builders (`build_qn_rtn`, `build_exe_rtn`, `build_data_ack`, `build_notify_ack`)
- Appendix A/H helpers (optional): encryption helpers under `crypto` (see below)

Intentionally NOT included:

- Network I/O (Tokio/Actix/TCP server), serial port reading/writing
- Database/storage, platform-specific mapping, UI
- HTTPS upload logic (Appendix H multimedia upload)

## Quick start

### 1) Parse a single complete frame

```rust
use a::{build_frame, parse_frame};

let payload = "QN=1;ST=22;CN=2011;PW=123456;MN=ABC;Flag=7;CP=&&DataTime=20250101010101;a21026-Rtd=12.3&&";
let frame = build_frame(payload);
assert!(frame.ends_with("\r\n"));

let pkt = parse_frame(&frame).unwrap();
assert_eq!(pkt.mn.as_deref(), Some("ABC"));
assert_eq!(pkt.cp.get("a21026-Rtd").map(String::as_str), Some("12.3"));
```

### 2) Extract frames from a byte stream

```rust,no_run
use a::{Framer, parse_frame};

let mut framer = Framer::new();
framer.push(b"##0025QN=1;ST=22;CN=2011;CP=&&&&");

while let Some(frame_bytes) = framer.next_frame() {
    let frame_str = String::from_utf8(frame_bytes).expect("HJ212 is ASCII");
    let _pkt = parse_frame(&frame_str)?;
}

# Ok::<(), a::Hj212Error>(())
```

### 3) Send-side: builders

```rust
use a::{CpBuilder, PayloadBuilder, parse_frame};

let mut cp = CpBuilder::new();
cp.data_time("20250101010101")
  .rtd_flag("a21026", "12.3", "N")
  .kv("a21026-Zs", "0.0");

let frame = PayloadBuilder::new(
    "20251225123000001", // QN
    "123456",            // PW
    "ABC",               // MN
    cp.build(),
)
.st("22")
.cn("2011")
.flag("7")
.frame();

let pkt = parse_frame(&frame).unwrap();
assert_eq!(pkt.mn.as_deref(), Some("ABC"));
```

## Optional: SM4 auth encryption (Appendix H)

The standard’s multimedia upload appendix specifies:

- `Authorization` plaintext is `username:password`
- SM4-ECB + PKCS7Padding
- then Base64

Enable feature `sm4`:

```toml
a = { version = "0.1", features = ["sm4"] }
```

And use `a::crypto::sm4_auth`.

## Notes on standard vs compat frames

- `build_frame(...)` emits **standard** frames: 4-digit LEN + uppercase CRC + trailing `\r\n`.
- If you must interop with a legacy variant without CRLF, use `build_frame_compat(...)`.

## CRC16 and LEN (important)

- `LEN` is the **ASCII byte length of the payload** (the bytes between `{LEN}` and `{CRC}`), i.e. `payload.as_bytes().len()`.
  It does **not** include the `##` prefix, the `LEN` digits, the 4-byte CRC text, or the trailing `\r\n`.
- CRC is computed over the **payload bytes only** using the HJ 212—2025 “ANSI CRC16” algorithm (poly `0xA001`, init `0xFFFF`, and the update step `crc = (crc >> 8) ^ byte`).
- For historical interop, this crate also exposes the CRC16/Modbus variant via `crc16_modbus_hex_upper/lower`.

Standard example from the text (CRC = `2200`):

```
##0087QN=20240601085857223;ST=32;CN=1011;PW=123456;MN=010000A8900016F000169DC0;Flag=9;CP=
&&&&2200\r\n
```

## Notes on CP parsing

- `parse_frame*` parses `CP=&&...&&` into `pkt.cp` by splitting **only on `;`** into `k=v` pairs.
- Some platform payloads group fields with commas (e.g. `a34006-Avg=... ,a34006-Flag=N`). Those commas are intentionally left for the **business layer** to interpret.
