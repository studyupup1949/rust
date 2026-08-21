//
// these tests implemente from:
// ref.) https://github.com/rust-lang/rust/blob/master/library/std/src/collections/hash/map/tests.rs
//
mod test_dbvu64 {
    use abyssiniandb::filedb::{FileDbParams, HashBucketsParam};
    use abyssiniandb::{DbMap, DbXxx, DbXxxBase};
    //use std::cell::RefCell;
    //
    #[test]
    #[should_panic]
    fn test_create_capacity_zero() {
        let db_name = "target/tmp/test_dbvu64/test_create_capacity_zero.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let _db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(0),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    #[test]
    fn test_insert() {
        let db_name = "target/tmp/test_dbvu64/test_insert.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        assert_eq!(db_map.len().unwrap(), 0);
        db_map.put(&1, &[2]).unwrap();
        assert_eq!(db_map.len().unwrap(), 1);
        db_map.put(&2, &[4]).unwrap();
        assert_eq!(db_map.len().unwrap(), 2);
        //
        assert_eq!(db_map.get(&1).unwrap(), Some(vec![2]));
        assert_eq!(db_map.get(&2).unwrap(), Some(vec![4]));
    }
    #[test]
    fn test_clone() {
        let db_name = "target/tmp/test_dbvu64/test_clone.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        assert_eq!(db_map.len().unwrap(), 0);
        db_map.put(&1, &[2]).unwrap();
        assert_eq!(db_map.len().unwrap(), 1);
        db_map.put(&2, &[4]).unwrap();
        assert_eq!(db_map.len().unwrap(), 2);
        //
        let mut db_map2 = db_map.clone();
        //
        assert_eq!(db_map2.get(&1).unwrap(), Some(vec![2]));
        assert_eq!(db_map2.get(&2).unwrap(), Some(vec![4]));
        assert_eq!(db_map2.len().unwrap(), 2);
    }
    /* #[test] fn test_empty_entry() {
        let db_name = "target/tmp/test_dbvu64/test_empty_entry.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        match db_map.entry(0) {
            Occupied(_) => panic!(),
            Vacant(_) => {}
        }
        assert!(*db_map.entry(0).or_insert(&[1]));
        assert_eq!(db_map.len(), 1);
    }
    */
    #[test]
    fn test_empty_iter() {
        let db_name = "target/tmp/test_dbvu64/test_empty_iter.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        assert_eq!(db_map.len().unwrap(), 0);
        assert!(db_map.is_empty().unwrap());
        //
        //assert_eq!(db_map.drain().next(), None);
        assert_eq!(db_map.keys().next(), None);
        assert_eq!(db_map.values().next(), None);
        //assert_eq!(db_map.values_mut().next(), None);
        assert_eq!(db_map.iter().next(), None);
        assert_eq!(db_map.iter_mut().next(), None);
        //assert_eq!(db_map.into_iter().next(), None);
    }
    #[cfg(feature = "large_test")]
    #[test]
    fn test_lots_of_insertions() {
        let db_name = "target/tmp/test_dbvu64/test_lots_of_insertions.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(10000),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        // Try this a few times to make sure we never screw up the hashmap's
        // internal state.
        for _ in 0..10 {
            assert!(db_map.is_empty().unwrap());
            //
            for i in 1..1001 {
                db_map.put(&i, &i.to_le_bytes()).unwrap();
                //
                for j in 1..=i {
                    let r = db_map.get(&j).unwrap();
                    assert_eq!(r, Some(j.to_le_bytes().to_vec()));
                }
                for j in i + 1..1001 {
                    let r = db_map.get(&j).unwrap();
                    assert_eq!(r, None);
                }
            }
            for i in 1001..2001 {
                assert!(!db_map.includes_key(&i).unwrap());
            }
            //
            // remove forwards
            for i in 1..1001 {
                assert!(db_map.delete(&i).unwrap().is_some());
                for j in 1..=i {
                    assert!(!db_map.includes_key(&j).unwrap());
                }
                for j in i + 1..1001 {
                    assert!(db_map.includes_key(&j).unwrap());
                }
            }
            for i in 1..1001 {
                assert!(!db_map.includes_key(&i).unwrap());
            }
            //
            for i in 1..1001 {
                db_map.put(&i, &i.to_le_bytes()).unwrap();
            }
            //
            // remove backwards
            for i in (1..1001).rev() {
                assert!(db_map.delete(&i).unwrap().is_some());
                for j in i..1001 {
                    assert!(!db_map.includes_key(&j).unwrap());
                }
                for j in 1..i {
                    assert!(db_map.includes_key(&j).unwrap());
                }
            }
        }
    }
    /* #[test] fn test_find_mut() {
        let mut m = HashMap::new();
        assert!(m.insert(1, 12).is_none());
        assert!(m.insert(2, 8).is_none());
        assert!(m.insert(5, 14).is_none());
        let new = 100;
        match m.get_mut(&5) {
            None => panic!(),
            Some(x) => *x = new,
        }
        assert_eq!(m.get(&5), Some(&new));
    }
    */
    /* #[test] fn test_insert_overwrite() {
        let mut m = HashMap::new();
        assert!(m.insert(1, 2).is_none());
        assert_eq!(*m.get(&1).unwrap(), 2);
        assert!(!m.insert(1, 3).is_none());
        assert_eq!(*m.get(&1).unwrap(), 3);
    }
    */
    #[test]
    fn test_insert_conflicts() {
        let db_name = "target/tmp/test_dbvu64/test_insert_conflicts.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        db_map.put(&1, &[2_u8]).unwrap();
        db_map.put(&5, &[3_u8]).unwrap();
        db_map.put(&9, &[4_u8]).unwrap();
        //
        assert_eq!(db_map.get(&9).unwrap(), Some(vec![4_u8]));
        assert_eq!(db_map.get(&5).unwrap(), Some(vec![3_u8]));
        assert_eq!(db_map.get(&1).unwrap(), Some(vec![2_u8]));
    }
    #[test]
    fn test_delete_conflicts() {
        let db_name = "target/tmp/test_dbvu64/test_delete_conflicts.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        db_map.put(&1, &[2_u8]).unwrap();
        assert_eq!(db_map.get(&1).unwrap(), Some(vec![2_u8]));
        //
        db_map.put(&5, &[3_u8]).unwrap();
        assert_eq!(db_map.get(&1).unwrap(), Some(vec![2_u8]));
        assert_eq!(db_map.get(&5).unwrap(), Some(vec![3_u8]));
        //
        db_map.put(&9, &[4_u8]).unwrap();
        assert_eq!(db_map.get(&1).unwrap(), Some(vec![2_u8]));
        assert_eq!(db_map.get(&5).unwrap(), Some(vec![3_u8]));
        assert_eq!(db_map.get(&9).unwrap(), Some(vec![4_u8]));
        //
        assert_eq!(db_map.delete(&1).unwrap(), Some(vec![2_u8]));
        assert_eq!(db_map.get(&9).unwrap(), Some(vec![4_u8]));
        assert_eq!(db_map.get(&5).unwrap(), Some(vec![3_u8]));
        assert_eq!(db_map.get(&1).unwrap(), None);
    }
    #[test]
    fn test_is_empty() {
        let db_name = "target/tmp/test_dbvu64/test_is_empty.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        db_map.put(&1, &[2_u8]).unwrap();
        assert!(!db_map.is_empty().unwrap());
        assert_eq!(db_map.delete(&1).unwrap(), Some(vec![2_u8]));
        assert!(db_map.is_empty().unwrap());
    }
    #[test]
    fn test_delete() {
        let db_name = "target/tmp/test_dbvu64/test_delete.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        db_map.put(&1, &[2_u8]).unwrap();
        assert!(!db_map.is_empty().unwrap());
        assert_eq!(db_map.delete(&1).unwrap(), Some(vec![2_u8]));
        assert_eq!(db_map.delete(&1).unwrap(), None);
    }
    /* #[test] fn test_remove_entry() {
        let mut m = HashMap::new();
        m.insert(1, 2);
        assert_eq!(m.remove_entry(&1), Some((1, 2)));
        assert_eq!(m.remove(&1), None);
    }
    */
    #[test]
    fn test_iterate() {
        let db_name = "target/tmp/test_dbvu64/test_iterate.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        for i in 0..32 {
            db_map.put(&i, &[i as u8 * 2]).unwrap();
        }
        assert_eq!(db_map.len().unwrap(), 32);
        //
        let mut observed: u32 = 0;
        for (k, v) in &db_map {
            let n: u64 = k.into();
            assert_eq!(v[0], n as u8 * 2);
            observed |= 1 << n;
        }
        assert_eq!(observed, 0xFFFF_FFFF);
    }
    #[test]
    fn test_keys() {
        let db_name = "target/tmp/test_dbvu64/test_keys.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        let xs = [(1, b'a'), (2, b'b'), (3, b'c')];
        db_map
            .put_from_iter(xs.iter().map(|&(k, v)| (k.into(), vec![v])))
            .unwrap();
        assert_eq!(db_map.len().unwrap(), 3);
        //
        let keys: Vec<u64> = db_map.keys().map(|i| i.into()).collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&1));
        assert!(keys.contains(&2));
        assert!(keys.contains(&3));
    }
    #[test]
    fn test_values() {
        let db_name = "target/tmp/test_dbvu64/test_values.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        let xs = [(1, b'a'), (2, b'b'), (3, b'c')];
        db_map
            .put_from_iter(xs.iter().map(|&(k, v)| (k.into(), vec![v])))
            .unwrap();
        assert_eq!(db_map.len().unwrap(), 3);
        //
        let values: Vec<String> = db_map
            .values()
            .map(|v| String::from_utf8_lossy(&v).to_string())
            .collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&"a".to_string()));
        assert!(values.contains(&"b".to_string()));
        assert!(values.contains(&"c".to_string()));
    }
    /* #[test] fn test_values_mut() {
        let pairs = [(1, 1), (2, 2), (3, 3)];
        let mut map: HashMap<_, _> = pairs.into_iter().collect();
        for value in map.values_mut() {
            *value = (*value) * 2
        }
        let values: Vec<_> = map.values().cloned().collect();
        assert_eq!(values.len(), 3);
        assert!(values.contains(&2));
        assert!(values.contains(&4));
        assert!(values.contains(&6));
    }
    #[test]
    fn test_into_keys() {
        let pairs = [(1, 'a'), (2, 'b'), (3, 'c')];
        let map: HashMap<_, _> = pairs.into_iter().collect();
        let keys: Vec<_> = map.into_keys().collect();

        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&1));
        assert!(keys.contains(&2));
        assert!(keys.contains(&3));
    }
    #[test]
    fn test_into_values() {
        let pairs = [(1, 'a'), (2, 'b'), (3, 'c')];
        let map: HashMap<_, _> = pairs.into_iter().collect();
        let values: Vec<_> = map.into_values().collect();

        assert_eq!(values.len(), 3);
        assert!(values.contains(&'a'));
        assert!(values.contains(&'b'));
        assert!(values.contains(&'c'));
    }
    */
    #[test]
    fn test_find() {
        let db_name = "target/tmp/test_dbvu64/test_find.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        assert_eq!(db_map.get(&1).unwrap(), None);
        db_map.put(&1, &[2_u8]).unwrap();
        match db_map.get(&1).unwrap() {
            None => panic!(),
            Some(v) => assert_eq!(*v, vec![2_u8]),
        }
    }
    /* #[test] fn test_eq() {
        let db_name = "target/tmp/test_dbvu64/test_find.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map1 = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        let mut db_map2 = db
            .db_map_vu64_with_params(
                "some_u64_2",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        db_map1.put(&1, &[2_u8]).unwrap();
        db_map1.put(&2, &[3_u8]).unwrap();
        db_map1.put(&3, &[4_u8]).unwrap();
        //
        db_map2.put(&1, &[2_u8]).unwrap();
        db_map2.put(&2, &[3_u8]).unwrap();
        //
        assert!(db_map1 != db_map2);
        //
        db_map2.put(&3, &[4_u8]).unwrap();
        //
        assert_eq!(db_map1, db_map2);
    }
    */
    /* #[test] fn test_show() {
        let db_name = "target/tmp/test_dbvu64/test_show.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map1 = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        let mut db_map2 = db
            .db_map_vu64_with_params(
                "some_u64_2",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        db_map1.put(&1, &[2_u8]).unwrap();
        db_map1.put(&3, &[4_u8]).unwrap();
        //
        let map_str = format!("{:?}", db_map1);
        assert_eq!(map_str, "");
        assert!(map_str == "{1: 2, 3: 4}" || map_str == "{3: 4, 1: 2}");
        assert_eq!(format!("{:?}", db_map2), "{}");
    }
    */
    /* #[test] fn test_reserve_shrink_to_fit() {
        let mut m = HashMap::new();
        m.insert(0, 0);
        m.remove(&0);
        assert!(m.capacity() >= m.len());
        for i in 0..128 {
            m.insert(i, i);
        }
        m.reserve(256);

        let usable_cap = m.capacity();
        for i in 128..(128 + 256) {
            m.insert(i, i);
            assert_eq!(m.capacity(), usable_cap);
        }

        for i in 100..(128 + 256) {
            assert_eq!(m.remove(&i), Some(i));
        }
        m.shrink_to_fit();

        assert_eq!(m.len(), 100);
        assert!(!m.is_empty());
        assert!(m.capacity() >= m.len());

        for i in 0..100 {
            assert_eq!(m.remove(&i), Some(i));
        }
        m.shrink_to_fit();
        m.insert(0, 0);

        assert_eq!(m.len(), 1);
        assert!(m.capacity() >= m.len());
        assert_eq!(m.remove(&0), Some(0));
    }
    */
    #[test]
    fn test_put_from_iter() {
        let db_name = "target/tmp/test_dbvu64/test_from_iter.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        let xs = [(1, 1), (2, 2), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];
        db_map
            .put_from_iter(xs.iter().map(|&(k, v)| (k.into(), vec![v as u8])))
            .unwrap();
        assert_eq!(db_map.len().unwrap(), 6);
        //
        for &(k, v) in &xs {
            assert_eq!(db_map.get(&k).unwrap(), Some(vec![v as u8]));
        }
        assert_eq!(db_map.len().unwrap() as usize, xs.len() - 1);
    }
    #[test]
    fn test_size_hint() {
        let db_name = "target/tmp/test_dbvu64/test_size_hint.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];
        db_map
            .put_from_iter(xs.iter().map(|&(k, v)| (k.into(), vec![v as u8])))
            .unwrap();
        assert_eq!(db_map.len().unwrap(), 6);
        //
        let mut iter = db_map.iter();
        for _ in iter.by_ref().take(3) {}
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }
    #[test]
    fn test_iter_len() {
        let db_name = "target/tmp/test_dbvu64/test_iter_len.abyssiniandb";
        let _ = std::fs::remove_dir_all(db_name);
        let db = abyssiniandb::open_file(db_name).unwrap();
        let mut db_map = db
            .db_map_vu64_with_params(
                "some_vu64_1",
                FileDbParams {
                    buckets_size: HashBucketsParam::Capacity(4),
                    ..Default::default()
                },
            )
            .unwrap();
        //
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];
        db_map
            .put_from_iter(xs.iter().map(|&(k, v)| (k.into(), vec![v as u8])))
            .unwrap();
        assert_eq!(db_map.len().unwrap(), 6);
        //
        let mut iter = db_map.iter();
        for _ in iter.by_ref().take(3) {}
        assert_eq!(iter.len(), 3);
    }
    /* #[test] fn test_mut_size_hint() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];
        let mut map: HashMap<_, _> = xs.iter().cloned().collect();
        let mut iter = map.iter_mut();
        for _ in iter.by_ref().take(3) {}
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }
    */
    /* #[test] fn test_iter_mut_len() {
        let xs = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)];
        let mut map: HashMap<_, _> = xs.iter().cloned().collect();
        let mut iter = map.iter_mut();
        for _ in iter.by_ref().take(3) {}
        assert_eq!(iter.len(), 3);
    }
    */
    /* #[test] fn test_index() {
        let mut map = HashMap::new();
        map.insert(1, 2);
        map.insert(2, 1);
        map.insert(3, 4);
        assert_eq!(map[&2], 1);
    }
    */
}
