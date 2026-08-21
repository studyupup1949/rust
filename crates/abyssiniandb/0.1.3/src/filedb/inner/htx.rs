use super::super::{FileBufSizeParam, FileDbParams, HashBucketsParam};
use super::piece::PieceMgr;
use super::semtype::*;
use super::vfile::VarFile;
use rabuf::{SmallRead, SmallWrite};
use std::cell::RefCell;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{Read, Result, Write};
use std::path::Path;
use std::rc::Rc;

type HeaderSignature = [u8; 8];

const CHUNK_SIZE: u32 = 32 * 4 * 1024;
const HTX_HEADER_SZ: u64 = 128;
const HTX_HEADER_SIGNATURE: HeaderSignature = [b'a', b'b', b'y', b's', b'd', b'b', b'H', 0u8];
const DEFAULT_HT_SIZE: u64 = 16 * 1024 * 1024;

#[derive(Debug)]
pub struct VarFileHtxCache {
    pub file: VarFile,
    buckets_size: u64,
    #[cfg(feature = "htx_print_hits")]
    hits: u64,
    #[cfg(feature = "htx_print_hits")]
    miss: u64,
}

impl VarFileHtxCache {
    fn new(file: VarFile) -> Self {
        Self {
            file,
            buckets_size: 0,
            #[cfg(feature = "htx_print_hits")]
            hits: 0,
            #[cfg(feature = "htx_print_hits")]
            miss: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HtxFile(pub Rc<RefCell<VarFileHtxCache>>);

const HTX_SIZE_FREE_OFFSET: [u64; 0] = [];
const HTX_SIZE_ARY: [u32; 0] = [];

/// returns the number of buckets needed to hold the given number of items,
/// taking the maximum load factor into account.
fn capacity_to_buckets_size(cap: u64) -> u64 {
    if cap == 0 {
        panic!("capacity should NOT be zero.");
    }
    //
    // for small hash buckets. minimum buckets size is 8.
    if cap < 8 {
        return 8;
    }
    //
    // otherwise require 1/9 buckets to be empty (88.8% load)
    let adjusted_cap = cap + cap / 8;
    //
    // Any overflows will have been caught by the checked_mul. Also, any
    // rounding errors from the division above will be cleaned up by
    // next_power_of_two (which can't overflow because of the previous division).
    adjusted_cap.next_power_of_two()
}

impl HtxFile {
    pub fn open_with_params<P: AsRef<Path>>(
        path: P,
        ks_name: &str,
        sig2: HeaderSignature,
        params: &FileDbParams,
    ) -> Result<Self> {
        let piece_mgr = PieceMgr::new(&HTX_SIZE_FREE_OFFSET, &HTX_SIZE_ARY);
        let mut pb = path.as_ref().to_path_buf();
        pb.push(format!("{ks_name}.htx"));
        let std_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(pb)?;
        let mut file = match params.htx_buf_size {
            FileBufSizeParam::Size(val) => {
                let idx_buf_chunk_size = CHUNK_SIZE;
                let idx_buf_num_chunks = val / idx_buf_chunk_size;
                VarFile::with_capacity(
                    piece_mgr,
                    "htx",
                    std_file,
                    idx_buf_chunk_size,
                    idx_buf_num_chunks.try_into().unwrap(),
                )?
            }
            FileBufSizeParam::PerMille(val) => {
                VarFile::with_per_mille(piece_mgr, "htx", std_file, CHUNK_SIZE, val)?
            }
            FileBufSizeParam::Auto => VarFile::new(piece_mgr, "htx", std_file)?,
        };
        let file_length: NodePieceOffset = file.seek_to_end()?;
        //
        let mut file_nc = VarFileHtxCache::new(file);
        //
        if file_length.is_zero() {
            //
            let buckets_size = match params.buckets_size {
                HashBucketsParam::BucketsSize(x) => x.next_power_of_two(),
                HashBucketsParam::Capacity(x) => capacity_to_buckets_size(x),
                HashBucketsParam::Default => DEFAULT_HT_SIZE,
            };
            //
            write_htxf_init_header(&mut file_nc.file, sig2, buckets_size)?;
            #[cfg(feature = "htx_bitmap")]
            let off = NodePieceOffset::new(HTX_HEADER_SZ + buckets_size * 8 + buckets_size / 8);
            #[cfg(not(feature = "htx_bitmap"))]
            let off = NodePieceOffset::new(HTX_HEADER_SZ + buckets_size * 8);
            //
            file_nc.file.set_file_length(off)?;
            let off = NodePieceOffset::new(off.as_value() - 8);
            file_nc.file.seek_from_start(off)?;
            file_nc.file.write_u64_le(0)?;
            //
            file_nc.buckets_size = buckets_size;
        } else {
            check_htxf_header(&mut file_nc.file, sig2)?;
            file_nc.buckets_size = file_nc.file.read_hash_buckets_size()?;
        }
        Ok(Self(Rc::new(RefCell::new(file_nc))))
    }
    #[inline]
    pub fn read_fill_buffer(&self) -> Result<()> {
        let mut locked = RefCell::borrow_mut(&self.0);
        locked.file.read_fill_buffer()
    }
    #[inline]
    pub fn flush(&self) -> Result<()> {
        let mut locked = RefCell::borrow_mut(&self.0);
        locked.file.flush()
    }
    #[inline]
    pub fn sync_all(&self) -> Result<()> {
        let mut locked = RefCell::borrow_mut(&self.0);
        locked.file.sync_all()
    }
    #[inline]
    pub fn sync_data(&self) -> Result<()> {
        let mut locked = RefCell::borrow_mut(&self.0);
        locked.file.sync_data()
    }
    #[cfg(feature = "buf_stats")]
    #[inline]
    pub fn buf_stats(&self) -> Vec<(String, i64)> {
        let locked = RefCell::borrow(&self.0);
        locked.file.buf_stats()
    }
    //
    #[inline]
    pub fn read_hash_buckets_size(&self) -> Result<u64> {
        let mut locked = RefCell::borrow_mut(&self.0);
        locked.file.read_hash_buckets_size()
    }
    #[inline]
    pub fn read_key_piece_offset(&self, hash: HashValue) -> Result<KeyPieceOffset> {
        let mut locked = RefCell::borrow_mut(&self.0);
        let buckets_size = locked.buckets_size;
        let idx = hash.as_value() % buckets_size;
        locked.file.read_key_piece_offset(idx)
    }
    #[inline]
    pub fn write_key_piece_offset(&self, hash: HashValue, offset: KeyPieceOffset) -> Result<()> {
        let mut locked = RefCell::borrow_mut(&self.0);
        let buckets_size = locked.buckets_size;
        let idx = hash.as_value() % buckets_size;
        locked
            .file
            .write_key_piece_offset(buckets_size, idx, offset)
    }
    #[cfg(feature = "htx_print_hits")]
    #[inline]
    pub fn set_hits(&mut self) {
        let mut locked = RefCell::borrow_mut(&self.0);
        locked.hits += 1;
    }
    #[cfg(feature = "htx_print_hits")]
    #[inline]
    pub fn set_miss(&mut self) {
        let mut locked = RefCell::borrow_mut(&self.0);
        locked.miss += 1;
    }
    #[inline]
    pub fn read_item_count(&self) -> Result<u64> {
        let mut locked = RefCell::borrow_mut(&self.0);
        locked.file.read_item_count()
    }
    #[inline]
    pub fn write_item_count_up(&mut self) -> Result<()> {
        let mut locked = RefCell::borrow_mut(&self.0);
        let val = locked.file.read_item_count()?;
        locked.file.write_item_count(val + 1)
    }
    #[inline]
    pub fn write_item_count_down(&mut self) -> Result<()> {
        let mut locked = RefCell::borrow_mut(&self.0);
        let val = locked.file.read_item_count()?;
        if val > 0 {
            locked.file.write_item_count(val - 1)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "htx_print_hits")]
impl Drop for HtxFile {
    fn drop(&mut self) {
        let (hits, miss) = {
            let locked = RefCell::borrow_mut(&self.0);
            (locked.hits, locked.miss)
        };
        let total = hits + miss;
        let ratio = hits as f64 / total as f64;
        eprintln!("htx hits: {}/{} [{:.2}%]", hits, total, 100.0 * ratio);
    }
}

// for debug
impl HtxFile {
    pub fn _ht_size_and_count(&self) -> Result<(u64, u64)> {
        let mut locked = RefCell::borrow_mut(&self.0);
        let ht_size = locked.file.read_hash_buckets_size()?;
        let count = locked.file.read_item_count()?;
        Ok((ht_size, count))
    }
    pub fn htx_filling_rate_per_mill(&self) -> Result<(u64, u32)> {
        let mut locked = RefCell::borrow_mut(&self.0);
        let buckets_size = locked.buckets_size;
        let mut count = 0;
        for idx in 0..buckets_size {
            let offset = locked.file.read_key_piece_offset(idx)?;
            if !offset.is_zero() {
                count += 1;
            }
        }
        Ok((count, (count * 1000 / buckets_size) as u32))
    }
}

/**
write initiale header to file.

## header map

The htx header size is 128 bytes.

```text
+--------+-------+-------------+---------------------------+
| offset | bytes | name        | comment                   |
+--------+-------+-------------+---------------------------+
| 0      | 8     | signature1  | b"abysdbH\0"              |
| 8      | 8     | signature2  | 8 bytes type signature    |
| 16     | 8     | ht size     | hash table size           |
| 24     | 8     | count       | count of items            |
| 32     | 96    | reserve1    |                           |
+--------+-------+-------------+---------------------------+
```

- signature1: always fixed 8 bytes
- signature2: 8 bytes type signature

*/
const HTX_HT_SIZE_OFFSET: u64 = 16;
const HTX_ITEM_COUNT_OFFSET: u64 = 24;

fn write_htxf_init_header(
    file: &mut VarFile,
    signature2: HeaderSignature,
    buckets_size: u64,
) -> Result<()> {
    file.seek_from_start(NodePieceOffset::new(0))?;
    // signature1
    file.write_all(&HTX_HEADER_SIGNATURE)?;
    // signature2
    file.write_all(&signature2)?;
    // buckets size
    file.write_u64_le(buckets_size)?;
    // count .. rserve1
    file.write_all(&[0u8; 104])?;
    //
    Ok(())
}

fn check_htxf_header(file: &mut VarFile, signature2: HeaderSignature) -> Result<()> {
    file.seek_from_start(NodePieceOffset::new(0))?;
    // signature1
    let mut sig1 = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
    file.read_exact(&mut sig1)?;
    assert!(sig1 == HTX_HEADER_SIGNATURE, "invalid header signature1");
    // signature2
    let mut sig2 = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
    file.read_exact(&mut sig2)?;
    assert!(
        sig2 == signature2,
        "invalid header signature2, type signature: {sig2:?}"
    );
    // top node offset
    let _top_node_offset = file.read_u64_le()?;
    assert!(_top_node_offset != 0, "invalid root offset");
    //
    Ok(())
}

impl VarFile {
    fn read_hash_buckets_size(&mut self) -> Result<u64> {
        self.seek_from_start(NodePieceOffset::new(HTX_HT_SIZE_OFFSET))?;
        self.read_u64_le()
    }
    fn _read_hash_buckets_size(&mut self, val: u64) -> Result<()> {
        self.seek_from_start(NodePieceOffset::new(HTX_HT_SIZE_OFFSET))?;
        self.write_u64_le(val)
    }
    fn read_item_count(&mut self) -> Result<u64> {
        self.seek_from_start(NodePieceOffset::new(HTX_ITEM_COUNT_OFFSET))?;
        self.read_u64_le()
    }
    fn write_item_count(&mut self, val: u64) -> Result<()> {
        self.seek_from_start(NodePieceOffset::new(HTX_ITEM_COUNT_OFFSET))?;
        self.write_u64_le(val)
    }
    pub fn read_key_piece_offset(&mut self, idx: u64) -> Result<KeyPieceOffset> {
        self.seek_from_start(NodePieceOffset::new(HTX_HEADER_SZ + 8 * idx))?;
        self.read_u64_le().map(KeyPieceOffset::new)
    }
    fn write_key_piece_offset(
        &mut self,
        bucket_size: u64,
        idx: u64,
        offset: KeyPieceOffset,
    ) -> Result<()> {
        /*<CHECK>
        let count = self.read_item_count()?;
        if offset.is_zero() {
            if count > 0 {
                self.write_item_count(count - 1)?;
            }
        } else {
            self.write_item_count(count + 1)?;
        }
        */
        // write flag into bitmap
        #[cfg(feature = "htx_bitmap")]
        {
            let bitmap_idx = idx / 8;
            let bitmap_bit_idx = idx % 8;
            //
            let bimap_start = HTX_HEADER_SZ + bucket_size * 8;
            self.seek_from_start(NodePieceOffset::new(bimap_start + bitmap_idx))?;
            let mut byte = self.read_u8()?;
            if offset.is_zero() {
                byte &= !(1 << bitmap_bit_idx);
            } else {
                byte |= 1 << bitmap_bit_idx;
            }
            //
            self.seek_from_start(NodePieceOffset::new(bimap_start + bitmap_idx))?;
            self.write_u8(byte)?;
        }
        // store into bucket
        self.seek_from_start(NodePieceOffset::new(HTX_HEADER_SZ + 8 * idx))?;
        self.write_u64_le(offset.into())?;
        //
        Ok(())
    }
    pub fn next_key_piece_offset(
        &mut self,
        buckets_size: u64,
        idx: u64,
    ) -> Result<(u64, KeyPieceOffset)> {
        // write flag into bitmap
        #[cfg(feature = "htx_bitmap")]
        let idx = {
            let bitmap_idx = idx / 8;
            let bitmap_bit_idx = idx % 8;
            //
            if bitmap_bit_idx == 0 {
                let bimap_start = HTX_HEADER_SZ + buckets_size * 8;
                self.seek_from_start(NodePieceOffset::new(bimap_start + bitmap_idx))?;
                let mut idx = idx;
                //
                let mut byte_8 = 0;
                while byte_8 == 0 && idx < buckets_size - 8 {
                    byte_8 = self.read_u64_le()?;
                    idx += 8 * 8;
                }
                if idx >= 8 * 8 {
                    self.seek_back_size(NodePieceSize::new(std::mem::size_of_val(&byte_8) as u32))?;
                    idx -= 8 * 8;
                }
                //
                let mut byte = 0;
                while byte == 0 && idx < buckets_size {
                    byte = self.read_u8()?;
                    idx += 8;
                }
                idx - 8
            } else {
                idx
            }
        };
        //
        self.seek_from_start(NodePieceOffset::new(HTX_HEADER_SZ + 8 * idx))?;
        let mut idx = idx;
        let mut off = 0;
        while off == 0 && idx < buckets_size {
            off = self.read_u64_le()?;
            idx += 1;
        }
        Ok((idx, KeyPieceOffset::new(off)))
    }
}

/*
```text
hash buckes table:
+--------+-------+-------------+-----------------------------------+
| offset | bytes | name        | comment                           |
+--------+-------+-------------+-----------------------------------+
| 0      | 8     | key offset  | offset of key piece               |
| 8      | 8     | key offset  | offset of key piece               |
| --     | --    | --          | --                                |
| --     | 8     | key offset  | offset of key piece               |
+--------+-------+-------------+-----------------------------------+

hash buckes bitmap:
+--------+-------+-------------+-----------------------------------+
| offset | bytes | name        | comment                           |
+--------+-------+-------------+-----------------------------------+
| 0      | 8     | bitmap1     | bitmap of buckets index           |
| --     | --    | --          | --                                |
| --     | 8     | bitmapN     | bitmap of buckets index           |
+--------+-------+-------------+-----------------------------------+
```
*/
