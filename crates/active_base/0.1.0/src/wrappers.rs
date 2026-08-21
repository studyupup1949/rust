use std::collections::{BTreeMap, BTreeSet};
use std::ops::{DerefMut, Deref};
use std::cmp::Ordering;
use std::fmt::Debug;

use serde::{Serialize, Deserialize};
use serde::ser::Serializer;
use serde::Deserializer;

use super::{ActiveRecord, ActiveStruct, RawRecord, RecordType, RecordMut, ActiveMap, RecordRef};

macro_rules! wrapper {
    ($id:ident: $(($($bounds:tt)+)),*) => {
        impl<T: $( $($bounds)+ + )* Clone> Clone for $id<T> {
            fn clone(&self) -> Self {$id(self.0.clone())}
        }
        impl<T: $( $($bounds)+ + )* Debug> Debug for $id<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                self.0.fmt(f)
            }
        }

        impl<T: $( $($bounds)+ + )* Default> Default for $id<T> {
            fn default() -> Self {$id(T::default())}
        }
        impl<T: $( $($bounds)+ + )*> Deref for $id<T> {
            type Target = T;
            fn deref(&self) -> &Self::Target {&self.0}
        }
        impl<T: $( $($bounds)+ + )*> DerefMut for $id<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {&mut self.0}
        }

        impl<T: $( $($bounds)+ + )*> From<T> for $id<T> {
            fn from(serde: T) -> Self {$id(serde)}
        }

        impl<T: $( $($bounds)+ + )* Serialize> Serialize for $id<T> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }
        impl<'de, T: $( $($bounds)+ + )* Deserialize<'de>> Deserialize<'de> for $id<T> {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                Ok($id(T::deserialize(deserializer)?))
            }
        }
    };
}

#[derive(ActiveRecord)]
pub struct A<T>(pub T);
wrapper!(A: );

///Converts all Ordering::Greater to Ordering::Less
pub struct ReadOnly<T: ActiveRecord>(pub T);
wrapper!(ReadOnly: (ActiveRecord));

impl<T: ActiveRecord> ActiveRecord for ReadOnly<T> {
    fn name() -> String {T::name()}

    fn record_type() -> RecordType {T::record_type()}
    fn get_record_mut(&mut self) -> RecordMut<'_> {self.0.get_record_mut()}
    fn get_record_ref(&self) -> RecordRef<'_> {self.0.get_record_ref()}

    fn get_children_mut(&mut self) -> std::collections::BTreeMap<String, RecordMut<'_>> {
        self.0.get_children_mut()
    }

    fn get_children(&self) -> std::collections::BTreeMap<String, RecordRef<'_>> {
        self.0.get_children()
    }

    fn set_state(&mut self, state: &str) {self.0.set_state(state)}
    fn get_state(&self) -> Option<String> {
        self.0.get_state()
    }

    fn get_raw(&self) -> RawRecord {
        self.0.get_raw()
    }

    fn from_raw(raw: RawRecord) -> Self {
        ReadOnly(T::from_raw(raw))
    }
}

impl<T: ActiveStruct> ActiveStruct for ReadOnly<T> {
    fn get_self(&self) -> Option<String> {
        self.0.get_self()
    }

    fn set_self(&mut self, selfs: String) {
        self.0.set_self(selfs);
    }

    fn state(&self, other: Option<Option<&str>>) -> std::cmp::Ordering {
        let has_self = other.is_some();
        match self.0.state(other) {
            Ordering::Greater => if has_self {Ordering::Less} else {Ordering::Equal},
            o => o
        }
    }
}

impl<T: ActiveMap> ActiveMap for ReadOnly<T> {
    fn active_map_insert(&mut self, key: String, value: RawRecord) {
        self.0.active_map_insert(key, value);
    }
    fn active_map_remove(&mut self, key: String) {
        self.0.active_map_remove(key);
    }

    fn state(&self, other: Option<&str>, children: &BTreeSet<String>) -> (bool, BTreeMap<String, Ordering>) {
        let (_, mut instructions) = self.0.state(other, children);
        instructions.iter_mut().for_each(|(c, o)| if o == &Ordering::Greater {
            if children.contains(c) {
                //State is different but read instead of write to remote
                *o = Ordering::Less
            } else {
                //Don't write to empty remote
                *o = Ordering::Equal
            }
        });
        (false, instructions)
    }
}

pub struct Stateless<T: ActiveRecord>(pub T);
wrapper!(Stateless: (ActiveRecord));

impl<T: ActiveRecord> ActiveRecord for Stateless<T> {
    fn name() -> String {T::name()}

    fn record_type() -> RecordType {match T::record_type() {
        RecordType::Struct(state, children) => RecordType::Struct(state.map(|_| false), children),
        RecordType::Map(_, selfs) => RecordType::Map(false, selfs),
    }}

    fn get_record_mut(&mut self) -> RecordMut<'_> {self.0.get_record_mut()}
    fn get_record_ref(&self) -> RecordRef<'_> {self.0.get_record_ref()}

    fn get_children_mut(&mut self) -> std::collections::BTreeMap<String, RecordMut<'_>> {
        self.0.get_children_mut()
    }

    fn get_children(&self) -> std::collections::BTreeMap<String, RecordRef<'_>> {
        self.0.get_children()
    }

    fn set_state(&mut self, _state: &str) {}
    fn get_state(&self) -> Option<String> {None}

    fn get_raw(&self) -> RawRecord {
        match self.0.get_raw() {
            RawRecord::Map(_, children) => RawRecord::Map(None, children),
            RawRecord::Struct(innerstate, children) => RawRecord::Struct(innerstate.map(|(_, selfs)| (None, selfs)), children)
        }
    }

    fn from_raw(raw: RawRecord) -> Self {
        Stateless(T::from_raw(raw))
    }
}

impl<T: ActiveStruct> ActiveStruct for Stateless<T> {
    fn get_self(&self) -> Option<String> {
        self.0.get_self()
    }

    fn set_self(&mut self, selfs: String) {
        self.0.set_self(selfs);
    }

    fn state(&self, other: Option<Option<&str>>) -> std::cmp::Ordering {
        if other.is_none() {Ordering::Greater} else {Ordering::Equal}
    }
}

impl<T: ActiveMap> ActiveMap for Stateless<T> {
    fn active_map_insert(&mut self, key: String, value: RawRecord) {
        self.0.active_map_insert(key, value);
    }
    fn active_map_remove(&mut self, key: String) {
        self.0.active_map_remove(key);
    }

    fn state(&self, _other: Option<&str>, children: &BTreeSet<String>) -> (bool, BTreeMap<String, Ordering>) {
        let my_children = self.get_children();
         
        let mut map = BTreeMap::from_iter(children.iter().filter_map(|k|
            (!my_children.contains_key(k))
                .then_some((k.to_string(), Ordering::Less))
        ));
        map.extend(my_children.keys().map(|k| {
            (k.to_string(), match children.contains(&k.to_string()) {
                true => Ordering::Equal,
                false => Ordering::Greater
            })
        }));
        (false, map)
    }
}
