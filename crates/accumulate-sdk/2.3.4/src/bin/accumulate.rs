//! Accumulate SDK command-line interface (RB-04).
//!
//! Contract: docs/ai-agent-readiness/CLI-SPEC.md in accumulate-studio.
//!   * Under `--json`, stdout carries EXACTLY ONE envelope object. Logs go to stderr.
//!   * Exit codes: 0 ok · 1 operation failed · 2 usage error · 3 network unreachable.
//!   * Errors carry canonical `ACC_*` codes so `retryable` tells an agent whether a
//!     retry is productive instead of leaving it to guess.
//!   * Never prompts. Mainnet needs `--network mainnet` AND `ACCUMULATE_ALLOW_MAINNET=1`.
//!
//! argv parsing is hand-rolled rather than pulling in `clap`: the surface is 13
//! verbs with a handful of flags, and the other four SDK CLIs parse by hand too,
//! so this keeps behaviour identical and the dependency set unchanged.

use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const ENVELOPE_VERSION: &str = "1";
const SDK_NAME: &str = "rust";
const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

const EXIT_OK: i32 = 0;
const EXIT_FAILED: i32 = 1;
const EXIT_USAGE: i32 = 2;
const EXIT_NETWORK: i32 = 3;

const DEFAULT_NETWORK: &str = "kermit";

struct CatalogEntry {
    category: &'static str,
    retryable: bool,
    protocol_codes: &'static [i64],
    patterns: &'static [&'static str],
    hint: &'static str,
    remediation: &'static str,
}

/// Mirrors `packages/codegen/src/manifests/errors.catalog.json`. Wire codes were
/// verified against a live node by `tools/agent-harness/negative-cases.mjs`.
static CATALOG: &[(&str, CatalogEntry)] = &[
    ("ACC_ACCOUNT_NOT_FOUND", CatalogEntry {
        category: "not_found", retryable: false, protocol_codes: &[-32807, -33404],
        patterns: &["accumulate error not found", "not found", "-32807", "-33404"],
        hint: "The account URL does not exist on this network.",
        remediation: "Verify the URL and the network. If you just created the account, wait for its creating transaction to reach 'delivered' first. Note that on the V2 API a malformed URL is also reported as not-found.",
    }),
    ("ACC_INVALID_PARAMS", CatalogEntry {
        category: "validation", retryable: false, protocol_codes: &[-32802, -32602],
        patterns: &["validation error", "field validation for", "invalid params", "-32802", "-32602"],
        hint: "The request parameters were rejected by the node.",
        remediation: "Check the operation's declared inputs. Hashes are 32-byte hex; amounts are base-unit integers.",
    }),
    ("ACC_METHOD_NOT_FOUND", CatalogEntry {
        category: "validation", retryable: false, protocol_codes: &[-32601],
        patterns: &["method not found", "-32601"],
        hint: "The node does not expose the RPC method that was called.",
        remediation: "Use the SDK's canonical client rather than raw RPC; it targets the right API version.",
    }),
    ("ACC_ROUTING_FAILED", CatalogEntry {
        category: "validation", retryable: false, protocol_codes: &[-33400],
        patterns: &["cannot route request", "nothing to route", "scope is missing", "-33400"],
        hint: "The node could not determine which partition should handle the request.",
        remediation: "Every transaction needs a header with a valid `principal` — that URL is the routing key. Build envelopes with TxBody + SmartSigner rather than by hand.",
    }),
    ("ACC_INSUFFICIENT_CREDITS", CatalogEntry {
        category: "insufficient_credits", retryable: false, protocol_codes: &[],
        patterns: &["insufficientcredits", "insufficient credits"],
        hint: "The signing key page does not hold enough credits to pay for this transaction.",
        remediation: "Call add_credits for the SIGNING key page, then wait for the credits to settle.",
    }),
    ("ACC_UNAUTHORIZED_SIGNER", CatalogEntry {
        category: "auth", retryable: false, protocol_codes: &[403],
        patterns: &["unauthorized", "key does not belong to signer"],
        hint: "The signing key is not on the key page that authorizes this principal.",
        remediation: "Sign with a key on the principal's authorizing key page (after create_identity, `<adi>/book/1`).",
    }),
    ("ACC_INSUFFICIENT_BALANCE", CatalogEntry {
        category: "insufficient_balance", retryable: false, protocol_codes: &[],
        patterns: &["insufficient balance", "insufficient funds", "exceeds balance"],
        hint: "The source account does not hold enough tokens for this transfer.",
        remediation: "Confirm the balance first. 1 ACME = 1e8 base units; custom tokens carry their own precision.",
    }),
    ("ACC_NETWORK_UNAVAILABLE", CatalogEntry {
        category: "network", retryable: true, protocol_codes: &[],
        patterns: &["econnrefused", "econnreset", "etimedout", "timeout", "connection closed",
                    "connection reset", "connection refused", "service unavailable",
                    "error sending request", "dns error", "tcp connect error", "operation timed out"],
        hint: "The endpoint could not be reached, or the request timed out.",
        remediation: "Retry with exponential backoff. This is the only class where a bare retry is productive.",
    }),
    ("ACC_INTERNAL", CatalogEntry {
        category: "internal", retryable: true, protocol_codes: &[-32603],
        patterns: &["internal error", "-32603"],
        hint: "The node reported an internal error.",
        remediation: "Retry once with backoff. If it persists, re-check the request shape.",
    }),
    ("ACC_USAGE", CatalogEntry {
        category: "validation", retryable: false, protocol_codes: &[], patterns: &[],
        hint: "The command was invoked incorrectly.",
        remediation: "Run `accumulate --help --json` for the full command tree, flags and required arguments.",
    }),
];

