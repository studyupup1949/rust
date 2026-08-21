//! `RnsBatch` — the single flat buffer format shared by both backends.
//!
//! Having one canonical layout means there is **no reformatting** when switching
//! between the CPU (rayon) and GPU (wgpu) backends: the GPU storage buffer *is*
//! this struct's `data`, and the CPU just iterates it in place.
//!
//! Layout is row-major `[batch_size × n_channels]`: element
//! `data[b * n_channels + c]` is the residue of item `b` in channel `c`.

use crate::rns::{Channels, RnsInt};

/// A batch of RNS values in a flat, backend-agnostic buffer.
#[derive(Clone, Debug)]
pub struct RnsBatch {
    /// Flat row-major residues: length `batch_size * channels.len()`.
    pub data: Vec<u64>,
    /// Number of items `B`.
    pub batch_size: usize,
    /// The `K` channels shared by every item.
    pub channels: Channels,
}

impl RnsBatch {
    /// Allocate a zeroed batch (alias of [`RnsBatch::zeros`]).
    pub fn new(batch_size: usize, channels: Channels) -> Self {
        Self::zeros(batch_size, channels)
    }

    /// Allocate a batch with all residues set to zero.
    pub fn zeros(batch_size: usize, channels: Channels) -> Self {
        let k = channels.len();
        RnsBatch {
            data: vec![0; batch_size * k],
            batch_size,
            channels,
        }
    }

    /// Number of channels `K`.
    #[inline]
    pub fn channels_len(&self) -> usize {
        self.channels.len()
    }

    /// Residue of item `b` in channel `c`.
    #[inline]
    pub fn get(&self, b: usize, c: usize) -> u64 {
        self.data[b * self.channels.len() + c]
    }

    /// Set the residue of item `b` in channel `c`.
    #[inline]
    pub fn set(&mut self, b: usize, c: usize, val: u64) {
        let k = self.channels.len();
        self.data[b * k + c] = val;
    }

    /// Pack a slice of [`RnsInt`] values into a batch.
    ///
    /// Panics if `items` is empty (no channels to infer) or if the items do not
    /// all share the same channels.
    pub fn from_rns_ints(items: &[RnsInt]) -> Self {
        assert!(!items.is_empty(), "cannot build an RnsBatch from zero items");
        let channels = items[0].channels.clone();
        let k = channels.len();
        let mut data = Vec::with_capacity(items.len() * k);
        for item in items {
            debug_assert_eq!(item.channels, channels, "all items must share channels");
            debug_assert_eq!(item.residues.len(), k);
            data.extend_from_slice(&item.residues);
        }
        RnsBatch {
            data,
            batch_size: items.len(),
            channels,
        }
    }

    /// Unpack the batch back into individual [`RnsInt`] values.
    pub fn to_rns_ints(&self) -> Vec<RnsInt> {
        let k = self.channels.len();
        (0..self.batch_size)
            .map(|b| {
                let residues = self.data[b * k..(b + 1) * k].to_vec();
                RnsInt::from_residues(residues, self.channels.clone())
            })
            .collect()
    }

    /// Pack for GPU upload: residues as little-endian `u32` bytes.
    ///
    /// Moduli are chosen `<= 2^31` so every residue fits in a `u32`. The cast to
    /// bytes is zero-copy via bytemuck.
    pub fn as_u32_bytes(&self) -> Vec<u8> {
        let as_u32: Vec<u32> = self.data.iter().map(|&v| v as u32).collect();
        bytemuck::cast_slice(&as_u32).to_vec()
    }

    /// Rebuild a batch from a flat `u32` slice downloaded from the GPU.
    pub fn from_u32(values: &[u32], batch_size: usize, channels: Channels) -> Self {
        let data = values.iter().map(|&v| v as u64).collect();
        RnsBatch {
            data,
            batch_size,
            channels,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_roundtrip() {
        let ch = Channels::standard(8);
        let mut batch = RnsBatch::zeros(4, ch);
        batch.set(2, 3, 7);
        assert_eq!(batch.get(2, 3), 7);
        assert_eq!(batch.get(0, 0), 0);
    }

    #[test]
    fn pack_unpack_roundtrip() {
        let ch = Channels::standard(16);
        let items = vec![
            RnsInt::from_i64(123, ch.clone()),
            RnsInt::from_i64(456, ch.clone()),
            RnsInt::from_i64(789, ch.clone()),
        ];
        let batch = RnsBatch::from_rns_ints(&items);
        assert_eq!(batch.batch_size, 3);
        let back = batch.to_rns_ints();
        for (a, b) in items.iter().zip(back.iter()) {
            assert_eq!(a.to_bigint(), b.to_bigint());
        }
    }

    #[test]
    fn u32_byte_layout() {
        let ch = Channels::standard(4);
        let mut batch = RnsBatch::zeros(1, ch);
        batch.set(0, 0, 1);
        batch.set(0, 1, 2);
        let bytes = batch.as_u32_bytes();
        assert_eq!(bytes.len(), 4 * 4); // 4 channels * 4 bytes
        assert_eq!(bytes[0], 1);
        assert_eq!(bytes[4], 2);
    }
}
