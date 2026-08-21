use crate::{ActorType, create_object, derived_kind_serde, impl_into_object};

create_object! {
    /// Represents a formal or informal collective of Actors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Group, Name};
    ///
    /// # fn main() {
    /// let group_name = Name::try_from("Big Beards of Austin").unwrap();
    /// let group = Group::new().with_name(group_name.clone());
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Group",
    ///   "name": "{group_name}"
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&group).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Group>(json_str.as_str()).unwrap(),
    ///     group
    /// );
    /// # }
    /// ```
    Group:
        #[serde(serialize_with = "obj_serde::ser")]
        ActorType {}
}

derived_kind_serde!(crate::ActorType::Group);
impl_into_object!(Group);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Name;

    #[test]
    fn test_valid() {
        let group_name = Name::try_from("Big Beards of Austin").unwrap();
        let group = Group::new().with_name(group_name.clone());

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Group",
  "name": "{group_name}"
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&group).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Group>(json_str.as_str()).unwrap(),
            group
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Group>(json_str).is_err());
    }
}
