use super::super::super::{DbMapKeyType, DbXxxBase, DbXxxObjectSafe};
use super::super::{
    CheckFileDbMap, CountOfPerSize, FileDbParams, KeysCountStats, LengthStats, RecordSizeStats,
};
use super::_cold;
use super::key::KeyPieceOffsetIter;
use super::semtype::*;
use super::val::ValuePieceOffsetIter;
use super::{key, val};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::io::Result;
use std::path::Path;
use std::rc::Rc;

//#[cfg(all(
//    feature = "idx_find_uu",
//    any(feature = "vf_node_u32", feature = "vf_node_u64")
//))]
//use rabuf::SmallRead;

use super::htx;

#[derive(Debug)]
pub struct FileDbXxxInner<KT: DbMapKeyType> {
    dirty: bool,
    //
    key_file: key::KeyFile<KT>,
    val_file: val::ValueFile,
    htx_file: htx::HtxFile,
    //
    _phantom: std::marker::PhantomData<KT>,
}

impl<KT: DbMapKeyType> FileDbXxxInner<KT> {
    pub(crate) fn open_with_params<P: AsRef<Path>>(
        path: P,
        ks_name: &str,
        params: FileDbParams,
    ) -> Result<FileDbXxxInner<KT>> {
        let key_file = key::KeyFile::open_with_params(&path, ks_name, KT::signature(), &params)?;
        let val_file = val::ValueFile::open_with_params(&path, ks_name, KT::signature(), &params)?;
        let htx_file = htx::HtxFile::open_with_params(&path, ks_name, KT::signature(), &params)?;
        //
        Ok(Self {
            key_file,
            val_file,
            htx_file,
            dirty: false,
            _phantom: std::marker::PhantomData,
        })
    }
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    /*
    fn key_piece_offset_iter(&self) -> DbXxxKeyPieceOffsetIter<KT> {
        DbXxxKeyPieceOffsetIter::new(self).unwrap()
    }
    */
    fn key_piece_offset_iter(&self) -> KeyPieceOffsetIter {
        self.key_file.piece_offset_iter()
    }
    fn value_piece_offset_iter(&self) -> ValuePieceOffsetIter {
        self.val_file.piece_offset_iter()
    }
}

// for utils
impl<KT: DbMapKeyType> FileDbXxxInner<KT> {
    #[inline]
    pub(crate) fn load_key_data(&self, piece_offset: KeyPieceOffset) -> Result<KT> {
        debug_assert!(!piece_offset.is_zero());
        self.key_file.read_piece_only_key(piece_offset)
    }
    #[inline]
    fn load_key_piece_size(&self, piece_offset: KeyPieceOffset) -> Result<KeyPieceSize> {
        self.key_file.read_piece_only_size(piece_offset)
    }
    #[inline]
    fn load_key_length(&self, piece_offset: KeyPieceOffset) -> Result<KeyLength> {
        self.key_file.read_piece_only_key_length(piece_offset)
    }
    #[inline]
    fn _load_key_value_offset(&self, piece_offset: KeyPieceOffset) -> Result<ValuePieceOffset> {
        self.key_file.read_piece_only_value_offset(piece_offset)
    }
    //
    #[inline]
    fn load_value(&self, piece_offset: KeyPieceOffset) -> Result<Vec<u8>> {
        debug_assert!(!piece_offset.is_zero());
        let value_offset = self.key_file.read_piece_only_value_offset(piece_offset)?;
        self.val_file.read_piece_only_value(value_offset)
    }
    #[inline]
    fn load_value_piece_size(&self, piece_offset: ValuePieceOffset) -> Result<ValuePieceSize> {
        self.val_file.read_piece_only_size(piece_offset)
    }
    /*
    fn load_value_piece_size(&self, piece_offset: KeyPieceOffset) -> Result<ValuePieceSize> {
        let value_offset = self.key_file.read_piece_only_value_offset(piece_offset)?;
        self.val_file.read_piece_only_size(value_offset)
    }
    */
    #[inline]
    fn load_value_length(&self, piece_offset: ValuePieceOffset) -> Result<ValueLength> {
        self.val_file.read_piece_only_value_length(piece_offset)
    }
    /*
    fn load_value_length(&self, piece_offset: KeyPieceOffset) -> Result<ValueLength> {
        let value_offset = self.key_file.read_piece_only_value_offset(piece_offset)?;
        self.val_file.read_piece_only_value_length(value_offset)
    }
    */
}

