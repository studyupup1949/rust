use acton_core::{Crystal, CrystalConfig};

fn main() {
    let mut crystal = Crystal::new(b"test_seed", CrystalConfig::default());
    let message = b"Hello, Acton Core!";
    let coords = crystal.encode(message);
    let decoded = crystal.decode(&coords, message.len()).unwrap();
    println!("Original: {:?}", String::from_utf8_lossy(message));
    println!("Decoded:  {:?}", String::from_utf8_lossy(&decoded));
}