use crate::{ActorType, create_object, derived_kind_serde, impl_into_object};

create_object! {
    /// Represents an organization.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Name, Organization};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Example Co.").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Organization",
    ///   "name": "{name}"
    /// }}"#
    ///     );
    ///
    /// let organization = Organization::new().with_name(name);
    ///
    /// assert_eq!(
    ///     serde_json::to_string_pretty(&organization).unwrap(),
    ///     json_str
    /// );
    /// assert_eq!(
    ///     serde_json::from_str::<Organization>(json_str.as_str()).unwrap(),
    ///     organization
    /// );
    /// # }
    /// ```
    Organization:
        #[serde(serialize_with = "obj_serde::ser")]
        ActorType {}
}

derived_kind_serde!(crate::ActorType::Organization);
impl_into_object!(Organization);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Name;

    #[test]
    fn test_valid() {
        let name = Name::try_from("Example Co.").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Organization",
  "name": "{name}"
}}"#
        );

        let organization = Organization::new().with_name(name);

        assert_eq!(
            serde_json::to_string_pretty(&organization).unwrap(),
            json_str
        );
        assert_eq!(
            serde_json::from_str::<Organization>(json_str.as_str()).unwrap(),
            organization
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Organization>(json_str).is_err());
    }
}
