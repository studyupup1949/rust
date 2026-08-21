use crate::{ActorType, create_object, derived_kind_serde, impl_into_object};

create_object! {
    /// Represents an individual person.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Name, Person};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Sally Smith").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Person",
    ///   "name": "{name}"
    /// }}"#
    ///     );
    ///
    /// let person = Person::new().with_name(name);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&person).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Person>(json_str.as_str()).unwrap(),
    ///     person
    /// );
    /// # }
    /// ```
    Person:
        #[serde(serialize_with = "obj_serde::ser")]
        ActorType {}
}

derived_kind_serde!(crate::ActorType::Person);
impl_into_object!(Person);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Name;

    #[test]
    fn test_valid() {
        let name = Name::try_from("Sally Smith").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Person",
  "name": "{name}"
}}"#
        );

        let person = Person::new().with_name(name);

        assert_eq!(serde_json::to_string_pretty(&person).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Person>(json_str.as_str()).unwrap(),
            person
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Person>(json_str).is_err());
    }
}
