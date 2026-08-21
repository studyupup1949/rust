//! End-to-end verification against a running robot. Exits 0 on full
//! success, non-zero on any failed check. Prints a tabular summary.
//!
//! Assumes another process (e.g. examples/webcam) is publishing the
//! `webcam` video track and stats under the same org.
//!
//!   ADAMO_API_KEY=<key> ADAMO_ROBOT_NAME=macbook cargo run --example verify

use std::collections::BTreeMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use adamo::{Protocol, PublishOptions, Session};

struct Check {
    name: &'static str,
    ok: bool,
    detail: String,
}

fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");
    let robot = std::env::var("ADAMO_ROBOT_NAME").unwrap_or_else(|_| "macbook".into());

    let mut checks: Vec<Check> = Vec::new();

    // --- session ---
    let t = Instant::now();
    let session = Session::open(&api_key, Protocol::Quic)?;
    let org = session.org()?.to_string();
    checks.push(Check {
        name: "session.open + org resolve",
        ok: !org.is_empty(),
        detail: format!("org={org} in {:?}", t.elapsed()),
    });

    // --- pub/sub round-trip ---
    let rt_key = "test/rust-verify";
    let sub = session.subscribe(rt_key)?;
    std::thread::sleep(Duration::from_millis(200));
    let payload = b"roundtrip";
    let t = Instant::now();
    session.put(
        rt_key,
        payload,
        PublishOptions {
            priority: 200,
            express: true,
        },
    )?;
    let rt = match sub.recv(Some(Duration::from_secs(3))) {
        Ok(s) => {
            let rtt = t.elapsed();
            checks.push(Check {
                name: "pub/sub round-trip",
                ok: s.payload == payload,
                detail: format!("rtt={:?} bytes={}", rtt, s.payload.len()),
            });
            true
        }
        Err(e) => {
            checks.push(Check {
                name: "pub/sub round-trip",
                ok: false,
                detail: format!("{e}"),
            });
            false
        }
    };
    drop(sub);
    let _ = rt;

    // --- stats/latency wire format + values ---
    let stats_key = format!("{robot}/stats/latency");
    let stats_sub = session.subscribe(&stats_key)?;
    // Collect a few samples so we can validate that values actually vary.
    let mut samples: Vec<serde_json::Value> = Vec::new();
    for _ in 0..3 {
        match stats_sub.recv(Some(Duration::from_secs(3))) {
            Ok(s) => match serde_json::from_slice::<serde_json::Value>(&s.payload) {
                Ok(v) => samples.push(v),
                Err(e) => {
                    checks.push(Check {
                        name: "stats/latency json parse",
                        ok: false,
                        detail: format!("{e}"),
                    });
                    break;
                }
            },
            Err(e) => {
                checks.push(Check {
                    name: "stats/latency recv",
                    ok: false,
                    detail: format!(
                        "no sample in 3s on `{stats_key}` — is the robot running? ({e})"
                    ),
                });
                break;
            }
        }
    }

    if !samples.is_empty() {
        checks.push(Check {
            name: "stats/latency recv",
            ok: true,
            detail: format!("got {} sample(s) on `{stats_key}`", samples.len()),
        });

        // Schema: frontend expects camelCase keys.
        let expected = [
            "type",
            "encoderLatencyMs",
            "captureLatencyMs",
            "pipelineLatencyMs",
            "framesEncoded",
            "timestamp",
        ];
        let s0 = &samples[0];
        let missing: Vec<_> = expected
            .iter()
            .filter(|k| s0.get(**k).is_none())
            .copied()
            .collect();
        checks.push(Check {
            name: "stats/latency schema (camelCase)",
            ok: missing.is_empty(),
            detail: if missing.is_empty() {
                "all fields present".into()
            } else {
                format!("missing: {missing:?}")
            },
        });

        // Values: capture must be in a plausible range (0..1000 ms).
        let cap = s0
            .get("captureLatencyMs")
            .and_then(|v| v.as_f64())
            .unwrap_or(-1.0);
        checks.push(Check {
            name: "captureLatencyMs in plausible range",
            ok: (0.0..1000.0).contains(&cap),
            detail: format!("captureLatencyMs = {cap}"),
        });

        let frames = s0
            .get("framesEncoded")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        checks.push(Check {
            name: "framesEncoded > 0",
            ok: frames > 0,
            detail: format!("framesEncoded = {frames}"),
        });

        let ts = s0.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0);
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let drift = (now_ms as i64 - ts as i64).abs();
        checks.push(Check {
            name: "timestamp is wall-clock ms (within 30s of now)",
            ok: drift < 30_000,
            detail: format!("timestamp = {ts}, drift = {drift} ms"),
        });

        // Verify values change over 3 samples (pipeline is live, not cached).
        let mut caps: Vec<f64> = samples
            .iter()
            .filter_map(|v| v.get("captureLatencyMs").and_then(|x| x.as_f64()))
            .collect();
        caps.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let spread = caps.last().copied().unwrap_or(0.0) - caps.first().copied().unwrap_or(0.0);
        checks.push(Check {
            name: "captureLatencyMs varies across samples",
            ok: spread > 0.0,
            detail: format!("caps={caps:?} spread={spread:.3}ms"),
        });
    }
    drop(stats_sub);

    // --- video frames ---
    let video_key = format!("{robot}/video/webcam");
    let video_sub = session.subscribe(&video_key)?;
    let mut video_frames = 0usize;
    let mut video_bytes = 0usize;
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match video_sub.try_recv()? {
            Some(s) => {
                video_frames += 1;
                video_bytes += s.payload.len();
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    }
    checks.push(Check {
        name: "video frames arriving",
        ok: video_frames > 0,
        detail: format!(
            "{} frames, {} bytes in 3s on `{video_key}`",
            video_frames, video_bytes
        ),
    });

    // --- ping/pong RTT (what the frontend uses for rttMs) ---
    // Protocol: frontend PUTs 4 bytes BE u32 req-id on stats/ping,
    // robot echoes same bytes back on stats/pong.
    let ping_key = format!("{robot}/stats/ping");
    let pong_key = format!("{robot}/stats/pong");
    let pong_sub = session.subscribe(&pong_key)?;
    // Let the subscription propagate before we fire the first ping.
    std::thread::sleep(Duration::from_millis(200));
    let req_id: u32 = 0xABCD_1234;
    let mut ping_payload = vec![0u8; 4];
    ping_payload[0] = (req_id >> 24) as u8;
    ping_payload[1] = (req_id >> 16) as u8;
    ping_payload[2] = (req_id >> 8) as u8;
    ping_payload[3] = req_id as u8;
    let ping_sent = Instant::now();
    session.put(
        &ping_key,
        &ping_payload,
        PublishOptions {
            priority: 200,
            express: true,
        },
    )?;
    let pong = pong_sub.recv(Some(Duration::from_secs(2)));
    let (pong_ok, pong_detail) = match pong {
        Ok(s) if s.payload == ping_payload => {
            (true, format!("rtt={:?} req_id=0x{req_id:08x} echoed", ping_sent.elapsed()))
        }
        Ok(s) => (
            false,
            format!("pong payload mismatch: got {:?}", s.payload),
        ),
        Err(e) => (false, format!("no pong in 2s — frontend will show null RTT ({e})")),
    };
    checks.push(Check {
        name: "ping/pong RTT responder (frontend rttMs)",
        ok: pong_ok,
        detail: pong_detail,
    });
    drop(pong_sub);

    // --- stats/system ---
    let sys_key = format!("{robot}/stats/system");
    let sys_sub = session.subscribe(&sys_key)?;
    let (sys_ok, sys_detail) = match sys_sub.recv(Some(Duration::from_secs(3))) {
        Ok(s) => match serde_json::from_slice::<serde_json::Value>(&s.payload) {
            Ok(v) => {
                let expected = ["cpu", "memoryPct", "memoryUsedMb", "timestamp"];
                let missing: Vec<_> = expected
                    .iter()
                    .filter(|k| v.get(**k).is_none())
                    .copied()
                    .collect();
                if missing.is_empty() {
                    (
                        true,
                        format!(
                            "cpu={:.1}% mem={:.0}%",
                            v.get("cpu").and_then(|x| x.as_f64()).unwrap_or(0.0),
                            v.get("memoryPct").and_then(|x| x.as_f64()).unwrap_or(0.0)
                        ),
                    )
                } else {
                    (false, format!("missing fields: {missing:?}"))
                }
            }
            Err(e) => (false, format!("json parse: {e}")),
        },
        Err(e) => (
            false,
            format!("no stats/system in 3s — frontend sidebar will be empty ({e})"),
        ),
    };
    checks.push(Check {
        name: "stats/system (frontend sidebar)",
        ok: sys_ok,
        detail: sys_detail,
    });

    // --- heartbeat (how the robot appears in discovery lists) ---
    let hb_key = format!("{robot}/heartbeat");
    let hb_sub = session.subscribe(&hb_key)?;
    let (hb_ok, hb_detail) = match hb_sub.recv(Some(Duration::from_secs(2))) {
        Ok(s) => match serde_json::from_slice::<serde_json::Value>(&s.payload) {
            Ok(v) => {
                let has_robot = v.get("robot").and_then(|x| x.as_str()) == Some(robot.as_str());
                let tracks = v
                    .get("tracks")
                    .and_then(|x| x.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                (
                    has_robot,
                    format!("robot={} tracks={tracks}", has_robot),
                )
            }
            Err(e) => (false, format!("json parse: {e}")),
        },
        Err(e) => (false, format!("no heartbeat: {e}")),
    };
    checks.push(Check {
        name: "heartbeat publisher (robot discovery)",
        ok: hb_ok,
        detail: hb_detail,
    });

    // --- report ---
    println!();
    let mut group: BTreeMap<&str, (bool, String)> = BTreeMap::new();
    for c in &checks {
        group.insert(c.name, (c.ok, c.detail.clone()));
    }
    for (name, (ok, detail)) in &group {
        let mark = if *ok { "PASS" } else { "FAIL" };
        println!("{mark:>4}  {name:<45}  {detail}");
    }

    let failed = checks.iter().filter(|c| !c.ok).count();
    println!();
    if failed == 0 {
        println!("== {} / {} checks passed ==", checks.len(), checks.len());
        Ok(())
    } else {
        eprintln!(
            "== {} / {} checks failed ==",
            failed,
            checks.len()
        );
        std::process::exit(1);
    }
}
