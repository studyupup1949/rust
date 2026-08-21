// RNS elementwise multiplication: one GPU thread per (batch_item × channel).
// result[i] = (a[i] * b[i]) % moduli[c]
//
// WGSL has no u64. With the standard 32-prime channel set every modulus is
// <= 131 < 2^16, so the product of two residues is < 2^32 and fits in u32.
// If channels are ever extended past 65535 (see MAX_SAFE_MODULUS), switch to a
// hi/lo u32 emulation of the 64-bit product here.

struct Params {
    batch_size: u32,
    n_channels: u32,
};

@group(0) @binding(0) var<uniform>             params: Params;
@group(0) @binding(1) var<storage, read>       moduli: array<u32>;
@group(0) @binding(2) var<storage, read>       a_data: array<u32>;
@group(0) @binding(3) var<storage, read>       b_data: array<u32>;
@group(0) @binding(4) var<storage, read_write> result: array<u32>;

fn mul_mod(a: u32, b: u32, m: u32) -> u32 {
    // Safe while m < 2^16 (product < 2^32).
    return (a * b) % m;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let b_idx = gid.x;
    let c     = gid.y;

    if (b_idx >= params.batch_size || c >= params.n_channels) {
        return;
    }

    let i = b_idx * params.n_channels + c;
    result[i] = mul_mod(a_data[i], b_data[i], moduli[c]);
}
