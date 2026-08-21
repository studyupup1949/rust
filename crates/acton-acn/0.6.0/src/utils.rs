pub fn generate_nonce() -> [u8; 24] {
    rand::random()
}

pub fn generate_session_id() -> String {
    hex::encode(rand::random::<[u8; 16]>())
}