fn entry(code: &str) -> &'static CatalogEntry {
    CATALOG.iter().find(|(c, _)| *c == code).map(|(_, e)| e).expect("unknown catalog code")
}

/// Map a raw error string onto a catalog code. Longest pattern wins, so
/// "key does not belong to signer" beats a bare "unauthorized".
///
/// Unrecognized errors deliberately fall back to a NON-retryable code: unknown
/// failures are far more often malformed requests than transient faults, and
/// defaulting to retryable is how an agent burns its turn budget in a loop.
fn classify(raw: &str) -> &'static str {
    let text = raw.to_lowercase();
    let mut best: Option<(usize, &'static str)> = None;
    for (code, e) in CATALOG {
        for p in e.patterns {
            if text.contains(p) && best.map_or(true, |(len, _)| p.len() > len) {
                best = Some((p.len(), code));
            }
        }
    }
    best.map(|(_, c)| c).unwrap_or("ACC_INVALID_PARAMS")
}

struct Usage(String);

struct VerbSpec {
    name: &'static str,
    summary: &'static str,
    network: bool,
    signs: bool,
    /// (name, type, required)
    args: &'static [(&'static str, &'static str, bool)],
    /// (name, type, required, default, repeatable)
    flags: &'static [(&'static str, &'static str, bool, Option<&'static str>, bool)],
}

