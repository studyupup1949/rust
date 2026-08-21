use std::{error::Error, io};

use serde::{ser::Serialize, de::DeserializeOwned};
use serde_json::{Serializer, ser::Formatter, Value, de::{Read, StrRead}, Deserializer, de::{IoRead, SliceRead}};

use crate::model;

use self::model_conv::ModelConv;

mod model_conv;

pub trait JsonSerde where Self: Sized {
    fn read_json<'de, R: Read<'de>>(deserializer: Deserializer<R>) -> Result<Self, Box<dyn Error>>;
    fn write_json<W: io::Write, F: Formatter>(&self, serializer: &mut Serializer<W, F>) -> Result<(), Box<dyn Error>>;

    fn from_value(value: &Value) -> Result<Self, Box<dyn Error>>;
    fn to_value(&self) -> Result<Value, Box<dyn Error>>;

    fn from_json_reader<'de, R: Read<'de>>(reader: R) -> Result<Self, Box<dyn Error>> {
        let de = Deserializer::new(reader);
        Self::read_json(de)
    }

    fn io_read_json<R: io::Read>(reader: R) -> Result<Self, Box<dyn Error>> {
        Self::from_json_reader(IoRead::new(reader))
    }

    fn io_write_json<W: io::Write>(&self, writer: W) -> Result<(), Box<dyn Error>> {
        let mut ser = Serializer::new(writer);
        self.write_json(&mut ser)
    }

    fn io_write_pretty_json<W: io::Write>(&self, writer: W) -> Result<(), Box<dyn Error>> {
        let mut ser = Serializer::pretty(writer);
        self.write_json(&mut ser)
    }

    fn from_json_bytes<'a>(bytes: &'a [u8]) -> Result<Self, Box<dyn Error>> {
        Self::from_json_reader(SliceRead::new(bytes))
    }

    fn to_json_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut writer = Vec::with_capacity(128);
        self.io_write_json(&mut writer)?;
        Ok(writer)
    }

    fn to_json_bytes_pretty(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut writer = Vec::with_capacity(128);
        self.io_write_pretty_json(&mut writer)?;
        Ok(writer)
    }

    fn from_json_str<'a>(str: &'a str) -> Result<Self, Box<dyn Error>> {
        Self::from_json_reader(StrRead::new(str))
    }

    fn to_json_string(&self) -> Result<String, Box<dyn Error>> {
        let bytes = self.to_json_bytes()?;
        Ok(unsafe {
            String::from_utf8_unchecked(bytes)
        })
    }

    fn to_string_pretty(&self) -> Result<String, Box<dyn Error>> {
        let bytes = self.to_json_bytes_pretty()?;
        Ok(unsafe {
            String::from_utf8_unchecked(bytes)
        })
    }
}

pub struct SerdeJsonValue<T> {
    pub value: T,
}

impl<T: Serialize + DeserializeOwned> SerdeJsonValue<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
        }
    }
}

impl<T: Serialize + DeserializeOwned> JsonSerde for SerdeJsonValue<T> {
    fn read_json<'de, R: Read<'de>>(mut deserializer: Deserializer<R>) -> Result<Self, Box<dyn Error>> {
        let value: T = serde::de::Deserialize::deserialize(&mut deserializer)?;
        deserializer.end()?;
        Ok(SerdeJsonValue { value })
    }

    fn write_json<W: io::Write, F: Formatter>(&self, serializer: &mut Serializer<W, F>) -> Result<(), Box<dyn Error>> {
        self.value.serialize(serializer)?;
        Ok(())
    }

    fn from_value(value: &Value) -> Result<Self, Box<dyn Error>> {
        let value: T = serde::de::Deserialize::deserialize(value)?;
        Ok(SerdeJsonValue { value })
    }

    fn to_value(&self) -> Result<Value, Box<dyn Error>> {
        Ok(serde_json::to_value(&self.value)?)
    }
}

