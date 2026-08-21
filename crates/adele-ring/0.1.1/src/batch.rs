//! `RnsBatch` — the single flat buffer format shared by both backends.
//!
//! Having one canonical layout means there is **no reformatting** when switching
//! between the CPU (rayon) and GPU (wgpu) backends: residues are stored natively
//! as `u32` (every basis prime is `< 2^16`), so the GPU storage buffer *is* this
//! struct's `data` and the bytemuck cast is genuinely zero-copy.
//!
//! Layout is row-major `[batch_size × n_channels]`: element
//! `data[b * n_channels + c]` is the residue of item `b` in channel `c`.

use crate::basis::Basis;
use crate::rns::RnsInt;

/// A batch of RNS values in a flat, backend-agnostic `u32` buffer.
#[derive(Clone, Debug)]
pub struct RnsBatch {
    /// Flat row-major residues: length `batch_size * basis.len()`.
    pub data: Vec<u32>,
    /// Number of items `B`.
    pub batch_size: usize,
    /// The `K` channels shared by every item.
    pub basis: Basis,
}

impl RnsBatch {
    /// Allocate a zeroed batch (alias of [`RnsBatch::zeros`]).
    pub fn new(batch_size: usize, basis: Basis) -> Self {
        Self::zeros(batch_size, basis)
    }

    /// Allocate a batch with all residues set to zero.
    pub fn zeros(batch_size: usize, basis: Basis) -> Self {
        let k = basis.len();
        RnsBatch { data: vec![0; batch_size * k], batch_size, basis }
    }

    /// Number of channels `K`.
    #[inline]
    pub fn channels_len(&self) -> usize {
        self.basis.len()
    }

    /// Residue of item `b` in channel `c`.
    #[inline]
    pub fn get(&self, b: usize, c: usize) -> u32 {
        self.data[b * self.basis.len() + c]
    }

    /// Set the residue of item `b` in channel `c`.
    #[inline]
    pub fn set(&mut self, b: usize, c: usize, val: u32) {
        let k = self.basis.len();
        self.data[b * k + c] = val;
    }

    /// Pack a slice of [`RnsInt`] values into a batch.
    ///
    /// Panics if `items` is empty (no basis to infer) or if the items do not all
    /// share the same basis.
    pub fn from_rns_ints(items: &[RnsInt]) -> Self {
        assert!(!items.is_empty(), "cannot build an RnsBatch from zero items");
        let basis = items[0].basis.clone();
        let k = basis.len();
        let mut data = Vec::with_capacity(items.len() * k);
        for item in items {
            debug_assert_eq!(item.basis, basis, "all items must share the basis");
            debug_assert_eq!(item.residues.len(), k);
            data.extend_from_slice(&item.residues);
        }
        RnsBatch { data, batch_size: items.len(), basis }
    }

    /// Unpack the batch back into individual [`RnsInt`] values.
    pub fn to_rns_ints(&self) -> Vec<RnsInt> {
        let k = self.basis.len();
        (0..self.batch_size)
            .map(|b| {
                let residues = self.data[b * k..(b + 1) * k].to_vec();
                RnsInt::from_residues(residues, self.basis.clone())
            })
            .collect()
    }

    /// Pack for GPU upload: residues as little-endian `u32` bytes (zero-copy).
    pub fn as_u32_bytes(&self) -> Vec<u8> {
        bytemuck::cast_slice(&self.data).to_vec()
    }

    /// Rebuild a batch from a flat `u32` slice downloaded from the GPU.
    pub fn from_u32(values: &[u32], batch_size: usize, basis: Basis) -> Self {
        RnsBatch { data: values.to_vec(), batch_size, basis }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_roundtrip() {
        let b = Basis::standard();
        let mut batch = RnsBatch::zeros(4, b);
        batch.set(2, 3, 7);
        assert_eq!(batch.get(2, 3), 7);
        assert_eq!(batch.get(0, 0), 0);
    }

    #[test]
    fn pack_unpack_roundtrip() {
        let b = Basis::standard();
        let items = vec![
            RnsInt::from_i64(123, b.clone()),
            RnsInt::from_i64(456, b.clone()),
            RnsInt::from_i64(789, b.clone()),
        ];
        let batch = RnsBatch::from_rns_ints(&items);
        assert_eq!(batch.batch_size, 3);
        let back = batch.to_rns_ints();
        for (a, bb) in items.iter().zip(back.iter()) {
            assert_eq!(a.to_bigint(), bb.to_bigint());
        }
    }

    #[test]
    fn u32_byte_layout() {
        let b = Basis::standard();
        let mut batch = RnsBatch::zeros(1, b);
        batch.set(0, 0, 1);
        batch.set(0, 1, 2);
        let bytes = batch.as_u32_bytes();
        assert_eq!(bytes[0], 1);
        assert_eq!(bytes[4], 2);
    }
}