static VERBS: &[VerbSpec] = &[
    VerbSpec { name: "query", summary: "Query any Accumulate account", network: true, signs: false,
        args: &[("url", "string", true)], flags: &[] },
    VerbSpec { name: "balance", summary: "Get a token account balance", network: true, signs: false,
        args: &[("url", "string", true)], flags: &[] },
    VerbSpec { name: "chain", summary: "Read chain entries for an account", network: true, signs: false,
        args: &[("url", "string", true)],
        flags: &[("--chain", "string", false, Some("main"), false),
                 ("--start", "integer", false, Some("0"), false),
                 ("--count", "integer", false, Some("10"), false)] },
    VerbSpec { name: "faucet", summary: "Request testnet ACME for a lite token account", network: true, signs: false,
        args: &[("url", "string", true)], flags: &[] },
    VerbSpec { name: "credits estimate", summary: "Estimate credits purchased for an ACME amount", network: true, signs: false,
        args: &[("url", "string", true)], flags: &[("--amount", "number", true, None, false)] },
    VerbSpec { name: "tx build", summary: "Build an unsigned transaction body", network: false, signs: false,
        args: &[("op", "string", true)], flags: &[("--param", "key=value", false, None, true)] },
    VerbSpec { name: "tx submit", summary: "Submit a signed envelope", network: true, signs: true,
        args: &[], flags: &[("--envelope", "path", true, None, false),
                            ("--key-file", "path", false, None, false),
                            ("--key-env", "string", false, None, false)] },
    VerbSpec { name: "tx wait", summary: "Poll a transaction until it reaches a final state", network: true, signs: false,
        args: &[("txid", "string", true)], flags: &[("--timeout", "integer", false, Some("60"), false)] },
    VerbSpec { name: "tx status", summary: "Read a transaction's current status", network: true, signs: false,
        args: &[("txid", "string", true)], flags: &[] },
    VerbSpec { name: "keys generate", summary: "Generate a keypair (never written to disk)", network: false, signs: false,
        args: &[], flags: &[("--algorithm", "string", false, Some("ed25519"), false)] },
    VerbSpec { name: "net list", summary: "List known networks", network: false, signs: false, args: &[], flags: &[] },
    VerbSpec { name: "net status", summary: "Check the selected network's reachability", network: true, signs: false,
        args: &[], flags: &[] },
    VerbSpec { name: "version", summary: "Report SDK and envelope versions", network: false, signs: false,
        args: &[], flags: &[] },
];

static GROUPS: &[&str] = &["credits", "tx", "keys", "net"];

fn command_tree() -> Value {
    let verbs: Vec<Value> = VERBS.iter().map(|v| {
        json!({
            "name": v.name, "summary": v.summary, "network": v.network, "signs": v.signs,
            "args": v.args.iter().map(|(n, t, r)| json!({"name": n, "type": t, "required": r})).collect::<Vec<_>>(),
            "flags": v.flags.iter().map(|(n, t, r, d, rep)| {
                let mut m = Map::new();
                m.insert("name".into(), json!(n));
                m.insert("type".into(), json!(t));
                m.insert("required".into(), json!(r));
                if let Some(dv) = d { m.insert("default".into(), json!(dv)); }
                if *rep { m.insert("repeatable".into(), json!(true)); }
                Value::Object(m)
            }).collect::<Vec<_>>(),
        })
    }).collect();
    json!({
        "command": "accumulate",
        "envelopeVersion": ENVELOPE_VERSION,
        "globalFlags": [
            {"name": "--json", "type": "boolean", "summary": "Emit one envelope object on stdout"},
            {"name": "--network", "type": "string", "default": DEFAULT_NETWORK,
             "summary": "Target network; mainnet also requires ACCUMULATE_ALLOW_MAINNET=1"},
            {"name": "--help", "type": "boolean", "summary": "Show help; with --json returns the command tree"},
        ],
        "verbs": verbs,
    })
}

/// Owns stdout. Exactly one object is ever written to it.
struct Emitter {
    as_json: bool,
    network: Option<String>,
    started: Instant,
}

impl Emitter {
    fn meta(&self) -> Value {
        json!({
            "network": self.network,
            "sdk": SDK_NAME,
            "version": SDK_VERSION,
            "durationMs": self.started.elapsed().as_millis() as u64,
        })
    }

    fn ok(&self, data: Value) -> i32 {
        if self.as_json {
            println!("{}", json!({
                "envelope": ENVELOPE_VERSION, "ok": true, "data": data, "meta": self.meta()
            }));
        } else {
            println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
        }
        EXIT_OK
    }

