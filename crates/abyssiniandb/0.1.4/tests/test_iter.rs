mod test_iter {
    use abyssiniandb::{DbBytes, DbMap, DbMapKeyType, DbString, DbU64};
    use std::collections::BTreeMap;
    //
    fn iter_test_map_empty_iter<T: DbMap<K>, K: DbMapKeyType>(db_map: &mut T) {
        assert!(db_map.is_empty().unwrap());
        assert_eq!(db_map.len().unwrap(), 0);
        //
        //assert_eq!(db_map.drain().next(), None);
        //assert_eq!(db_map.keys().next(), None);
        //assert_eq!(db_map.values().next(), None);
        //assert_eq!(db_map.values_mut().next(), None);
        #[cfg(not(miri))] // buggy on miri
        {
            assert_eq!(db_map.iter().next(), None);
        }
        //assert_eq!(db_map.iter_mut().next(), None);
        //assert_eq!(db_map.into_iter().next(), None);
    }
    //
    fn basic_test_map_string<T: DbMap<DbString>>(db_map: &mut T) {
        // insert
        db_map.put_string("key01", "value1").unwrap();
        db_map.put_string("key02", "value2").unwrap();
        db_map.put_string("key03", "value3").unwrap();
        db_map.put_string("key04", "value4").unwrap();
        db_map.put_string("key05", "value5").unwrap();
        assert_eq!(db_map.len().unwrap(), 5);
        // iterator
        let btmap: BTreeMap<DbString, Vec<u8>> = db_map.iter_mut().collect();
        let mut iter = btmap.into_iter();
        //let mut iter = db_map.iter_mut();
        assert_eq!(iter.next(), Some(("key01".into(), "value1".into())));
        assert_eq!(iter.next(), Some(("key02".into(), "value2".into())));
        assert_eq!(iter.next(), Some(("key03".into(), "value3".into())));
        assert_eq!(iter.next(), Some(("key04".into(), "value4".into())));
        assert_eq!(iter.next(), Some(("key05".into(), "value5".into())));
        assert_eq!(iter.next(), None);
        //
        db_map.sync_data().unwrap();
    }
    fn medium_test_map_string<T: DbMap<DbString>>(db_map: &mut T) {
        #[rustfmt::skip]
        const LOOP_MAX: u64 = if cfg!(miri) { 10 } else { 100 };
        // insert
        for i in 0..LOOP_MAX {
            let key = format!("key{i:02}");
            let value = format!("value{i}");
            db_map.put_string(&key, &value).unwrap();
        }
        assert_eq!(db_map.len().unwrap(), LOOP_MAX);
        // iterator
        let btmap: BTreeMap<DbString, Vec<u8>> = db_map.iter_mut().collect();
        let mut iter = btmap.into_iter();
        //let mut iter = db_map.iter_mut();
        for i in 0..LOOP_MAX {
            let key = format!("key{i:02}");
            let value = format!("value{i}");
            assert_eq!(iter.next(), Some((key.into(), value.as_bytes().to_vec())));
        }
        assert_eq!(iter.next(), None);
        //
        // iter on loop
        let btmap: BTreeMap<DbString, Vec<u8>> = db_map.iter_mut().collect();
        let iter = btmap.into_iter();
        //let mut iter = db_map.iter_mut();
        for (i, (k, v)) in (0_i32..).zip(iter) {
            let key = format!("key{i:02}");
            let value = format!("value{i}");
            assert_eq!(k, key.into());
            assert_eq!(v, value.as_bytes().to_vec());
        }
        //
        // into iter on loop
        //let mut iter = db_map.into_iter();
        //
        //db_map.sync_data().unwrap();
    }
    fn basic_test_map_dbint<T: DbMap<DbU64>>(db_map: &mut T) {
        // insert
        db_map.put_string(&12301, "value1").unwrap();
        db_map.put_string(&12302, "value2").unwrap();
        db_map.put_string(&12303, "value3").unwrap();
        db_map.put_string(&12304, "value4").unwrap();
        db_map.put_string(&12305, "value5").unwrap();
        assert_eq!(db_map.len().unwrap(), 5);
        // iterator
        let btmap: BTreeMap<DbU64, Vec<u8>> = db_map.iter_mut().collect();
        let mut iter = btmap.into_iter();
        //let mut iter = db_map.iter_mut();
        assert_eq!(iter.next(), Some((12301.into(), b"value1".to_vec())));
        assert_eq!(iter.next(), Some((12302.into(), b"value2".to_vec())));
        assert_eq!(iter.next(), Some((12303.into(), b"value3".to_vec())));
        assert_eq!(iter.next(), Some((12304.into(), b"value4".to_vec())));
        assert_eq!(iter.next(), Some((12305.into(), b"value5".to_vec())));
        assert_eq!(iter.next(), None);
        //
        db_map.sync_data().unwrap();
    }
    fn medium_test_map_dbint<T: DbMap<DbU64>>(db_map: &mut T) {
        #[rustfmt::skip]
        const LOOP_MAX: u64 = if cfg!(miri) { 10 } else { 100 };
        // insert
        for i in 0..LOOP_MAX {
            let key = 12300u64 + i;
            let value = format!("value{i}");
            db_map.put_string(&key, &value).unwrap();
        }
        assert_eq!(db_map.len().unwrap(), LOOP_MAX);
        // iterator
        let btmap: BTreeMap<DbU64, Vec<u8>> = db_map.iter_mut().collect();
        let mut iter = btmap.into_iter();
        //let mut iter = db_map.iter_mut();
        for i in 0..LOOP_MAX {
            let key = 12300u64 + i;
            let value = format!("value{i}");
            assert_eq!(iter.next(), Some((key.into(), value.as_bytes().to_vec())));
        }
        assert_eq!(iter.next(), None);
        //
        // iter on loop
        let btmap: BTreeMap<DbU64, Vec<u8>> = db_map.iter_mut().collect();
        let iter = btmap.into_iter();
        //let mut iter = db_map.iter_mut();
        for (i, (k, v)) in (0_i32..).zip(iter) {
            let key = 12300u64 + i as u64;
            let value = format!("value{i}");
            assert_eq!(k, key.into());
            assert_eq!(v, value.as_bytes().to_vec());
        }
        //
        // into iter on loop
        //let mut iter = db_map.into_iter();
        //
        //db_map.sync_data().unwrap();
    }
    fn basic_test_map_bytes<T: DbMap<DbBytes>>(db_map: &mut T) {
        // insert
        db_map.put_string(b"key01", "value1").unwrap();
        db_map.put_string(b"key02", "value2").unwrap();
        db_map.put_string(b"key03", "value3").unwrap();
        db_map.put_string(b"key04", "value4").unwrap();
        db_map.put_string(b"key05", "value5").unwrap();
        assert_eq!(db_map.len().unwrap(), 5);
        // iterator
        let btmap: BTreeMap<DbBytes, Vec<u8>> = db_map.iter_mut().collect();
        let mut iter = btmap.into_iter();
        //let mut iter = db_map.iter_mut();
        assert_eq!(iter.next(), Some((b"key01".into(), b"value1".to_vec())));
        assert_eq!(iter.next(), Some((b"key02".into(), b"value2".to_vec())));
        assert_eq!(iter.next(), Some((b"key03".into(), b"value3".to_vec())));
        assert_eq!(iter.next(), Some((b"key04".into(), b"value4".to_vec())));
        assert_eq!(iter.next(), Some((b"key05".into(), b"value5".to_vec())));
        assert_eq!(iter.next(), None);
        //
        db_map.sync_data().unwrap();
    }
    fn medium_test_map_bytes<T: DbMap<DbBytes>>(db_map: &mut T) {
        #[rustfmt::skip]
        const LOOP_MAX: u64 = if cfg!(miri) { 3 } else { 100 };
        // insert
        for i in 0..LOOP_MAX {
            let key = format!("key{i:02}");
            let value = format!("value{i}");
            db_map.put_string(&key, &value).unwrap();
        }
        assert_eq!(db_map.len().unwrap(), LOOP_MAX);
        // iterator
        let btmap: BTreeMap<DbBytes, Vec<u8>> = db_map.iter_mut().collect();
        let mut iter = btmap.into_iter();
        //let mut iter = db_map.iter_mut();
        for i in 0..LOOP_MAX {
            let key = format!("key{i:02}");
            let value = format!("value{i}");
            assert_eq!(iter.next(), Some((key.into(), value.as_bytes().to_vec())));
        }
        assert_eq!(iter.next(), None);
        //
        // iter on loop
        let btmap: BTreeMap<DbBytes, Vec<u8>> = db_map.iter_mut().collect();
        let iter = btmap.into_iter();
        //let mut iter = db_map.iter_mut();
        for (i, (k, v)) in (0_i32..).zip(iter) {
            let key = format!("key{i:02}");
            let value = format!("value{i}");
            assert_eq!(k, key.into());
            assert_eq!(v, value.as_bytes().to_vec());
        }
        //
        // into iter on loop
        //let mut iter = db_map.into_iter();
        //
        //db_map.sync_data().unwrap();
    }
    //
    #[cfg(not(miri))] // buggy on miri
    #[test]
    fn test_file_map_string() {
        let db_name = "target/tmp/test_iter-s.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db.db_map_string("some_string_1").unwrap();
        //
        iter_test_map_empty_iter(&mut db_map);
        //
        basic_test_map_string(&mut db_map);
        medium_test_map_string(&mut db_map);
    }
    #[cfg(not(miri))] // buggy on miri
    #[test]
    fn test_file_map_dbu64() {
        let db_name = "target/tmp/test_iter-u.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db.db_map_u64("some_u64_1").unwrap();
        //
        iter_test_map_empty_iter(&mut db_map);
        //
        basic_test_map_dbint(&mut db_map);
        medium_test_map_dbint(&mut db_map);
    }
    #[cfg(not(miri))] // buggy on miri
    #[test]
    fn test_file_map_bytes() {
        let db_name = "target/tmp/test_iter-b.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db.db_map_bytes("some_bytes_1").unwrap();
        //
        iter_test_map_empty_iter(&mut db_map);
        //
        basic_test_map_bytes(&mut db_map);
        medium_test_map_bytes(&mut db_map);
    }
}
