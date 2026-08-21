Copyright (c) 2026 kcjsa
# A2F - Analysis to Fake Protocol

A2F is an asynchronous, out-of-order tolerant, high-latency cryptography protocol.

The core idea: "Send in chaos. Receive in order. Timestamps bind everything."

## How It Works

**Sender:**
1. Generate a session key K_t for timestamp t
2. Wrap the session key using AES-256 + ChaCha20-Poly1305
3. Encrypt the data using the session key
4. Attach timestamp t to both the wrapped key and encrypted data
5. Shuffle all packets and send them in random order

**Network:**
- Packets may arrive out of order
- Packets may be lost
- Packets may be delayed (high latency)

**Receiver:**
1. Receive packets in any order
2. Buffer packets by timestamp
3. When both the key and data for a timestamp arrive, decrypt the data
4. Old packets that never complete expire after a timeout

## Features

- Out-of-order delivery support
- Data can arrive before key - buffered until key arrives
- high-latency tolerant - packet completion is time-window based
- Dummy packet mixing - traffic analysis resistance
- Multi-layer encryption - AES-256 + ChaCha20-Poly1305
- Simple implementation - just timestamps and a buffer

## Quick Start

Add to your Cargo.toml:

[dependencies]
a2f = { git = "https://github.com/kcjsa/a2f" }
もしくは 
a2f = "0.1.0"

Basic usage example:
```
use a2f::{A2FConfig, A2FSender, A2FReceiver, current_timestamp};

fn main() -> anyhow::Result<()> {
    let master_key = [0x42; 32];
    let config = A2FConfig::default();
    
    let mut sender = A2FSender::new(master_key, &config);
    let mut receiver = A2FReceiver::new(master_key, &config);
    
    let ts = current_timestamp();
    
    let key_packet = sender.generate_key_packet(ts)?;
    let data_packet = sender.encrypt_data(b"Hello, A2F!", ts)?;
    
    let packets = sender.shuffle_packets(vec![key_packet, data_packet]);
    
    for packet in packets {
        if let Some(decrypted) = receiver.receive_packet(packet)? {
            println!("Decrypted: {}", String::from_utf8_lossy(&decrypted));
        }
    }
    
    Ok(())
}
```

## Test Results

All tests passed:

- Basic encrypt/decrypt: OK
- Key before data: OK
- Data before key: OK
- Multiple messages mixed: OK
- Dummy packets with packet loss: OK
- Real UDP communication: OK

## Use Cases

- Starlink and LEO satellite networks
- Long-distance high-latency networks
- Mobile and unstable connections
- Mesh networks
- Censorship circumvention

## Security

- Multi-layer encryption: AES-256 plus ChaCha20-Poly1305
- Traffic analysis resistant due to random packet order
- A2F is designed as a transport-agnostic layer and can be implemented over different unreliable or partially reliable datagram transports such as UDP, RakNet, or WebRTC data channels via adapters.
- Ephemeral session keys

## License

Apache-2.0

## Author

kcjsa on GitHub

https://qiita.com/kcjsa/items/c28c2201349c6d38361d