    fn fail(&self, raw: &str, code: Option<&str>, exit_code: Option<i32>) -> i32 {
        let resolved = code.unwrap_or_else(|| classify(raw));
        let e = entry(resolved);
        let mut err = Map::new();
        err.insert("code".into(), json!(resolved));
        err.insert("category".into(), json!(e.category));
        err.insert("retryable".into(), json!(e.retryable));
        err.insert("hint".into(), json!(e.hint));
        err.insert("remediation".into(), json!(e.remediation));
        err.insert("raw".into(), json!(raw));
        if !e.protocol_codes.is_empty() {
            err.insert("protocolCodes".into(), json!(e.protocol_codes));
        }

        let ec = exit_code.unwrap_or(match resolved {
            "ACC_USAGE" => EXIT_USAGE,
            "ACC_NETWORK_UNAVAILABLE" => EXIT_NETWORK,
            _ => EXIT_FAILED,
        });
        if self.as_json {
            println!("{}", json!({
                "envelope": ENVELOPE_VERSION, "ok": false, "error": Value::Object(err), "meta": self.meta()
            }));
        } else {
            eprintln!("error: {}: {}", resolved, e.hint);
            eprintln!("  retryable: {}", if e.retryable { "yes" } else { "no" });
            eprintln!("  fix: {}", e.remediation);
        }
        ec
    }
}

fn base_url(network: &str) -> Result<&'static str, Usage> {
    match network {
        "mainnet" => {
            if std::env::var("ACCUMULATE_ALLOW_MAINNET").ok().as_deref() != Some("1") {
                return Err(Usage("refusing to target mainnet: pass --network mainnet AND set \
                                  ACCUMULATE_ALLOW_MAINNET=1. Both are required, deliberately.".into()));
            }
            Ok("https://mainnet.accumulatenetwork.io")
        }
        "kermit" => Ok("https://kermit.accumulatenetwork.io"),
        "testnet" => Ok("https://testnet.accumulatenetwork.io"),
        "local" => Ok("http://localhost:26660"),
        other => Err(Usage(format!(
            "unknown network '{other}' — known: kermit, testnet, mainnet, local"))),
    }
}

/// One JSON-RPC round trip. A protocol error is returned in the payload; a
/// transport failure is an `Err`.
async fn rpc(base: &str, version: &str, method: &str, params: Value) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .post(format!("{base}/{version}"))
        .json(&json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}))
        .send()
        .await
        .map_err(|e| format!("{e}"))?;
    let text = res.text().await.map_err(|e| format!("{e}"))?;
    serde_json::from_str::<Value>(&text)
        .map_err(|e| format!("non-JSON response: {e}: {}", &text.chars().take(160).collect::<String>()))
}

fn rpc_error_text(err: &Value) -> String {
    let msg = err.get("message").and_then(|m| m.as_str()).unwrap_or("rpc error");
    let code = err.get("code").and_then(|c| c.as_i64());
    let data = err.get("data").map(|d| d.to_string()).unwrap_or_default();
    match code {
        Some(c) => format!("{msg} {data} ({c})"),
        None => format!("{msg} {data}"),
    }
}

fn parse_verb(tokens: &[String]) -> Result<(&'static str, Vec<String>), Usage> {
    let head = tokens.first().ok_or_else(|| Usage(
        "no verb given — run `accumulate --help --json` for the command tree".into()))?;
    if GROUPS.contains(&head.as_str()) {
        if tokens.len() < 2 {
            return Err(Usage(format!("'{head}' is a command group; it needs a subcommand")));
        }
        let name = format!("{head} {}", tokens[1]);
        let found = VERBS.iter().find(|v| v.name == name).ok_or_else(|| Usage(
            format!("unknown subcommand '{}' for group '{head}'", tokens[1])))?;
        return Ok((found.name, tokens[2..].to_vec()));
    }
    let found = VERBS.iter().find(|v| v.name == head.as_str()).ok_or_else(|| Usage(
        format!("unknown verb '{head}' — run `accumulate --help --json` for the command tree")))?;
    Ok((found.name, tokens[1..].to_vec()))
}

#[derive(Default)]
struct Args {
    values: BTreeMap<String, String>,
    repeated: BTreeMap<String, Vec<String>>,
}

impl Args {
    fn get(&self, k: &str) -> Option<&str> { self.values.get(k).map(|s| s.as_str()) }
    fn int(&self, k: &str) -> i64 { self.get(k).and_then(|v| v.parse().ok()).unwrap_or(0) }
}

fn flag_key(name: &str) -> String { name.trim_start_matches("--").replace('-', "_") }

