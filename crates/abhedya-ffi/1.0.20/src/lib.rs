use std::ffi::{c_int, c_uchar};
use std::slice;
use abhedya_core::{SecretKey, PublicKey, EncryptionMode, encrypt, decrypt};

// Define Encryption Mode for C
#[repr(C)]
pub enum EncryptionModeC {
    Standard = 0,
    Metered = 1,
}

// Helper to serialize SecretKey (s: Vec<i16>) -> Bytes
fn serialize_sk(sk: &SecretKey, buffer: *mut c_uchar, len: *mut usize) -> c_int {
    let raw_len = sk.s.len() * 2;
    // Safety: Caller must provide valid pointers. We check lengths.
    unsafe {
        if *len < raw_len {
            *len = raw_len;
            return -1; // Buffer too small
        }
        let dest = slice::from_raw_parts_mut(buffer as *mut u8, raw_len);
        // Little endian serialization
        for (i, &val) in sk.s.iter().enumerate() {
            let bytes = val.to_le_bytes();
            dest[i*2] = bytes[0];
            dest[i*2 + 1] = bytes[1];
        }
        *len = raw_len;
    }
    0
}

// Helper to serialize PublicKey (A: NxN, b: N) -> Bytes
fn serialize_pk(pk: &PublicKey, buffer: *mut c_uchar, len: *mut usize) -> c_int {
    // N=768. A is 768x768. b is 768.
    let n = pk.b.len();
    let total_elements = n * n + n;
    let raw_len = total_elements * 2;
    
    // Safety: Caller must provide valid pointers.
    unsafe {
        if *len < raw_len {
            *len = raw_len;
            return -1;
        }
        let dest = slice::from_raw_parts_mut(buffer as *mut u8, raw_len);
        let mut cursor = 0;
        
        // Serialize A (Row-Major)
        for row in &pk.A {
            for &val in row {
                let bytes = val.to_le_bytes();
                dest[cursor] = bytes[0];
                dest[cursor+1] = bytes[1];
                cursor += 2;
            }
        }
        
        // Serialize b
        for &val in &pk.b {
            let bytes = val.to_le_bytes();
            dest[cursor] = bytes[0];
            dest[cursor+1] = bytes[1];
            cursor += 2;
        }
        *len = raw_len;
    }
    0
}

/// Generate a KeyPair.
/// Returns 0 on success, -1 on buffer too small.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn abhedya_keygen(
    pk_buffer: *mut c_uchar, pk_len: *mut usize,
    sk_buffer: *mut c_uchar, sk_len: *mut usize
) -> c_int {
    let mut rng = rand::rng();
    let sk = SecretKey::new(&mut rng);
    let pk = PublicKey::new(&sk, &mut rng);
    
    let sk_res = serialize_sk(&sk, sk_buffer, sk_len);
    if sk_res != 0 { return sk_res; }
    
    let pk_res = serialize_pk(&pk, pk_buffer, pk_len);
    if pk_res != 0 { return pk_res; }
    
    0
}

