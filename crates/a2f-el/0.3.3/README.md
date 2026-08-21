Copyright (c) 2026 kcjsa

# A2F-EL - Analysis to Fake Protocol Essential Lite

**A2F-EL** is a lightweight, essential version of A2F. It removes multi-layer encryption and dummy packets, focusing on **practical security** with built-in key exchange.

The core idea: *"Send in chaos. Receive in order. Timestamps bind everything."*

## Difference from A2F (Full Version)

| Feature | A2F (Full) | **A2F-EL** |
|---------|-----------|-------------|
| Encryption | AES-256 + ChaCha20-Poly1305 | **ChaCha20-Poly1305 only** |
| Key Exchange | User-implemented | **X25519 built-in** |
| Dummy Packets | Yes | No |
| Heartbeat | Yes | No |
| Traffic Analysis Resistance | High | Basic (shuffle only) |
| Binary Size | Large | **~40% smaller** |

## How It Works

**Sender:**
1. Generate X25519 key pair for key exchange
2. Exchange public keys with receiver
3. Derive shared session key via HKDF
4. Encrypt data using ChaCha20-Poly1305
5. Attach timestamp and sequence number
6. Shuffle packets and send in random order

**Network:**
- Packets may arrive out of order
- Packets may be lost
- Packets may be delayed (high latency)

**Receiver:**
1. Receive packets in any order
2. Perform key exchange to obtain session key
3. Decrypt data using session key
4. Replay attack protection via sliding window

## Features

-  **Built-in X25519 key exchange** (Perfect Forward Secrecy)
-  **ChaCha20-Poly1305 only** (lightweight, no AES)
-  **Out-of-order delivery support**
-  **High-latency tolerant**
-  **Replay attack protection** (sliding window)
-  **Packet shuffling** (basic traffic analysis resistance)
-  **No async runtime required**

## Quick Start

Add to your Cargo.toml:
```
[dependencies]
a2f-el = "0.2.0"
```
Basic usage example:
```
use a2f_el::{A2FELSender, A2FELReceiver, A2FConfig, current_timestamp};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sender = A2FELSender::new();
    let mut receiver = A2FELReceiver::new(&A2FConfig::default());

    // Key exchange (mutual)
    let sender_pub = sender.start_key_exchange();
    let receiver_pub = receiver.get_public_key();
    
    receiver.complete_key_exchange(&sender_pub)?;
    sender.complete_key_exchange(&receiver_pub)?;

    // Encrypt and send
    let packet = sender.encrypt_data(b"Hello, A2F-EL!", current_timestamp())?;
    
    // Receive and decrypt
    if let Some(decrypted) = receiver.receive_packet(packet)? {
        println!("Decrypted: {}", String::from_utf8_lossy(&decrypted));
    }

    Ok(())
}
```
## Test Results

All tests passed:

- Basic key exchange → encrypt → decrypt: ✅
- Replay attack prevention: ✅
- Multiple messages with shuffle: ✅
- Out-of-order packet tolerance: ✅

## Use Cases

- Starlink and LEO satellite networks
- Long-distance high-latency networks
- Mobile and unstable connections
- Mesh networks
- Censorship circumvention
- Resource-constrained devices (IoT, embedded)

## Security

- **Encryption:** ChaCha20-Poly1305 (single layer, sufficient for most use cases)
- **Key Exchange:** X25519 (Elliptic Curve Diffie-Hellman) with HKDF key derivation
- **Replay Protection:** Sliding window (configurable size)
- **Traffic Analysis:** Basic protection via packet order shuffling
- **Forward Secrecy:** Yes (ephemeral X25519 keys)

## License

Apache-2.0

## Author

kcjsa on GitHub

- GitHub: https://github.com/kcjsa/a2f-el
- Qiita Article (Full A2F): https://qiita.com/kcjsa/items/c28c2201349c6d38361d