fn parse_verb_args(verb: &str, tokens: &[String]) -> Result<Args, Usage> {
    let spec = VERBS.iter().find(|v| v.name == verb).expect("verb exists");
    let mut out = Args::default();
    for (name, _, _, default, repeatable) in spec.flags {
        let key = flag_key(name);
        if let Some(d) = default { out.values.insert(key.clone(), (*d).to_string()); }
        if *repeatable { out.repeated.insert(key, Vec::new()); }
    }

    let mut positional: Vec<String> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        let t = &tokens[i];
        if t.starts_with("--") {
            let f = spec.flags.iter().find(|(n, _, _, _, _)| n == t)
                .ok_or_else(|| Usage(format!("unknown flag '{t}' for '{verb}'")))?;
            i += 1;
            let raw = tokens.get(i).ok_or_else(|| Usage(format!("flag '{t}' expects a value")))?;
            let key = flag_key(t);
            if f.4 {
                out.repeated.entry(key).or_default().push(raw.clone());
            } else {
                if f.1 == "integer" && raw.parse::<i64>().is_err() {
                    return Err(Usage(format!("flag '{t}' expects an integer, got '{raw}'")));
                }
                if f.1 == "number" && raw.parse::<f64>().is_err() {
                    return Err(Usage(format!("flag '{t}' expects a number, got '{raw}'")));
                }
                out.values.insert(key, raw.clone());
            }
        } else {
            positional.push(t.clone());
        }
        i += 1;
    }

    for (idx, (name, _, _)) in spec.args.iter().enumerate() {
        if let Some(v) = positional.get(idx) { out.values.insert((*name).to_string(), v.clone()); }
    }
    if positional.len() > spec.args.len() {
        return Err(Usage(format!("unexpected arguments for '{verb}': {}",
            positional[spec.args.len()..].join(" "))));
    }
    for (name, _, required) in spec.args {
        if *required && out.get(name).is_none() {
            return Err(Usage(format!("'{verb}' requires <{name}>")));
        }
    }
    for (name, _, required, _, _) in spec.flags {
        if *required && out.get(&flag_key(name)).is_none() {
            return Err(Usage(format!("'{verb}' requires {name}")));
        }
    }
    Ok(out)
}

