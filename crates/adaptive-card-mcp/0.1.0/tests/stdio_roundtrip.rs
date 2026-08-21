//! End-to-end test: spawn the `adaptive-card-mcp` binary as a child process,
//! send a JSON-RPC `initialize` + `tools/list` + `tools/call validate_card`
//! sequence over stdio, and assert that the responses are well-formed.

use assert_cmd::cargo::CommandCargoExt;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn send_line(stdin: &mut std::process::ChildStdin, msg: &Value) {
    writeln!(stdin, "{msg}").expect("write json-rpc line");
    stdin.flush().expect("flush stdin");
}

fn next_response(reader: &mut BufReader<std::process::ChildStdout>) -> Value {
    let mut line = String::new();
    let n = reader.read_line(&mut line).expect("read stdout line");
    assert!(n > 0, "server closed stdout unexpectedly");
    serde_json::from_str(line.trim())
        .unwrap_or_else(|e| panic!("server produced non-JSON output: {line:?}: {e}"))
}

#[test]
fn stdio_initialize_list_and_call_validate_card() {
    let mut child = Command::cargo_bin("adaptive-card-mcp")
        .expect("cargo binary")
        .env("TRANSPORT", "stdio")
        .env("RUST_LOG", "warn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn adaptive-card-mcp");

    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    // 1. initialize
    let init = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "stdio-roundtrip-test", "version": "0.0.0" }
        }
    });
    send_line(&mut stdin, &init);
    let resp = next_response(&mut reader);
    assert_eq!(resp["id"], 1, "init response id mismatch: {resp}");
    assert!(
        resp["result"]["serverInfo"]["name"] == "adaptive-card-mcp",
        "unexpected server name: {resp}"
    );
    assert!(
        resp["result"]["capabilities"]["tools"].is_object(),
        "server should advertise tools capability: {resp}"
    );

    // 2. notifications/initialized
    let notif = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
    send_line(&mut stdin, &notif);

    // 3. tools/list
    let list = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    send_line(&mut stdin, &list);
    let resp = next_response(&mut reader);
    assert_eq!(resp["id"], 2, "tools/list response id mismatch: {resp}");
    let tools = resp["result"]["tools"]
        .as_array()
        .expect("tools array in result");
    assert_eq!(
        tools.len(),
        10,
        "expected 10 tools, got {}: {resp}",
        tools.len()
    );
    assert!(
        tools.iter().any(|t| t["name"] == "validate_card"),
        "validate_card missing from tools list"
    );

    // 4. tools/call validate_card
    let call = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "validate_card",
            "arguments": {
                "card": {
                    "type": "AdaptiveCard",
                    "version": "1.6",
                    "speak": "hi",
                    "body": [
                        { "type": "TextBlock", "text": "Hi", "wrap": true }
                    ]
                },
                "host": "teams"
            }
        }
    });
    send_line(&mut stdin, &call);
    let resp = next_response(&mut reader);
    assert_eq!(resp["id"], 3, "tools/call response id mismatch: {resp}");
    let structured = &resp["result"]["structuredContent"];
    assert_eq!(
        structured["valid"], true,
        "validate_card should report valid: {resp}"
    );

    // clean shutdown
    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();
}
