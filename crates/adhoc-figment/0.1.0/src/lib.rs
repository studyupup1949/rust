#![doc = include_str!("../README.md")]

use figment::{
    self,
    value::{Dict, Map, Tag, Value},
    Metadata, Profile, Provider,
};

/// Provides a single given value.
pub struct AdHocProvider {
    dict: Dict,
}

impl AdHocProvider {
    pub fn new<Path: AsRef<str>, Val: Into<Value>>(
        path: Path,
        value: Val,
    ) -> AdHocProvider {
        let mut value = value.into();
        let path = path.as_ref();
        let mut dict = Dict::new();
        for segment in path.rsplit('.') {
            dict.insert(segment.into(), value);
            value = Value::Dict(Tag::default(), dict);
            dict = Dict::new();
        }
        AdHocProvider {
            dict: value.as_dict().unwrap().clone(),
        }
    }
}

impl Provider for AdHocProvider {
    fn metadata(&self) -> Metadata {
        Metadata::named("Ad Hoc Provider")
    }

    fn data(&self) -> std::result::Result<Map<Profile, Dict>, figment::Error> {
        let mut map = Map::new();
        map.insert("default".into(), self.dict.clone());
        Ok(map)
    }

    fn profile(&self) -> Option<Profile> {
        Some("default".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use figment::Figment;

    #[test]
    fn it_works() {
        let provider = AdHocProvider::new("some.path", "value");
        let figment = Figment::from(provider);
        assert_eq!(
            figment.extract_inner::<String>("some.path").unwrap(),
            "value"
        );
    }
}