async fn run_verb(verb: &str, a: &Args, network: &str, em: &Emitter) -> Result<i32, Usage> {
    match verb {
        "version" => return Ok(em.ok(json!({
            "sdk": SDK_NAME, "version": SDK_VERSION, "envelope": ENVELOPE_VERSION}))),
        "net list" => return Ok(em.ok(json!({"networks": [
            {"id": "kermit", "endpoint": "https://kermit.accumulatenetwork.io", "faucet": true, "default": true},
            {"id": "testnet", "endpoint": "https://testnet.accumulatenetwork.io", "faucet": true, "default": false},
            {"id": "mainnet", "endpoint": "https://mainnet.accumulatenetwork.io", "faucet": false,
             "default": false, "requiresOptIn": true},
            {"id": "local", "endpoint": "http://localhost:26660", "faucet": true, "default": false},
        ]}))),
        "keys generate" => {
            let algorithm = a.get("algorithm").unwrap_or("ed25519").to_lowercase();
            if algorithm != "ed25519" {
                return Err(Usage(format!("unsupported algorithm '{algorithm}' — only ed25519 is supported")));
            }
            // Uses the SDK's own derivation so the lite address carries its
            // checksum; an address missing it looks right and is rejected on chain.
            use accumulate_client::crypto::ed25519::Ed25519Signer;
            use accumulate_client::helpers::{derive_lite_identity_url, derive_lite_token_account_url};
            let kp = Ed25519Signer::generate();
            let pk = kp.public_key_bytes();
            return Ok(em.ok(json!({
                "algorithm": "ed25519",
                "publicKey": hex::encode(pk),
                "liteIdentity": derive_lite_identity_url(&pk),
                "liteTokenAccount": derive_lite_token_account_url(&pk),
            })));
        }
        "tx build" => {
            let mut params = Map::new();
            for raw in a.repeated.get("param").cloned().unwrap_or_default() {
                match raw.split_once('=') {
                    Some((k, v)) => { params.insert(k.to_string(), json!(v)); }
                    None => return Err(Usage(format!("--param must be key=value, got '{raw}'"))),
                }
            }
            return Ok(em.ok(json!({
                "op": a.get("op"), "params": Value::Object(params), "signed": false,
                "note": "unsigned body; sign and submit with `tx submit --envelope`"})));
        }
        _ => {}
    }

    let base = base_url(network)?;

    // V3 takes {"scope": <url>}. Verified against a live node.
    let query = |scope: String| rpc(base, "v3", "query", json!({"scope": scope}));

    match verb {
        "query" => {
            let url = a.get("url").unwrap().to_string();
            match query(url.clone()).await {
                Err(e) => Ok(em.fail(&e, None, None)),
                Ok(v) => match v.get("error") {
                    Some(err) => Ok(em.fail(&rpc_error_text(err), None, None)),
                    None => Ok(em.ok(json!({"url": url, "account": v.get("result")}))),
                },
            }
        }
        "balance" => {
            let url = a.get("url").unwrap().to_string();
            match query(url.clone()).await {
                Err(e) => Ok(em.fail(&e, None, None)),
                Ok(v) => match v.get("error") {
                    Some(err) => Ok(em.fail(&rpc_error_text(err), None, None)),
                    None => {
                        let balance = v.pointer("/result/account/balance").cloned();
                        Ok(em.ok(json!({"url": url, "balance": balance, "raw": v.get("result")})))
                    }
                },
            }
        }
        "chain" => {
            let url = a.get("url").unwrap().to_string();
            let params = json!({"scope": url, "query": {
                "queryType": "chain",
                "name": a.get("chain").unwrap_or("main"),
                "range": {"start": a.int("start"), "count": a.int("count")}}});
            match rpc(base, "v3", "query", params).await {
                Err(e) => Ok(em.fail(&e, None, None)),
                Ok(v) => match v.get("error") {
                    Some(err) => Ok(em.fail(&rpc_error_text(err), None, None)),
                    None => Ok(em.ok(json!({"url": url, "chain": a.get("chain"),
                        "start": a.int("start"), "count": a.int("count"), "entries": v.get("result")}))),
                },
            }
        }
        "faucet" => {
            let url = a.get("url").unwrap().to_string();
            match rpc(base, "v2", "faucet", json!({"url": url})).await {
                Err(e) => Ok(em.fail(&e, None, None)),
                Ok(v) => match v.get("error") {
                    Some(err) => Ok(em.fail(&rpc_error_text(err), None, None)),
                    None => Ok(em.ok(json!({"url": url, "result": v.get("result")}))),
                },
            }
        }
        "credits estimate" => {
            match query("acc://dn.acme/oracle".into()).await {
                Err(e) => Ok(em.fail(&e, None, None)),
                Ok(v) => Ok(em.ok(json!({
                    "url": a.get("url"), "acme": a.get("amount"),
                    "oracle": v.get("result"),
                    "note": "credits = acme * oraclePrice / 1e8 (oracle is unscaled)"}))),
            }
        }
        "tx status" => {
            let txid = a.get("txid").unwrap().to_string();
            match query(txid.clone()).await {
                Err(e) => Ok(em.fail(&e, None, None)),
                Ok(v) => match v.get("error") {
                    Some(err) => Ok(em.fail(&rpc_error_text(err), None, None)),
                    None => Ok(em.ok(json!({"txid": txid, "status": v.get("result")}))),
                },
            }
        }
        "tx wait" => {
            let txid = a.get("txid").unwrap().to_string();
            let timeout = a.int("timeout").max(1) as u64;
            let deadline = Instant::now() + Duration::from_secs(timeout);
            while Instant::now() < deadline {
                match query(txid.clone()).await {
                    Err(e) => return Ok(em.fail(&e, None, None)),
                    Ok(v) => {
                        if let Some(err) = v.get("error") {
                            return Ok(em.fail(&rpc_error_text(err), None, None));
                        }
                        let status = v.pointer("/result/status/code").and_then(|s| s.as_str()).map(String::from);
                        if matches!(status.as_deref(), Some("delivered") | Some("failed")) {
                            return Ok(em.ok(json!({"txid": txid, "final": true,
                                "status": status, "raw": v.get("result")})));
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Ok(em.fail(&format!("timed out waiting for {txid} to reach a final state"),
                Some("ACC_NETWORK_UNAVAILABLE"), Some(EXIT_FAILED)))
        }
        "net status" => {
            // A protocol rejection still proves the node answered, so only a
            // transport failure counts as unreachable — that is what exit 3 means.
            match query("acc://dn.acme".into()).await {
                Err(e) => Ok(em.fail(&e, Some("ACC_NETWORK_UNAVAILABLE"), Some(EXIT_NETWORK))),
                Ok(v) => Ok(em.ok(json!({"network": network, "endpoint": base, "reachable": true,
                    "probe": v.get("result"), "probeError": v.get("error").map(rpc_error_text)}))),
            }
        }
        "tx submit" => {
            if a.get("key_file").is_none() && a.get("key_env").is_none() {
                return Err(Usage("tx submit signs, so it requires --key-file or --key-env; \
                                  no ambient default key is ever used".into()));
            }
            let path = a.get("envelope").unwrap();
            let body = std::fs::read_to_string(path)
                .map_err(|e| Usage(format!("could not read envelope '{path}': {e}")))?;
            let envelope: Value = serde_json::from_str(&body)
                .map_err(|e| Usage(format!("envelope is not valid JSON: {e}")))?;
            match rpc(base, "v3", "submit", envelope).await {
                Err(e) => Ok(em.fail(&e, None, None)),
                Ok(v) => match v.get("error") {
                    Some(err) => Ok(em.fail(&rpc_error_text(err), None, None)),
                    None => Ok(em.ok(json!({"submitted": true, "result": v.get("result")}))),
                },
            }
        }
        other => Err(Usage(format!("unknown verb '{other}'"))),
    }
}

#[tokio::main]
async fn main() {
    let started = Instant::now();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let as_json = argv.iter().any(|a| a == "--json");

    let mut network = DEFAULT_NETWORK.to_string();
    let ni = argv.iter().position(|a| a == "--network");
    if let Some(i) = ni {
        match argv.get(i + 1) {
            Some(v) => network = v.clone(),
            None => {
                let em = Emitter { as_json, network: None, started };
                std::process::exit(em.fail("flag '--network' expects a value", Some("ACC_USAGE"), None));
            }
        }
    }

    let em = Emitter { as_json, network: Some(network.clone()), started };

    let tokens: Vec<String> = argv.iter().enumerate()
        .filter(|(i, a)| {
            *a != "--json" && Some(*i) != ni && ni.map_or(true, |n| *i != n + 1)
        })
        .map(|(_, a)| a.clone())
        .collect();

    let wants_help = tokens.iter().any(|t| t == "--help" || t == "-h");
    let verb_tokens: Vec<String> = tokens.into_iter()
        .filter(|t| t != "--help" && t != "-h" && t != "--version").collect();

    if wants_help || verb_tokens.is_empty() {
        if as_json {
            std::process::exit(em.ok(command_tree()));
        }
        println!("accumulate — Accumulate SDK CLI\n");
        for v in VERBS {
            println!("  {:<20} {}", v.name, v.summary);
        }
        println!("\nRun with --json --help for the machine-readable command tree.");
        std::process::exit(EXIT_OK);
    }

    let code = match parse_verb(&verb_tokens) {
        Err(Usage(m)) => em.fail(&m, Some("ACC_USAGE"), None),
        Ok((verb, rest)) => match parse_verb_args(verb, &rest) {
            Err(Usage(m)) => em.fail(&m, Some("ACC_USAGE"), None),
            Ok(a) => match run_verb(verb, &a, &network, &em).await {
                Err(Usage(m)) => em.fail(&m, Some("ACC_USAGE"), None),
                Ok(c) => c,
            },
        },
    };
    std::process::exit(code);
}
