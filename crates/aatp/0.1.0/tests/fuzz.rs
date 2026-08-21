//! Dependency-free fuzz / property harness. Throws deterministic pseudo-random bytes at
//! every parser and asserts it NEVER panics and always returns `Ok`-or-typed-`Error`, and
//! that the structured round-trips are identities. Upgrades the "fails closed on hostile
//! input" guarantee from an audit-by-trace to an empirical one over thousands of inputs.
//!
//! Zero dependencies (a tiny SplitMix64 PRNG), and the seed is FIXED so a `cargo test`
//! run is reproducible and the mutation prober stays stable. Soak longer in CI with
//! `AATP_FUZZ_ITERS=1000000 cargo test --test fuzz`.

use aatp::codec::{Reader, Writer};
use aatp::message::{self, AgentMessage};
use aatp::{OVERHEAD, decode, dict, encode, shadow};

/// A tiny dependency-free PRNG (SplitMix64).
struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn byte(&mut self) -> u8 {
        (self.next_u64() & 0xFF) as u8
    }
    /// A value in `0..=n` (inclusive, so `upto(0)` is a valid empty-length draw).
    fn upto(&mut self, n: usize) -> usize {
        (self.next_u64() % (n as u64 + 1)) as usize
    }
    /// A random byte vector of length `0..max` (draws the length itself, so callers
    /// borrow `rng` exactly once).
    fn blob(&mut self, max: usize) -> Vec<u8> {
        let len = self.upto(max);
        (0..len).map(|_| self.byte()).collect()
    }
    /// A random ASCII string of length `0..max` (always valid UTF-8).
    fn text(&mut self, max: usize) -> String {
        let len = self.upto(max);
        (0..len)
            .map(|_| (b'a' + (self.byte() % 26)) as char)
            .collect()
    }
}

fn iters(default: usize) -> usize {
    std::env::var("AATP_FUZZ_ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

/// No parser may panic, over-read, over-allocate, or hang on ARBITRARY bytes — every call
/// must return `Ok` or a typed `Err`. Reaching the end of the loop *is* the assertion.
#[test]
fn parsers_never_panic_on_arbitrary_bytes() {
    let mut rng = Rng(0x1234_5678_9ABC_DEF0);
    let mut out = vec![0u8; 8192];
    for _ in 0..iters(20_000) {
        let buf = rng.blob(400);
        let _ = decode(&buf);
        let _ = message::decode(&buf);
        let _ = dict::decompress(&buf, &mut out);
        let _ = shadow::mask(&buf, rng.next_u64(), &mut out);
        let mut r = Reader::new(&buf);
        // walk a mix of field reads; each fails closed when the buffer runs out
        let _ = r.u8();
        let _ = r.u64();
        let _ = r.str();
        let _ = r.bytes();
        let _ = r.u32();
    }
}

/// `decode ∘ encode` is the identity for any payload; the wrong-length or corrupt cases
/// fail closed. Also exercises the full seal→open chain (compress + mask + frame).
#[test]
fn frame_and_full_chain_round_trips_hold() {
    let mut rng = Rng(0xCAFE_F00D_1234_5678);
    for _ in 0..iters(20_000) {
        let payload = rng.blob(300);
        let ty = rng.byte();

        // bare frame round trip
        let mut frame = vec![0u8; OVERHEAD + payload.len()];
        let n = encode(ty, &payload, &mut frame).unwrap();
        let pkt = decode(&frame[..n]).unwrap();
        assert_eq!(pkt.msg_type(), ty);
        assert_eq!(pkt.payload(), &payload[..]);

        // flip one byte anywhere and it must be refused (never silently accepted)
        if n > 0 {
            let i = rng.upto(n - 1); // in-bounds index 0..n
            frame[i] ^= 1 | rng.byte();
            // the only frames a single flip can still validate are ones where the flip
            // landed in trailing/ignored space — payload must still match, and magic/
            // version/CRC are all covered, so any *real* change is caught.
            if let Ok(p) = decode(&frame[..n]) {
                assert!(
                    p.msg_type() == ty && p.payload() == &payload[..],
                    "a byte flip produced a *different* valid frame"
                );
            }
        }
    }
}

/// `decompress ∘ compress` and `mask ∘ mask` are identities over random data.
#[test]
fn dict_and_shadow_are_invertible_over_random_data() {
    let mut rng = Rng(0x0BAD_C0DE_F00D_BEEF);
    for _ in 0..iters(20_000) {
        let data = rng.blob(256);

        // dictionary compression round trip
        let mut packed = vec![0u8; data.len() * 2 + 4];
        let plen = dict::compress(&data, &mut packed).unwrap();
        let mut back = vec![0u8; data.len() + 16];
        let blen = dict::decompress(&packed[..plen], &mut back).unwrap();
        assert_eq!(
            &back[..blen],
            &data[..],
            "dict round trip must be the identity"
        );

        // shadow mask is its own inverse
        let key = rng.next_u64();
        let mut once = vec![0u8; data.len()];
        shadow::mask(&data, key, &mut once).unwrap();
        let mut twice = vec![0u8; data.len()];
        shadow::mask(&once, key, &mut twice).unwrap();
        assert_eq!(&twice[..], &data[..], "mask twice must be the identity");
    }
}

/// The cursor and the message codec round-trip any structured value.
#[test]
fn cursor_and_message_round_trips_hold() {
    let mut rng = Rng(0xF00D_01CE_C0FF_EE00);
    for _ in 0..iters(20_000) {
        // typed cursor round trip
        let (a, b, c, d) = (
            rng.byte(),
            rng.next_u64() as u16,
            rng.next_u64() as u32,
            rng.next_u64(),
        );
        let blob = rng.blob(64);
        let s = rng.text(40);
        let mut buf = vec![0u8; 256];
        let n = {
            let mut w = Writer::new(&mut buf);
            w.u8(a).unwrap();
            w.u16(b).unwrap();
            w.u32(c).unwrap();
            w.u64(d).unwrap();
            w.bytes(&blob).unwrap();
            w.str(&s).unwrap();
            w.position()
        };
        let mut r = Reader::new(&buf[..n]);
        assert_eq!(r.u8().unwrap(), a);
        assert_eq!(r.u16().unwrap(), b);
        assert_eq!(r.u32().unwrap(), c);
        assert_eq!(r.u64().unwrap(), d);
        assert_eq!(r.bytes().unwrap(), &blob[..]);
        assert_eq!(r.str().unwrap(), s);

        // AgentMessage round trip
        let name = rng.text(30);
        let content = rng.text(120);
        let msg = AgentMessage {
            id: rng.next_u64(),
            role: rng.byte(),
            name: &name,
            content: &content,
            token_count: rng.next_u64() as u32,
        };
        let mut mbuf = vec![0u8; message::encoded_len(&msg)];
        message::encode(&msg, &mut mbuf).unwrap();
        assert_eq!(message::decode(&mbuf).unwrap(), msg);
    }
}
