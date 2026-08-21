//! Device management.
use lifxi::http::prelude::*;

// TODO: Add result wrapper.
type Result = lifxi::http::ClientResult;

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum Type {
    /// A [LIFX](https://lifx.com) light bulb.
    ///
    /// LIFX devices are managed using [`lifxi`](https://github.com/Aehmlo/lifxi).
    #[serde(rename = "lifx")]
    LifxBulb { selector: Selector },
}

/// The bread and butter of the device manager.
#[derive(Deserialize)]
pub struct Device {
    /// The device type and any appropriate configuration.
    #[serde(flatten)]
    pub r#type: Type,
    /// The device name.
    pub name: String,
    /// A list of alternative names for the device.
    pub alternatives: Option<Vec<String>>,
}

impl Device {
    /// Changes the power state of the device.
    pub fn power(&self, on: bool, fast: bool) -> Result {
        match &self.r#type {
            Type::LifxBulb { selector } => crate::config::LIFX_CLIENT
                .select(selector.clone())
                .set_state()
                .power(on)
                .fast(fast)
                .send(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deserialize() {
        let device: Device =
            toml::from_str("type = \"lifx\"\nselector = \"label:Foo\"\nname = \"foo\"").unwrap();
        assert_eq!(
            device.r#type,
            Type::LifxBulb {
                selector: Selector::Label("Foo".to_owned())
            }
        );
        assert!(toml::from_str::<Device>("").is_err());
        assert!(toml::from_str::<Device>("type = \"lifx\"").is_err());
    }
}