/// Encrypt a message.
/// Input: Raw byte array. Internally converted to bit-polynomial.
/// Output format: [u_vecs (flattened) | v_vec]
/// Size per bit = (768 + 1) * 2 bytes = 1538 bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn abhedya_encrypt(
    pk_buffer: *const c_uchar, pk_len: usize,
    msg_ptr: *const c_uchar, msg_len: usize,
    mode: EncryptionModeC,
    out_buffer: *mut c_uchar, out_len: *mut usize
) -> c_int {
    // 1. Deserialize PK
    let n = 768;
    let expected_pk_size = (n * n + n) * 2;
    if pk_len < expected_pk_size { return -2; } // Invalid PK
    
    let pk_bytes = unsafe { slice::from_raw_parts(pk_buffer, pk_len) };
    let mut a = Vec::with_capacity(n);
    let mut cursor = 0;
    
    // Read A
    for _ in 0..n {
        let mut row = Vec::with_capacity(n);
        for _ in 0..n {
            let val = i16::from_le_bytes([pk_bytes[cursor], pk_bytes[cursor+1]]);
            row.push(val);
            cursor += 2;
        }
        a.push(row);
    }
    // Read b
    let mut b = Vec::with_capacity(n);
    for _ in 0..n {
        let val = i16::from_le_bytes([pk_bytes[cursor], pk_bytes[cursor+1]]);
        b.push(val);
        cursor += 2;
    }
    
    let pk = PublicKey { A: a, b };
    
    // 2. Prepare Message
    // Convert bytes to bits for encryption
    let msg_bytes = unsafe { slice::from_raw_parts(msg_ptr, msg_len) };
    let mut m_poly = Vec::new();
    for &byte in msg_bytes {
        for i in 0..8 {
            m_poly.push(((byte >> i) & 1) as i16);
        }
    }
    
    // 3. Encrypt
    let mut rng = rand::rng();
    let rust_mode = match mode {
        EncryptionModeC::Standard => EncryptionMode::Standard,
        EncryptionModeC::Metered => EncryptionMode::Metered,
    };
    
    let (u_vecs, v_vec) = encrypt(&pk, &m_poly, &mut rng, rust_mode);
    
    // 4. Serialize Ciphertext
    // u: Vec<Vec<i16>>, v: Vec<i16>
    let bit_count = m_poly.len();
    let required_len = bit_count * (n + 1) * 2; 
    
    unsafe {
        if *out_len < required_len {
            *out_len = required_len;
            return -1;
        }
    }
    
    let dest = unsafe { slice::from_raw_parts_mut(out_buffer, required_len) };
    let mut out_cursor = 0;
    
    for (u, v_val) in u_vecs.iter().zip(v_vec.iter()) {
        // Write u
        for &val in u {
            let bytes = val.to_le_bytes();
            dest[out_cursor] = bytes[0];
            dest[out_cursor+1] = bytes[1];
            out_cursor += 2;
        }
        // Write v
        let bytes = v_val.to_le_bytes();
        dest[out_cursor] = bytes[0];
        dest[out_cursor+1] = bytes[1];
        out_cursor += 2;
    }
    
    unsafe { *out_len = required_len; }
    0
}

/// Decrypt a message.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn abhedya_decrypt(
    sk_buffer: *const c_uchar, sk_len: usize,
    ct_buffer: *const c_uchar, ct_len: usize,
    out_buffer: *mut c_uchar, out_len: *mut usize
) -> c_int {
    // 1. Deserialize SK
    let n = 768;
    if sk_len < n * 2 { return -2; }
    
    let sk_bytes = unsafe { slice::from_raw_parts(sk_buffer, sk_len) };
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let val = i16::from_le_bytes([sk_bytes[i*2], sk_bytes[i*2+1]]);
        s.push(val);
    }
    let sk = SecretKey { s };
    
    // 2. Deserialize Ciphertext
    // Block size = (768 + 1) * 2 = 1538 bytes
    let block_size = (n + 1) * 2;
    if ct_len % block_size != 0 { return -3; } 
    
    let num_bits = ct_len / block_size;
    let ct_bytes = unsafe { slice::from_raw_parts(ct_buffer, ct_len) };
    let mut u_vecs = Vec::with_capacity(num_bits);
    let mut v_vec = Vec::with_capacity(num_bits);
    
    let mut cursor = 0;
    for _ in 0..num_bits {
        // Read u
        let mut u = Vec::with_capacity(n);
        for _ in 0..n {
            let val = i16::from_le_bytes([ct_bytes[cursor], ct_bytes[cursor+1]]);
            u.push(val);
            cursor += 2;
        }
        u_vecs.push(u);
        
        // Read v
        let v_val = i16::from_le_bytes([ct_bytes[cursor], ct_bytes[cursor+1]]);
        v_vec.push(v_val);
        cursor += 2;
    }
    
    // 3. Decrypt
    let msg_poly = decrypt(&sk, &u_vecs, &v_vec);
    
    // 4. Decode Bits to Bytes
    let num_bytes = (num_bits + 7) / 8;
    unsafe {
        if *out_len < num_bytes {
            *out_len = num_bytes;
            return -1;
        }
    }
    
    let dest = unsafe { slice::from_raw_parts_mut(out_buffer, num_bytes) };
    // Zero out first
    for i in 0..num_bytes { dest[i] = 0; }
    
    for (i, &bit) in msg_poly.iter().enumerate() {
        if bit == 1 {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            dest[byte_idx] |= 1 << bit_idx;
        }
    }
    
    unsafe { *out_len = num_bytes; }
    0
}