// insert: NEW
impl<KT: DbMapKeyType> FileDbXxxInner<KT> {
    #[inline]
    fn store_value_on_insert(
        &mut self,
        piece_offset: KeyPieceOffset,
        value: &[u8],
    ) -> Result<KeyPieceOffset> {
        let mut key_piece = self.key_file.read_piece(piece_offset)?;
        let mut val_piece = self.val_file.read_piece(key_piece.value_offset)?;
        val_piece.value = value.to_vec();
        let new_value_piece = self.val_file.write_piece(val_piece)?;
        let new_key_piece = if key_piece.value_offset == new_value_piece.offset {
            key_piece
        } else {
            _cold();
            key_piece.value_offset = new_value_piece.offset;
            self.key_file.write_piece(key_piece)?
        };
        Ok(new_key_piece.offset)
    }
}

// delete: NEW
impl<KT: DbMapKeyType> FileDbXxxInner<KT> {}

// find: NEW
impl<KT: DbMapKeyType> FileDbXxxInner<KT> {
    fn find_in_hash_buckets_kt(
        &mut self,
        hash: HashValue,
        key_kt: &KT,
    ) -> Result<Option<(KeyPieceOffset, KeyPieceOffset)>> {
        let mut prev_key_offset = KeyPieceOffset::new(0);
        let mut key_offset = self.htx_file.read_key_piece_offset(hash)?;
        if !key_offset.is_zero() {
            let mut locked_key = self.key_file.0.borrow_mut();
            //
            while !key_offset.is_zero() {
                let flg = {
                    let key_string = locked_key.read_piece_only_key_maybeslice(key_offset)?;
                    match key_kt.cmp_u8(&key_string) {
                        Ordering::Equal => true,
                        Ordering::Greater => false,
                        Ordering::Less => false,
                    }
                };
                if flg {
                    #[cfg(feature = "htx_print_hits")]
                    self.htx_file.set_hits();
                    return Ok(Some((key_offset, prev_key_offset)));
                } else {
                    _cold();
                    #[cfg(feature = "htx_print_hits")]
                    self.htx_file.set_miss();
                    //
                    prev_key_offset = key_offset;
                    key_offset = locked_key.read_piece_only_bucket_next_offset(key_offset)?;
                }
            }
        }
        Ok(None)
    }
}

// impl trait: DbXxxBase
impl<KT: DbMapKeyType> DbXxxBase for FileDbXxxInner<KT> {
    #[inline]
    fn len(&self) -> Result<u64> {
        self.htx_file.read_item_count()
    }
    #[inline]
    fn read_fill_buffer(&mut self) -> Result<()> {
        self.val_file.read_fill_buffer()?;
        self.key_file.read_fill_buffer()?;
        self.htx_file.read_fill_buffer()?;
        Ok(())
    }
    #[inline]
    fn flush(&mut self) -> Result<()> {
        if self.is_dirty() {
            // save all data
            self.val_file.flush()?;
            self.key_file.flush()?;
            self.htx_file.flush()?;
            self.dirty = false;
        }
        Ok(())
    }
    #[inline]
    fn sync_all(&mut self) -> Result<()> {
        if self.is_dirty() {
            // save all data and meta
            self.val_file.sync_all()?;
            self.key_file.sync_all()?;
            self.htx_file.sync_all()?;
            self.dirty = false;
        }
        Ok(())
    }
    #[inline]
    fn sync_data(&mut self) -> Result<()> {
        if self.is_dirty() {
            // save all data
            self.val_file.sync_data()?;
            self.key_file.sync_data()?;
            self.htx_file.sync_data()?;
            self.dirty = false;
        }
        Ok(())
    }
}

