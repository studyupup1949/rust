// RNS elementwise subtraction: one GPU thread per (batch_item × channel).
// result[i] = (a[i] + m - b[i]) % m
//
// Balanced residues need no sign-magnitude branch: adding m before the modulo
// keeps the intermediate non-negative (a, b < m < 2^16, so a + m < 2^17 fits u32).

struct Params {
    batch_size: u32,
    n_channels: u32,
};

@group(0) @binding(0) var<uniform>             params: Params;
@group(0) @binding(1) var<storage, read>       moduli: array<u32>;
@group(0) @binding(2) var<storage, read>       a_data: array<u32>;
@group(0) @binding(3) var<storage, read>       b_data: array<u32>;
@group(0) @binding(4) var<storage, read_write> result: array<u32>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let b_idx = gid.x; // batch item
    let c     = gid.y; // channel

    if (b_idx >= params.batch_size || c >= params.n_channels) {
        return;
    }

    let i = b_idx * params.n_channels + c;
    let m = moduli[c];

    result[i] = (a_data[i] + m - b_data[i]) % m;
}
