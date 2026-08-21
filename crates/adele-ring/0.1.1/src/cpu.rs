//! `CpuBackend` — the always-available backend, using rayon to parallelize over
//! batch items (and, for single values, over channels above a threshold).

use num_bigint::BigInt;
use rayon::prelude::*;

use crate::backend::ArithmeticBackend;
use crate::batch::RnsBatch;
use crate::rns::{add_channel, crt_balanced, mul_channel, sub_channel, RnsInt};
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
        let moduli = a.basis.moduli();
        let k = a.basis.len();
        let residues: Vec<u32> = if k >= RAYON_CHANNEL_THRESHOLD {
            self.pool.install(|| {
                a.residues
                    .par_iter()
                    .zip(b.residues.par_iter())
                    .zip(moduli.par_iter())
                    .map(|((&av, &bv), &m)| add_channel(av, bv, m))
                    .collect()
            })
        } else {
            a.residues
                .iter()
                .zip(b.residues.iter())
                .zip(moduli.iter())
                .map(|((&av, &bv), &m)| add_channel(av, bv, m))
                .collect()
        };
        RnsInt::from_residues(residues, a.basis.clone())
    }

    fn elementwise(
        &self,
        a: &RnsBatch,
        b: &RnsBatch,
        f: impl Fn(u32, u32, u32) -> u32 + Sync + Send,
    ) -> RnsBatch {
        let k = a.basis.len();
        let moduli = a.basis.moduli();
        let mut result = RnsBatch::zeros(a.batch_size, a.basis.clone());
        self.pool.install(|| {
            result
                .data
                .par_chunks_mut(k)
                .enumerate()
                .for_each(|(b_idx, out_row)| {
                    let base = b_idx * k;
                    for c in 0..k {
                        out_row[c] = f(a.data[base + c], b.data[base + c], moduli[c]);
                    }
                });
        });
        result
    }
}

impl ArithmeticBackend for CpuBackend {
    fn batch_add(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
        self.elementwise(a, b, add_channel)
    }

    fn batch_sub(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
        self.elementwise(a, b, sub_channel)
    }

    fn batch_mul(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
        self.elementwise(a, b, mul_channel)
    }

    fn batch_crt(&self, batch: &RnsBatch) -> Vec<BigInt> {
        let k = batch.basis.len();
        let moduli = batch.basis.moduli();
        self.pool.install(|| {
            (0..batch.batch_size)
                .into_par_iter()
                .map(|b| crt_balanced(&batch.data[b * k..(b + 1) * k], moduli))
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
    use crate::basis::Basis;

    #[test]
    fn batch_add_matches_scalar() {
        let b = Basis::standard();
        let x = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, b.clone()); 64]);
        let y = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, b.clone()); 64]);
        let cpu = CpuBackend::new();
        for item in cpu.batch_add(&x, &y).to_rns_ints() {
            assert_eq!(item.to_bigint(), BigInt::from(579));
        }
    }

    #[test]
    fn batch_sub_matches_scalar() {
        let b = Basis::standard();
        let x = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, b.clone()); 64]);
        let y = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, b.clone()); 64]);
        let cpu = CpuBackend::new();
        for item in cpu.batch_sub(&x, &y).to_rns_ints() {
            assert_eq!(item.to_bigint(), BigInt::from(-333));
        }
    }

    #[test]
    fn batch_mul_matches_scalar() {
        let b = Basis::standard();
        let x = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, b.clone()); 64]);
        let y = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, b.clone()); 64]);
        let cpu = CpuBackend::new();
        for item in cpu.batch_mul(&x, &y).to_rns_ints() {
            assert_eq!(item.to_bigint(), BigInt::from(123 * 456));
        }
    }

    #[test]
    fn single_add() {
        let b = Basis::standard();
        let cpu = CpuBackend::new();
        let r = cpu.rns_add_single(&RnsInt::from_i64(10, b.clone()), &RnsInt::from_i64(32, b));
        assert_eq!(r.to_bigint(), BigInt::from(42));
    }

    #[test]
    fn batch_crt_is_balanced() {
        let b = Basis::standard();
        let x = RnsBatch::from_rns_ints(&[RnsInt::from_i64(-5, b.clone()), RnsInt::from_i64(9, b)]);
        let cpu = CpuBackend::new();
        let crt = cpu.batch_crt(&x);
        assert_eq!(crt, vec![BigInt::from(-5), BigInt::from(9)]);
    }
}
