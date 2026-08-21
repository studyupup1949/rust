use super::Hasher;
use crate::aaml::AAML;

#[cfg(feature = "serde")]
impl serde::Serialize for AAML {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AAML", 2)?;
        state.serialize_field("map", &self.map)?;
        state.serialize_field("schemas", &self.schemas)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AAML {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct AAMLData {
            map: std::collections::HashMap<Box<str>, Box<str>, Hasher>,
            schemas: std::collections::HashMap<String, crate::commands::schema::SchemaDef>,
        }

        let data = AAMLData::deserialize(deserializer)?;
        let mut aaml = AAML::new();
        *aaml.get_map_mut() = data.map;
        *aaml.get_schemas_mut() = data.schemas;
        Ok(aaml)
    }
}
