//! `CpuBackend` — the always-available backend, using rayon to parallelize over
//! batch items (and, for single values, over channels above a threshold).

use num_bigint::BigUint;
use rayon::prelude::*;

use crate::backend::ArithmeticBackend;
use crate::batch::RnsBatch;
use crate::rns::{garner_crt, gpu_mul_channel, RnsInt};
use crate::RAYON_CHANNEL_THRESHOLD;

/// CPU backend backed by a dedicated rayon thread pool.
pub struct CpuBackend {
    pool: rayon::ThreadPool,
}

impl Default for CpuBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuBackend {
    /// Build a backend with a thread pool spanning all logical cores.
    pub fn new() -> Self {
        Self {
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(0) // 0 = use all logical cores
                .thread_name(|i| format!("adele-ring-cpu-{i}"))
                .build()
                .expect("rayon pool init failed"),
        }
    }

    /// Single-value addition: parallel over channels only when `k` is large.
    ///
    /// Below [`RAYON_CHANNEL_THRESHOLD`] channels, rayon's per-task overhead
    /// (~50ns) exceeds the cost of a channel op (~1ns), so we stay sequential.
    pub fn rns_add_single(&self, a: &RnsInt, b: &RnsInt) -> RnsInt {
        let moduli = a.channels.moduli();
        let k = a.channels.len();
        let residues: Vec<u64> = if k >= RAYON_CHANNEL_THRESHOLD {
            self.pool.install(|| {
                a.residues
                    .par_iter()
                    .zip(b.residues.par_iter())
                    .zip(moduli.par_iter())
                    .map(|((&av, &bv), &m)| (av + bv) % m)
                    .collect()
            })
        } else {
            a.residues
                .iter()
                .zip(b.residues.iter())
                .zip(moduli.iter())
                .map(|((&av, &bv), &m)| (av + bv) % m)
                .collect()
        };
        RnsInt::from_residues(residues, a.channels.clone())
    }
}

impl ArithmeticBackend for CpuBackend {
    fn batch_rns_add(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
        let k = a.channels.len();
        let moduli = a.channels.moduli();
        let mut result = RnsBatch::zeros(a.batch_size, a.channels.clone());
        self.pool.install(|| {
            result
                .data
                .par_chunks_mut(k)
                .enumerate()
                .for_each(|(b_idx, out_row)| {
                    let base = b_idx * k;
                    for c in 0..k {
                        let av = a.data[base + c];
                        let bv = b.data[base + c];
                        out_row[c] = (av + bv) % moduli[c];
                    }
                });
        });
        result
    }

    fn batch_rns_mul(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
        let k = a.channels.len();
        let moduli = a.channels.moduli();
        let mut result = RnsBatch::zeros(a.batch_size, a.channels.clone());
        self.pool.install(|| {
            result
                .data
                .par_chunks_mut(k)
                .enumerate()
                .for_each(|(b_idx, out_row)| {
                    let base = b_idx * k;
                    for c in 0..k {
                        let av = a.data[base + c];
                        let bv = b.data[base + c];
                        out_row[c] = gpu_mul_channel(av, bv, moduli[c]);
                    }
                });
        });
        result
    }

    fn batch_crt(&self, batch: &RnsBatch) -> Vec<BigUint> {
        let k = batch.channels.len();
        let moduli = batch.channels.moduli();
        self.pool.install(|| {
            (0..batch.batch_size)
                .into_par_iter()
                .map(|b| {
                    let residues = &batch.data[b * k..(b + 1) * k];
                    garner_crt(residues, moduli)
                })
                .collect()
        })
    }

    fn name(&self) -> &'static str {
        "cpu-rayon"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rns::Channels;

    #[test]
    fn batch_add_matches_scalar() {
        let ch = Channels::standard(32);
        let a = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, ch.clone()); 64]);
        let b = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, ch.clone()); 64]);
        let cpu = CpuBackend::new();
        let sum = cpu.batch_rns_add(&a, &b);
        for item in sum.to_rns_ints() {
            assert_eq!(item.to_bigint(), num_bigint::BigInt::from(579));
        }
    }

    #[test]
    fn batch_mul_matches_scalar() {
        let ch = Channels::standard(32);
        let a = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, ch.clone()); 64]);
        let b = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, ch.clone()); 64]);
        let cpu = CpuBackend::new();
        let prod = cpu.batch_rns_mul(&a, &b);
        for item in prod.to_rns_ints() {
            assert_eq!(item.to_bigint(), num_bigint::BigInt::from(123 * 456));
        }
    }

    #[test]
    fn single_add() {
        let ch = Channels::standard(32);
        let cpu = CpuBackend::new();
        let r = cpu.rns_add_single(&RnsInt::from_i64(10, ch.clone()), &RnsInt::from_i64(32, ch));
        assert_eq!(r.to_bigint(), num_bigint::BigInt::from(42));
    }
}
