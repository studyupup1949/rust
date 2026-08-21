//! The [`ArithmeticBackend`] trait and the [`Executor`] that selects between CPU
//! and GPU at runtime. All batch math in the crate flows through the Executor —
//! it never hard-codes a backend.

use std::sync::{Arc, OnceLock};

use num_bigint::BigUint;

use crate::batch::RnsBatch;
use crate::cpu::CpuBackend;
use crate::gpu::GpuBackend;

/// A backend that can perform elementwise RNS arithmetic over a [`RnsBatch`].
pub trait ArithmeticBackend: Send + Sync {
    /// Elementwise add: `result[b][c] = (a[b][c] + b[b][c]) % m[c]`.
    fn batch_rns_add(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch;

    /// Elementwise multiply: `result[b][c] = (a[b][c] * b[b][c]) % m[c]`.
    fn batch_rns_mul(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch;

    /// CRT-reconstruct every item in the batch.
    fn batch_crt(&self, batch: &RnsBatch) -> Vec<BigUint>;

    /// Backend name for diagnostics.
    fn name(&self) -> &'static str;
}

/// Runtime dispatcher between the CPU and (optional) GPU backends.
pub struct Executor {
    cpu: Arc<CpuBackend>,
    gpu: Option<Arc<GpuBackend>>,
    /// Batches smaller than this use the CPU even when a GPU is present, because
    /// the GPU's upload/dispatch/download round-trip (~100µs) dominates for small
    /// inputs. Public so callers can tune it for their hardware.
    pub gpu_threshold: usize,
}

impl Executor {
    /// Probe for a GPU and build the executor. This is the only constructor.
    pub fn init() -> Self {
        let cpu = Arc::new(CpuBackend::new());
        let gpu = GpuBackend::try_init().ok().map(Arc::new);
        match &gpu {
            Some(g) => log::info!("adele-ring: GPU backend active ({})", g.adapter_name()),
            None => log::info!("adele-ring: no GPU found, using CPU backend"),
        }
        Self {
            cpu,
            gpu,
            gpu_threshold: 128,
        }
    }

    /// Whether a GPU backend is available.
    pub fn has_gpu(&self) -> bool {
        self.gpu.is_some()
    }

    /// Borrow the CPU backend directly (used by benchmarks).
    pub fn cpu(&self) -> &CpuBackend {
        &self.cpu
    }

    /// Borrow the GPU backend if present (used by benchmarks).
    pub fn gpu(&self) -> Option<&GpuBackend> {
        self.gpu.as_deref()
    }

    /// Pick the best backend for a given batch size.
    fn select(&self, batch_size: usize) -> &dyn ArithmeticBackend {
        match &self.gpu {
            Some(g) if batch_size >= self.gpu_threshold => g.as_ref(),
            _ => self.cpu.as_ref(),
        }
    }

    /// Elementwise batch addition.
    pub fn add(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
        self.select(a.batch_size).batch_rns_add(a, b)
    }

    /// Elementwise batch multiplication.
    pub fn mul(&self, a: &RnsBatch, b: &RnsBatch) -> RnsBatch {
        self.select(a.batch_size).batch_rns_mul(a, b)
    }

    /// CRT reconstruction — always CPU-side (Garner's algorithm is sequential).
    pub fn crt(&self, batch: &RnsBatch) -> Vec<BigUint> {
        self.cpu.batch_crt(batch)
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::init()
    }
}

static EXECUTOR: OnceLock<Executor> = OnceLock::new();

/// The lazily-initialized, crate-wide executor.
pub fn executor() -> &'static Executor {
    EXECUTOR.get_or_init(Executor::init)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rns::{Channels, RnsInt};

    #[test]
    fn executor_adds_batches() {
        let ch = Channels::standard(32);
        let a = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(7, ch.clone()); 200]);
        let b = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(35, ch.clone()); 200]);
        let sum = executor().add(&a, &b);
        for item in sum.to_rns_ints() {
            assert_eq!(item.to_bigint(), num_bigint::BigInt::from(42));
        }
    }

    #[test]
    fn cpu_gpu_identical() {
        let exec = Executor::init();
        if !exec.has_gpu() {
            return;
        }
        let ch = Channels::standard(32);
        let a = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(123, ch.clone()); 256]);
        let b = RnsBatch::from_rns_ints(&vec![RnsInt::from_i64(456, ch.clone()); 256]);
        let cpu = exec.cpu().batch_rns_add(&a, &b);
        let gpu = exec.gpu().unwrap().batch_rns_add(&a, &b);
        assert_eq!(cpu.data, gpu.data);
    }
}
