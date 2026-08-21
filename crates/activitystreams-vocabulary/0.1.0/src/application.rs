use crate::{ActorType, create_object, derived_kind_serde, impl_into_object};

create_object! {
    /// Describes a software application.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Application, Name};
    ///
    /// # fn mainn() {
    /// let name =  Name::try_from("Exampletron 3000").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Application",
    ///   "name": "{name}"
    /// }}"#);
    ///
    /// let application = Application::new().with_name(name);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&application).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Application>(json_str.as_str()).unwrap(),
    ///     application
    /// );
    /// # }
    /// ```
    Application:
        #[serde(serialize_with = "obj_serde::ser")]
        ActorType {}
}

derived_kind_serde!(crate::ActorType::Application);
impl_into_object!(Application);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Name;

    #[test]
    fn test_valid() {
        let name = Name::try_from("Exampletron 3000").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Application",
  "name": "{name}"
}}"#
        );

        let application = Application::new().with_name(name);

        assert_eq!(
            serde_json::to_string_pretty(&application).unwrap(),
            json_str
        );
        assert_eq!(
            serde_json::from_str::<Application>(json_str.as_str()).unwrap(),
            application
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Application>(json_str).is_err());
    }
}
