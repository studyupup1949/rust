use acton_core::{ActonClient, TcpTransport, Transport};
use std::{thread, time::Duration};

fn main() {
    let mut alice_tcp = TcpTransport::new("alice", 8000);
    alice_tcp.start().unwrap();
    let mut bob_tcp = TcpTransport::new("bob", 8001);
    bob_tcp.start().unwrap();
    bob_tcp.connect("127.0.0.1:8000").unwrap();
    alice_tcp.connect("127.0.0.1:8001").unwrap();
    let mut alice = ActonClient::new("alice seed", Box::new(alice_tcp));
    let mut bob = ActonClient::new("bob seed", Box::new(bob_tcp));
    let session = alice.initiate_session(&bob.identity().public_id).unwrap();
    let session_id = session.session_id();
    let shared_secret = b"test_shared_secret_32_bytes_long_!!";
    let _ = bob.accept_session(&alice.identity().public_id, shared_secret).unwrap();
    alice.send(&session_id, b"TCP message from Alice to Bob").unwrap();
    thread::sleep(Duration::from_millis(100));
    match bob.receive() {
        Ok(messages) => {
            for (_from, msg) in messages {
                println!("Bob via TCP: {}", String::from_utf8_lossy(&msg));
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}