// impl trait: DbXxxObjectSafe<KT>
impl<KT: DbMapKeyType> DbXxxObjectSafe<KT> for FileDbXxxInner<KT> {
    #[inline]
    fn get_kt(&mut self, key_kt: &KT) -> Result<Option<Vec<u8>>> {
        let hash = HashValue::new(key_kt.hash_value());
        let opt = self.find_in_hash_buckets_kt(hash, key_kt)?;
        if let Some((key_offset, _prev_key_offset)) = opt {
            self.load_value(key_offset).map(Some)
        } else {
            _cold();
            Ok(None)
        }
    }
    #[inline]
    fn put_kt(&mut self, key_kt: &KT, value: &[u8]) -> Result<()> {
        let hash = HashValue::new(key_kt.hash_value());
        let opt = self.find_in_hash_buckets_kt(hash, key_kt)?;
        if let Some((key_offset, _prev_key_offset)) = opt {
            let new_key_offset = self.store_value_on_insert(key_offset, value)?;
            if key_offset != new_key_offset {
                unimplemented!("key_offset != new_key_offset : in put_kt");
            }
        } else {
            _cold();
            // adding
            let bucket_next_offset = self.htx_file.read_key_piece_offset(hash)?;
            let new_val_piece = self.val_file.add_value_piece(value)?;
            let new_key_piece =
                self.key_file
                    .add_key_piece(key_kt, new_val_piece.offset, bucket_next_offset)?;
            self.htx_file
                .write_key_piece_offset(hash, new_key_piece.offset)?;
            self.htx_file.write_item_count_up()?;
        }
        Ok(())
    }
    #[inline]
    fn del_kt(&mut self, key_kt: &KT) -> Result<Option<Vec<u8>>> {
        let hash = HashValue::new(key_kt.hash_value());
        let opt = self.find_in_hash_buckets_kt(hash, key_kt)?;
        if let Some((key_offset, _prev_key_offset)) = opt {
            let key_piece = self.key_file.read_piece(key_offset)?;
            let value = self
                .val_file
                .read_piece_only_value(key_piece.value_offset)?;
            //
            if _prev_key_offset.is_zero() {
                self.htx_file
                    .write_key_piece_offset(hash, key_piece.bucket_next_offset)?;
            } else {
                _cold();
                // changing link of bucket chain.
                let mut prev_key_piece = self.key_file.read_piece(_prev_key_offset)?;
                prev_key_piece.bucket_next_offset = key_piece.bucket_next_offset;
                let new_prev_key = self.key_file.write_piece(prev_key_piece)?;
                if _prev_key_offset != new_prev_key.offset {
                    _cold();
                    panic!("_prev_key_offset != new_prev_key_offset : in del_kt");
                }
            }
            //
            self.val_file.delete_piece(key_piece.value_offset)?;
            self.key_file.delete_piece(key_offset)?;
            self.htx_file.write_item_count_down()?;
            //
            Ok(Some(value))
        } else {
            _cold();
            Ok(None)
        }
    }
    #[inline]
    fn includes_key_kt(&mut self, key_kt: &KT) -> Result<bool> {
        let hash = HashValue::new(key_kt.hash_value());
        let opt = self.find_in_hash_buckets_kt(hash, key_kt)?;
        if let Some((_key_offset, _prev_key_offset)) = opt {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// for Iterator
//
#[derive(Debug)]
struct DbXxxKeyPieceOffsetIter<'a, KT: DbMapKeyType> {
    db_map_inner: &'a FileDbXxxInner<KT>,
    remaining_item_count: u64,
    buckets_size: u64,
    buckets_idx: u64,
    key_offset: KeyPieceOffset,
}

impl<'a, KT: DbMapKeyType> DbXxxKeyPieceOffsetIter<'a, KT> {
    fn _new(db_map_inner: &'a FileDbXxxInner<KT>) -> Result<Self> {
        let (buckets_size, remaining_item_count) = {
            //let db_map_inner = RefCell::borrow(&db_map);
            (
                db_map_inner.htx_file.read_hash_buckets_size()?,
                db_map_inner.htx_file.read_item_count()?,
            )
        };
        Ok(Self {
            db_map_inner,
            remaining_item_count,
            buckets_size,
            buckets_idx: 0,
            key_offset: KeyPieceOffset::new(0),
        })
    }
    fn next_piece_offset(&mut self) -> Option<KeyPieceOffset> {
        //let db_map_inner = RefCell::borrow(&self.db_map);
        let db_map_inner = self.db_map_inner;
        let mut key_inner = RefCell::borrow_mut(&db_map_inner.key_file.0);
        let mut htx_inner = RefCell::borrow_mut(&db_map_inner.htx_file.0);
        if !self.key_offset.is_zero() {
            self.key_offset = key_inner
                .read_piece_only_bucket_next_offset(self.key_offset)
                .unwrap();
        } else {
            _cold();
        }
        if self.key_offset.is_zero() {
            let mut key_offset = self.key_offset;
            let mut buckets_idx = self.buckets_idx;
            let buckets_size = self.buckets_size;
            while key_offset.is_zero() && buckets_idx < buckets_size {
                let (next_idx, offset) = htx_inner
                    .file
                    .next_key_piece_offset(buckets_size, buckets_idx)
                    .unwrap();
                key_offset = offset;
                buckets_idx = next_idx;
            }
            self.key_offset = key_offset;
            self.buckets_idx = buckets_idx;
        }
        //
        if self.key_offset.is_zero() || self.remaining_item_count == 0 {
            _cold();
            None
        } else {
            if self.remaining_item_count > 0 {
                self.remaining_item_count -= 1;
            }
            Some(self.key_offset)
        }
    }
}

// impl trait: Iterator
impl<'a, KT: DbMapKeyType> Iterator for DbXxxKeyPieceOffsetIter<'a, KT> {
    type Item = KeyPieceOffset;
    #[inline]
    fn next(&mut self) -> Option<KeyPieceOffset> {
        self.next_piece_offset()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.remaining_item_count as usize,
            Some(self.remaining_item_count as usize),
        )
    }
}

impl<'a, KT: DbMapKeyType> ExactSizeIterator for DbXxxKeyPieceOffsetIter<'a, KT> {}

//
#[derive(Debug)]
pub struct DbXxxIterMut<KT: DbMapKeyType> {
    db_map: Rc<RefCell<FileDbXxxInner<KT>>>,
    remaining_item_count: u64,
    buckets_size: u64,
    buckets_idx: u64,
    key_offset: KeyPieceOffset,
}

impl<KT: DbMapKeyType> DbXxxIterMut<KT> {
    pub fn new(db_map: Rc<RefCell<FileDbXxxInner<KT>>>) -> Result<Self> {
        let (buckets_size, remaining_item_count) = {
            let db_map_inner = RefCell::borrow(&db_map);
            (
                db_map_inner.htx_file.read_hash_buckets_size()?,
                db_map_inner.htx_file.read_item_count()?,
            )
        };
        Ok(Self {
            db_map,
            remaining_item_count,
            buckets_size,
            buckets_idx: 0,
            key_offset: KeyPieceOffset::new(0),
        })
    }
    fn next_piece_offset(&mut self) -> Option<KeyPieceOffset> {
        let db_map_inner = RefCell::borrow(&self.db_map);
        let mut key_inner = RefCell::borrow_mut(&db_map_inner.key_file.0);
        let mut htx_inner = RefCell::borrow_mut(&db_map_inner.htx_file.0);
        if !self.key_offset.is_zero() {
            self.key_offset = key_inner
                .read_piece_only_bucket_next_offset(self.key_offset)
                .unwrap();
        } else {
            _cold();
        }
        if self.key_offset.is_zero() {
            let mut key_offset = self.key_offset;
            let mut buckets_idx = self.buckets_idx;
            let buckets_size = self.buckets_size;
            while key_offset.is_zero() && buckets_idx < buckets_size {
                let (next_idx, offset) = htx_inner
                    .file
                    .next_key_piece_offset(buckets_size, buckets_idx)
                    .unwrap();
                key_offset = offset;
                buckets_idx = next_idx;
            }
            self.key_offset = key_offset;
            self.buckets_idx = buckets_idx;
        }
        //
        if self.key_offset.is_zero() || self.remaining_item_count == 0 {
            _cold();
            None
        } else {
            if self.remaining_item_count > 0 {
                self.remaining_item_count -= 1;
            }
            Some(self.key_offset)
        }
    }
}

// impl trait: Iterator
impl<KT: DbMapKeyType> Iterator for DbXxxIterMut<KT> {
    type Item = (KT, Vec<u8>);
    #[inline]
    fn next(&mut self) -> Option<(KT, Vec<u8>)> {
        if let Some(key_offset) = self.next_piece_offset() {
            let db_map_inner = RefCell::borrow_mut(&self.db_map);
            let key = db_map_inner.load_key_data(key_offset).unwrap();
            let value_vec = db_map_inner.load_value(key_offset).unwrap();
            Some((key, value_vec))
        } else {
            _cold();
            None
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.remaining_item_count as usize,
            Some(self.remaining_item_count as usize),
        )
    }
}

impl<KT: DbMapKeyType> ExactSizeIterator for DbXxxIterMut<KT> {}

//
#[derive(Debug)]
pub struct DbXxxIter<KT: DbMapKeyType> {
    iter: DbXxxIterMut<KT>,
}

impl<KT: DbMapKeyType> DbXxxIter<KT> {
    #[inline]
    pub fn new(db_map: Rc<RefCell<FileDbXxxInner<KT>>>) -> Result<Self> {
        Ok(Self {
            iter: DbXxxIterMut::new(db_map)?,
        })
    }
}

// impl trait: Iterator
impl<KT: DbMapKeyType> Iterator for DbXxxIter<KT> {
    type Item = (KT, Vec<u8>);
    #[inline]
    fn next(&mut self) -> Option<(KT, Vec<u8>)> {
        self.iter.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<KT: DbMapKeyType> ExactSizeIterator for DbXxxIter<KT> {}

//
#[derive(Debug)]
pub struct DbXxxIntoIter<KT: DbMapKeyType> {
    iter: DbXxxIterMut<KT>,
}

impl<KT: DbMapKeyType> DbXxxIntoIter<KT> {
    #[inline]
    pub fn new(db_map: Rc<RefCell<FileDbXxxInner<KT>>>) -> Result<Self> {
        Ok(Self {
            iter: DbXxxIterMut::new(db_map)?,
        })
    }
}

// impl trait: Iterator
impl<KT: DbMapKeyType> Iterator for DbXxxIntoIter<KT> {
    type Item = (KT, Vec<u8>);
    #[inline]
    fn next(&mut self) -> Option<(KT, Vec<u8>)> {
        self.iter.next()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<KT: DbMapKeyType> ExactSizeIterator for DbXxxIntoIter<KT> {}

/// An iterator over the keys of a DbMap.
#[derive(Debug)]
pub struct DbXxxKeys<KT: DbMapKeyType> {
    iter: DbXxxIterMut<KT>,
}

impl<KT: DbMapKeyType> DbXxxKeys<KT> {
    #[inline]
    pub fn new(db_map: Rc<RefCell<FileDbXxxInner<KT>>>) -> Result<Self> {
        Ok(Self {
            iter: DbXxxIterMut::new(db_map)?,
        })
    }
}

// impl trait: Iterator
impl<KT: DbMapKeyType> Iterator for DbXxxKeys<KT> {
    type Item = KT;
    #[inline]
    fn next(&mut self) -> Option<KT> {
        self.iter.next().map(|(k, _v)| k)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<KT: DbMapKeyType> ExactSizeIterator for DbXxxKeys<KT> {}

/// An iterator over the values of a DbMap.
#[derive(Debug)]
pub struct DbXxxValues<KT: DbMapKeyType> {
    iter: DbXxxIterMut<KT>,
}

impl<KT: DbMapKeyType> DbXxxValues<KT> {
    #[inline]
    pub fn new(db_map: Rc<RefCell<FileDbXxxInner<KT>>>) -> Result<Self> {
        Ok(Self {
            iter: DbXxxIterMut::new(db_map)?,
        })
    }
}

// impl trait: Iterator
impl<KT: DbMapKeyType> Iterator for DbXxxValues<KT> {
    type Item = Vec<u8>;
    #[inline]
    fn next(&mut self) -> Option<Vec<u8>> {
        self.iter.next().map(|(_k, v)| v)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<KT: DbMapKeyType> ExactSizeIterator for DbXxxValues<KT> {}

// for debug
impl<KT: DbMapKeyType + std::fmt::Display> CheckFileDbMap for FileDbXxxInner<KT> {
    #[cfg(feature = "htx")]
    fn ht_size_and_count(&self) -> Result<(u64, u64)> {
        self.htx_file.ht_size_and_count()
    }
    /// count of the free key piece
    fn count_of_free_key_piece(&self) -> Result<CountOfPerSize> {
        self.key_file.count_of_free_key_piece()
    }
    /// count of the free value piece
    fn count_of_free_value_piece(&self) -> Result<CountOfPerSize> {
        self.val_file.count_of_free_value_piece()
    }
    /// buffer statistics
    #[cfg(feature = "buf_stats")]
    fn buf_stats(&self) -> Vec<(String, i64)> {
        let mut vec = self.dat_file.buf_stats();
        let mut vec2 = self.idx_file.buf_stats();
        vec.append(&mut vec2);
        vec
    }
    /// piece size statistics
    fn key_piece_size_stats(&self) -> Result<RecordSizeStats<Key>> {
        let mut piece_vec = RecordSizeStats::default();
        //
        for key_piece_offset in self.key_piece_offset_iter() {
            let size = self.load_key_piece_size(key_piece_offset)?;
            let length = self.load_key_length(key_piece_offset)?;
            if !length.is_zero() {
                piece_vec.touch_size(size);
            }
        }
        Ok(piece_vec)
    }
    fn value_piece_size_stats(&self) -> Result<RecordSizeStats<Value>> {
        let mut piece_vec = RecordSizeStats::default();
        //
        for value_piece_offset in self.value_piece_offset_iter() {
            let size = self.load_value_piece_size(value_piece_offset)?;
            let length = self.load_value_length(value_piece_offset)?;
            if !length.is_zero() {
                piece_vec.touch_size(size);
            }
        }
        Ok(piece_vec)
    }
    /// key length statistics
    fn key_length_stats(&self) -> Result<LengthStats<Key>> {
        let mut length_vec = LengthStats::default();
        //
        for key_piece_offset in self.key_piece_offset_iter() {
            //let size = self.load_key_piece_size(key_piece_offset)?;
            let length = self.load_key_length(key_piece_offset)?;
            if !length.is_zero() {
                length_vec.touch_length(length);
            }
        }
        Ok(length_vec)
    }
    /// value length statistics
    fn value_length_stats(&self) -> Result<LengthStats<Value>> {
        let mut length_vec = LengthStats::default();
        //
        for value_piece_offset in self.value_piece_offset_iter() {
            //let size = self.load_value_piece_size(value_piece_offset)?;
            let length = self.load_value_length(value_piece_offset)?;
            if !length.is_zero() {
                length_vec.touch_length(length);
            }
        }
        Ok(length_vec)
    }
    /// keys count statistics
    fn keys_count_stats(&self) -> Result<KeysCountStats> {
        //self.idx_file.keys_count_stats()
        Ok(KeysCountStats::new(Vec::new()))
    }
    //#[cfg(feature = "htx")]
    fn htx_filling_rate_per_mill(&self) -> Result<(u64, u32)> {
        self.htx_file.htx_filling_rate_per_mill()
    }
    /*
    /// convert the index node tree to graph string for debug.
    fn graph_string(&self) -> Result<String> {
        self.idx_file.graph_string()
    }
    /// convert the index node tree to graph string for debug.
    fn graph_string_with_key_string(&self) -> Result<String> {
        self.idx_file.graph_string_with_key_string(self)
    }
    /// check the index node tree is balanced
    fn is_balanced(&self) -> Result<bool> {
        let top_node = self.idx_file.read_top_node()?;
        self.idx_file.is_balanced(&top_node)
    }
    /// check the index node tree is multi search tree
    fn is_mst_valid(&self) -> Result<bool> {
        let top_node = self.idx_file.read_top_node()?;
        self.idx_file.is_mst_valid(&top_node, self)
    }
    /// check the index node except the root and leaves of the tree has branches of hm or more.
    fn is_dense(&self) -> Result<bool> {
        let top_node = self.idx_file.read_top_node()?;
        self.idx_file.is_dense(&top_node)
    }
    /// get the depth of the index node
    fn depth_of_node_tree(&self) -> Result<u64> {
        let top_node = self.idx_file.read_top_node()?;
        self.idx_file.depth_of_node_tree(&top_node)
    }
    /// count of the free node
    fn count_of_free_node(&self) -> Result<CountOfPerSize> {
        self.idx_file.count_of_free_node()
    }
    /// count of the used piece and the used node
    fn count_of_used_node(&self) -> Result<(CountOfPerSize, CountOfPerSize, CountOfPerSize)> {
        self.idx_file.count_of_used_node(|off| {
            let ks = self.load_key_piece_size(off);
            if let Err(err) = ks {
                Err(err)
            } else {
                let vs = self.load_value_piece_size(off);
                if let Err(err) = vs {
                    Err(err)
                } else {
                    Ok((ks.unwrap(), vs.unwrap()))
                }
            }
        })
    }
    */
}