impl JsonSerde for model::Context {
    fn read_json<'de, R: Read<'de>>(mut deserializer: Deserializer<R>) -> Result<Self, Box<dyn Error>> {
        let value: <Self as ModelConv>::JsonSerdeValue = serde::de::Deserialize::deserialize(&mut deserializer)?;
        deserializer.end()?;
        Ok(ModelConv::to_model(value)?)
    }

    fn write_json<W: io::Write, F: Formatter>(&self, serializer: &mut Serializer<W, F>) -> Result<(), Box<dyn Error>> {
        let value = self.from_model()?;
        value.serialize(serializer)?;
        Ok(())
    }

    fn from_value(value: &Value) -> Result<Self, Box<dyn Error>> {
        let value: <Self as ModelConv>::JsonSerdeValue = serde::de::Deserialize::deserialize(value)?;
        Ok(ModelConv::to_model(value)?)
    }

    fn to_value(&self) -> Result<Value, Box<dyn Error>> {
        Ok(serde_json::to_value(self.from_model()?)?)
    }
}

impl JsonSerde for model::Object {
    fn read_json<'de, R: Read<'de>>(mut deserializer: Deserializer<R>) -> Result<Self, Box<dyn Error>> {
        let value: <Self as ModelConv>::JsonSerdeValue = serde::de::Deserialize::deserialize(&mut deserializer)?;
        deserializer.end()?;
        Ok(ModelConv::to_model(value)?)
    }

    fn write_json<W: io::Write, F: Formatter>(&self, serializer: &mut Serializer<W, F>) -> Result<(), Box<dyn Error>> {
        let value = self.from_model()?;
        value.serialize(serializer)?;
        Ok(())
    }

    fn from_value(value: &Value) -> Result<Self, Box<dyn Error>> {
        let value: <Self as ModelConv>::JsonSerdeValue = serde::de::Deserialize::deserialize(value)?;
        Ok(ModelConv::to_model(value)?)
    }

    fn to_value(&self) -> Result<Value, Box<dyn Error>> {
        Ok(serde_json::to_value(self.from_model()?)?)
    }
}

impl JsonSerde for model::Link {
    fn read_json<'de, R: Read<'de>>(mut deserializer: Deserializer<R>) -> Result<Self, Box<dyn Error>> {
        let value: <Self as ModelConv>::JsonSerdeValue = serde::de::Deserialize::deserialize(&mut deserializer)?;
        deserializer.end()?;
        Ok(ModelConv::to_model(value)?)
    }

    fn write_json<W: io::Write, F: Formatter>(&self, serializer: &mut Serializer<W, F>) -> Result<(), Box<dyn Error>> {
        let value = self.from_model()?;
        value.serialize(serializer)?;
        Ok(())
    }

    fn from_value(value: &Value) -> Result<Self, Box<dyn Error>> {
        let value: <Self as ModelConv>::JsonSerdeValue = serde::de::Deserialize::deserialize(value)?;
        Ok(ModelConv::to_model(value)?)
    }

    fn to_value(&self) -> Result<Value, Box<dyn Error>> {
        Ok(serde_json::to_value(self.from_model()?)?)
    }
}

impl JsonSerde for model::ObjectOrLink {
    fn read_json<'de, R: Read<'de>>(mut deserializer: Deserializer<R>) -> Result<Self, Box<dyn Error>> {
        let value: <Self as ModelConv>::JsonSerdeValue = serde::de::Deserialize::deserialize(&mut deserializer)?;
        deserializer.end()?;
        Ok(ModelConv::to_model(value)?)
    }

    fn write_json<W: io::Write, F: Formatter>(&self, serializer: &mut Serializer<W, F>) -> Result<(), Box<dyn Error>> {
        let value = self.from_model()?;
        value.serialize(serializer)?;
        Ok(())
    }

    fn from_value(value: &Value) -> Result<Self, Box<dyn Error>> {
        let value: <Self as ModelConv>::JsonSerdeValue = serde::de::Deserialize::deserialize(value)?;
        Ok(ModelConv::to_model(value)?)
    }

    fn to_value(&self) -> Result<Value, Box<dyn Error>> {
        Ok(serde_json::to_value(self.from_model()?)?)
    }
}
