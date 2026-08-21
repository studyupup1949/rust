use std::collections::{BTreeSet, BTreeMap};
use std::fmt::{Display, Debug};
use std::str::FromStr;
use std::cmp::Ordering;

use super::{ActiveRecord, ActiveMap, RawRecord, RecordType, RecordMut, RecordRef};

impl<K: Ord + Display + FromStr<Err: Debug>, T: ActiveRecord> ActiveRecord for BTreeMap<K, T> {
    fn name() -> String {format!("BTreeMap<{}, {}>", std::any::type_name::<K>(), T::name())}

    fn record_type() -> RecordType {
        RecordType::Map(false, Box::new(T::record_type()))
    }

    fn get_raw(&self) -> RawRecord {
        RawRecord::Map(None,
            self.iter().map(|(k, v)| (k.to_string(), v.get_raw())).collect()
        )
    }

    fn from_raw(raw: RawRecord) -> Self {
        BTreeMap::from_iter(raw.map().1.into_iter().map(|(k, v)|
            (K::from_str(&k).unwrap(), T::from_raw(v))
        ))
    }

    fn set_state(&mut self, _state: &str) {}
    fn get_state(&self) -> Option<String> {None}

    fn get_record_mut(&mut self) -> RecordMut<'_> {
        RecordMut::Map(self)
    }

    fn get_children_mut(&mut self) -> BTreeMap<String, RecordMut<'_>> {
        self.iter_mut().map(|(k, v)| (k.to_string(), v.get_record_mut())).collect()
    }

    fn get_record_ref(&self) -> RecordRef<'_> {
        RecordRef::Map(self)
    }

    fn get_children(&self) -> BTreeMap<String, RecordRef<'_>> {
        self.iter().map(|(k, v)| (k.to_string(), v.get_record_ref())).collect()
    }
}

impl<K: Ord + Display + FromStr<Err: Debug>, T: ActiveRecord> ActiveMap for BTreeMap<K, T> {
    fn active_map_insert(&mut self, key: String, value: RawRecord) {
        self.insert(K::from_str(&key).unwrap(), T::from_raw(value));
    }
    fn active_map_remove(&mut self, key: String) {
        self.remove(&K::from_str(&key).unwrap());
    }

    fn state(&self, _other: Option<&str>, children: &BTreeSet<String>) -> (bool, BTreeMap<String, Ordering>) {
        let mut map = BTreeMap::from_iter(children.iter().filter_map(|k|
            (!self.contains_key(&K::from_str(k).unwrap()))
                .then_some((k.to_string(), Ordering::Less))
        ));
        map.extend(self.keys().map(|k| {
            (k.to_string(), match children.contains(&k.to_string()) {
                true => Ordering::Equal,
                false => Ordering::Greater
            })
        }));
        (false, map)
    }
}

impl<T: ActiveRecord> ActiveRecord for Vec<T> {
    fn name() -> String {"VecOf".to_string()+&T::name()}

    fn record_type() -> RecordType {
        RecordType::Map(false, Box::new(T::record_type()))
    }

    fn get_raw(&self) -> RawRecord {
        RawRecord::Map(None,
            self.iter().enumerate().map(|(k, v)| (k.to_string(), v.get_raw())).collect()
        )
    }

    fn from_raw(raw: RawRecord) -> Self {
        Vec::from_iter(raw.map().1.into_values().map(|r| T::from_raw(r)))
      //vec.sort_by_key(|a| a.0);
      //Vec::from_iter(vec.into_iter().map(|(_, v)|
      //    T::from_raw(v)
      //))
    }

    fn set_state(&mut self, _state: &str) {}
    fn get_state(&self) -> Option<String> {None}

    fn get_record_mut(&mut self) -> RecordMut<'_> {
        RecordMut::Map(self)
    }

    fn get_record_ref(&self) -> RecordRef<'_> {
        RecordRef::Map(self)
    }

    fn get_children_mut(&mut self) -> BTreeMap<String, RecordMut<'_>> {
        self.iter_mut().enumerate().map(|(k, v)| (k.to_string(), v.get_record_mut())).collect()
    }

    fn get_children(&self) -> BTreeMap<String, RecordRef<'_>> {
        self.iter().enumerate().map(|(k, v)| (k.to_string(), v.get_record_ref())).collect()
    }
}

impl<T: ActiveRecord> ActiveMap for Vec<T> {
    fn active_map_insert(&mut self, key: String, value: RawRecord) {
        self.insert(key.parse().unwrap(), T::from_raw(value));
    }
    fn active_map_remove(&mut self, key: String) {
        self.remove(key.parse().unwrap());
    }

    fn state(&self, _other: Option<&str>, children: &BTreeSet<String>) -> (bool, BTreeMap<String, Ordering>) {
        let self_contains_key = |k: &str| self.len() > k.parse().unwrap();
        let mut map = BTreeMap::from_iter(children.iter().filter_map(|k|
            (!self_contains_key(k))
                .then_some((k.to_string(), Ordering::Less))
        ));
        map.extend(self.iter().enumerate().map(|(k, _)| {
            (k.to_string(), match children.contains(&k.to_string()) {
                true => Ordering::Equal,
                false => Ordering::Greater
            })
        }));
        (false, map)
    }
}
