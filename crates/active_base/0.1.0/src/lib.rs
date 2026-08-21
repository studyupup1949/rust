use std::collections::{BTreeSet, BTreeMap};
use std::cmp::Ordering;
use std::fmt::Debug;

mod maps;

#[cfg(feature = "wrappers")]
mod wrappers;
#[cfg(feature = "wrappers")]
pub use wrappers::*;

pub use active_proc::ActiveRecord;

pub trait Parent {fn parent(&self) -> &[&str];}
impl Parent for &[&str] {fn parent(&self) -> &[&str] {&self[..self.len()-1]}}

pub enum RecordRef<'a> {
    Struct(&'a dyn ActiveStruct),
    Map(&'a dyn ActiveMap)
}

pub enum RecordMut<'a> {
    Struct(&'a mut dyn ActiveStruct),
    Map(&'a mut dyn ActiveMap)
}

impl<'a> RecordMut<'a> {
    pub fn get_raw(&self) -> RawRecord {
        match self {
            RecordMut::Struct(s) => s.get_raw(),
            RecordMut::Map(m) => m.get_raw()
        }
    }
}

type SelfState = Option<(Option<String>, String)>;

#[derive(Debug)]
pub enum RawRecord {
    Map(Option<String>, BTreeMap<String, Self>),
    Struct(Option<(Option<String>, String)>, BTreeMap<String, Self>)
}
impl RawRecord {
    pub fn map(self) -> (Option<String>, BTreeMap<String, Self>) {
        if let RawRecord::Map(state, inner) = self {(state, inner)} else {panic!("Not a map");}
    }

    pub fn r#struct(self) -> (SelfState, BTreeMap<String, Self>) {
        if let RawRecord::Struct(innerstate, map) = self {(innerstate, map)} else {panic!("Not a struct");}
    }
}

#[derive(Debug)]
pub enum RecordType {
    //Fewer keys can map to many types
    Struct(Option<bool>, BTreeMap<String, Self>),//Option has self properties, bool has state
    //Many keys but to a single type
    Map(bool, Box<Self>),//Has state
}

impl RecordType {
    pub fn get(&self, path: &[&str]) -> Result<&Self, String> {
        if path.len() < 2 {return Ok(self);}
        self.get_child(path.get(1).unwrap()).ok_or(path[1..].join("/")).and_then(|c| c.get(&path[1..]))
    }

    pub fn get_child(&self, name: &str) -> Option<&Self> {
        match self {
            RecordType::Struct(_, map) => map.get(name),
            RecordType::Map(_, map) => Some(map),
        }
    }
}

pub trait ActiveRecord {
    fn name() -> String where Self: Sized;
    fn record_type() -> RecordType where Self: Sized;

    ///From raw should not fail if no State is provided even if this is a stateful object.
    ///Provide a default state in such a case
    fn from_raw(raw: RawRecord) -> Self where Self: Sized;
    fn get_raw(&self) -> RawRecord;

    fn set_state(&mut self, state: &str);
    fn get_state(&self) -> Option<String>;

    fn get_record_mut(&mut self) -> RecordMut<'_>;
    fn get_children_mut(&mut self) -> BTreeMap<String, RecordMut<'_>>;
    fn get_record_ref(&self) -> RecordRef<'_>;
    fn get_children(&self) -> BTreeMap<String, RecordRef<'_>>;
}

pub trait ActiveMap: ActiveRecord {
    fn active_map_insert(&mut self, key: String, value: RawRecord);
    fn active_map_remove(&mut self, key: String);
    fn state(&self, other: Option<&str>, children: &BTreeSet<String>) -> (bool, BTreeMap<String, Ordering>);
}

pub trait ActiveStruct: ActiveRecord {
    fn get_self(&self) -> Option<String>;
    fn set_self(&mut self, selfs: String);
    ///Other is None if no object exists at the other location.
    ///If Ordering::Less is returned there must be an existing object at the location to read.
    fn state(&self, other: Option<Option<&str>>) -> Ordering;
}

//  impl Serialize for dyn ActiveRecord {
//      fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
//          self.serialize(serializer)
//      }
//  }

//  impl<'de, T: $( $($bounds)+ + )* Deserialize<'de>> Deserialize<'de> for $id<T> {
//      fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
//          Ok($id(T::deserialize(deserializer)?))
//      }
//  }

//  use serde::{Serialize, Deserialize};
//  impl<T: Serialize + for<'a> Deserialize<'a>> ActiveRecord for T { 
//      fn name() -> String {std::any::type_name::<T>().to_string()}

//      fn record_type() -> RecordType {
//          RecordType::Struct(Some(true), std::collections::BTreeMap::default())
//      }

//      fn get_record_mut(&mut self) -> RecordMut<'_> {RecordMut::Struct(self)}
//      fn get_record_ref(&self) -> RecordRef<'_> {RecordRef::Struct(self)}
//      fn get_children_mut(&mut self) -> std::collections::BTreeMap<String, RecordMut<'_>> {std::collections::BTreeMap::default()}
//      fn get_children(&self) -> std::collections::BTreeMap<String, RecordRef<'_>> {std::collections::BTreeMap::default()}
//      fn set_state(&mut self, _state: &str) {}
//      fn get_state(&self) -> Option<String> {
//          let mut s = std::hash::DefaultHasher::new();
//          std::hash::Hasher::write(&mut s, self.get_self().unwrap().as_bytes());
//          Some(format!("{:#x}", std::hash::Hasher::finish(&s)))
//      }

//      fn get_raw(&self) -> RawRecord {
//          RawRecord::Struct(self.get_self().map(|s| (self.get_state(), s)), std::collections::BTreeMap::default())
//      }

//      fn from_raw(raw: RawRecord) -> Self {
//          serde_json::from_str(&raw.r#struct().0.unwrap().1).unwrap()
//      }
//  }

//  impl<T: Serialize + for<'a> Deserialize<'a>> ActiveStruct for T {
//      fn get_self(&self) -> Option<String> {
//          Some(serde_json::to_string(&self).unwrap())
//      }

//      fn set_self(&mut self, selfs: String) {
//          *self = serde_json::from_str(&selfs).unwrap();
//      }

//      fn state(&self, other: Option<Option<&str>>) -> std::cmp::Ordering {
//          match other {
//              None => std::cmp::Ordering::Greater,
//              Some(other) if other != self.get_state().as_deref() => std::cmp::Ordering::Greater,
//              _ => std::cmp::Ordering::Equal
//          }
//      }
//  }
