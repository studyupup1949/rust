use crate::{ActorType, create_object, derived_kind_serde, impl_into_object};

create_object! {
    /// Represents a service of any kind.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Name, Service};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Acme Web Service").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Service",
    ///   "name": "{name}"
    /// }}"#
    ///     );
    ///
    /// let service = Service::new().with_name(name);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&service).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Service>(json_str.as_str()).unwrap(),
    ///     service
    /// );
    /// # }
    /// ```
    Service:
        #[serde(serialize_with = "obj_serde::ser")]
        ActorType {}
}

derived_kind_serde!(crate::ActorType::Service);
impl_into_object!(Service);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Name;

    #[test]
    fn test_valid() {
        let name = Name::try_from("Acme Web Service").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Service",
  "name": "{name}"
}}"#
        );

        let service = Service::new().with_name(name);

        assert_eq!(serde_json::to_string_pretty(&service).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Service>(json_str.as_str()).unwrap(),
            service
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Service>(json_str).is_err());
    }
}
