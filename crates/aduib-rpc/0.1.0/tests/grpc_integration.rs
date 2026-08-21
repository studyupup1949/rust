use std::net::{SocketAddr, TcpListener};
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use aduib_rpc::{AduibRpcClientBuilder, TransportKind};
use serde_json::json;

fn pick_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.local_addr().unwrap().port()
}

fn python_exe() -> std::ffi::OsString {
    std::env::var_os("ADUIB_RPC_PYTHON")
        .or_else(|| std::env::var_os("PYTHON"))
        .unwrap_or_else(|| std::ffi::OsString::from("python"))
}

fn python_has_grpc() -> bool {
    Command::new(python_exe())
        .arg("-c")
        .arg("import grpc")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn wait_for_port_or_child_exit(child: &mut Child, addr: SocketAddr, timeout: Duration) {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if std::net::TcpStream::connect(addr).is_ok() {
            return;
        }

        if let Ok(Some(status)) = child.try_wait() {
            let mut stderr = String::new();
            if let Some(mut s) = child.stderr.take() {
                let _ = s.read_to_string(&mut stderr);
            }
            panic!(
                "grpc python server exited early ({status}). stderr:\n{stderr}"
            );
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    let _ = child.kill();
    let _ = child.wait();
    let mut stderr = String::new();
    if let Some(mut s) = child.stderr.take() {
        let _ = s.read_to_string(&mut stderr);
    }
    panic!("server did not open port {addr} in time. stderr:\n{stderr}");
}

fn spawn_python_grpc_server(port: u16) -> Child {
    // Use the repository-root python module. We assume tests run from rust-sdk/.
    // Prefer `python` from PATH.
    let mut cmd = Command::new(python_exe());

    cmd.arg("-m")
        .arg("tests.grpc_server_local")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    // Set cwd to repo root so `-m tests.grpc_server_local` resolves.
    cmd.current_dir(repo_root());

    cmd.spawn().expect("spawn python grpc server")
}

fn repo_root() -> std::path::PathBuf {
    // rust-sdk/crates/aduib-rpc -> rust-sdk -> repo root
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("resolve repo root")
        .to_path_buf()
}

#[tokio::test]
async fn grpc_unary_completion_end_to_end() {
    if !python_has_grpc() {
        eprintln!("skipping gRPC integration test: python module 'grpc' not installed (try `pip install grpcio`).");
        return;
    }

    let port = pick_free_port();
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    let mut child = spawn_python_grpc_server(port);
    wait_for_port_or_child_exit(&mut child, addr, Duration::from_secs(10));

    let client = AduibRpcClientBuilder::new(format!("127.0.0.1:{port}"))
        .transport(TransportKind::Grpc)
        .build()
        .expect("build client");

    let resp = client
        .completion("CaculService.add", Some(json!({"x": 1, "y": 2})), None)
        .await
        .expect("grpc completion");

    assert!(resp.is_success());
    assert_eq!(resp.result.unwrap(), json!(3));

    // Best-effort cleanup.
    let _ = child.kill();
    let _ = child.wait();
}

