use crate::{
    bindings::exports::konall::achan::value::{GuestValue, OwnValue, ValueTy},
    value::Value,
};

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use wit_bindgen::Resource;

impl GuestValue for Value {
    fn ty(&self) -> ValueTy {
        match self {
            Self::Null => ValueTy::Null,
            Self::Boolean(_) => ValueTy::Boolean,
            Self::Number(_) => ValueTy::Number,
            Self::String(_) => ValueTy::String,
            Self::List(_) => ValueTy::List,
            Self::Map(_) => ValueTy::Map,
        }
    }

    fn null() -> OwnValue {
        OwnValue::new(Self::Null)
    }

    fn boolean(v: bool) -> OwnValue {
        OwnValue::new(Self::Boolean(v))
    }

    fn number(v: f64) -> OwnValue {
        OwnValue::new(Self::Number(v))
    }

    fn string(v: String) -> OwnValue {
        OwnValue::new(Self::String(v))
    }

    fn list(v: Vec<OwnValue>) -> OwnValue {
        OwnValue::new(Self::List(
            v.into_iter().map(Resource::into_inner).collect(),
        ))
    }

    fn map(v: Vec<(String, OwnValue)>) -> OwnValue {
        OwnValue::new(Self::Map(
            v.into_iter()
                .map(|(k, x)| (k, Resource::into_inner(x)))
                .collect(),
        ))
    }

    fn as_null(&self) -> Option<Result<(), ()>> {
        self.as_null().map(|_| Ok(()))
    }

    fn as_boolean(&self) -> Option<bool> {
        self.as_boolean()
    }

    fn as_number(&self) -> Option<f64> {
        self.as_number()
    }

    fn as_string(&self) -> Option<String> {
        self.as_str().map(|s| s.to_owned())
    }

    fn as_list(&self) -> Option<Vec<OwnValue>> {
        self.as_slice()
            .map(|v| v.into_iter().map(|x| OwnValue::new(x.clone())).collect())
    }

    fn as_map(&self) -> Option<Vec<(String, OwnValue)>> {
        self.as_map().map(|v| {
            v.into_iter()
                .map(|(k, x)| (k.to_owned(), OwnValue::new(x.clone())))
                .collect()
        })
    }
}
