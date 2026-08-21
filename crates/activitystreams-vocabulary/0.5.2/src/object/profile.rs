use crate::{Object, create_object, field_access};

create_object! {
    /// A [Profile] is a content object that describes another Object, typically used to describe Actor Type objects.
    ///
    /// The `describes` property is used to reference the object being described by the profile.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Name, Person, Profile};
    ///
    /// # fn main() {
    /// let summary = "Sally's Profile";
    /// let name = Name::try_from("Sally Smith").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Profile",
    ///   "summary": "{summary}",
    ///   "describes": {{
    ///     "type": "Person",
    ///     "name": "{name}"
    ///   }}
    /// }}"#
    ///      );
    ///
    /// let describes = Person::new_inner().with_name(name);
    /// let profile = Profile::new().with_summary(summary).with_describes(describes);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&profile).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Profile>(json_str.as_str()).unwrap(),
    ///     profile
    /// );
    /// # }
    /// ```
    Profile: ObjectType {
        #[serde(skip_serializing_if = "Option::is_none")]
        describes: Option<Box<Object>>,
    }
}

field_access! {
    Profile<Vocab> {
        /// On a [Profile] object, the `describes` property identifies the object described by the Profile.
        describes: option_box_deref { Object },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Object, Person};

    #[test]
    fn test_valid() {
        let summary = "Sally's Profile";
        let name = Name::try_from("Sally Smith").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Profile",
  "summary": "{summary}",
  "describes": {{
    "type": "Person",
    "name": "{name}"
  }}
}}"#
        );

        let describes = Person::new_inner().with_name(name);
        let profile = Profile::new()
            .with_summary(summary)
            .with_describes(describes);

        assert_eq!(serde_json::to_string_pretty(&profile).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Profile>(json_str.as_str()).unwrap(),
            profile
        );
    }

    #[test]
    fn test_valid_object() {
        let summary = "Sally's Profile";
        let name = Name::try_from("Sally Smith").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Profile",
  "summary": "{summary}",
  "describes": {{
    "type": "Person",
    "name": "{name}"
  }}
}}"#
        );

        let describes = Person::new_inner().with_name(name);
        let profile: Object = Profile::new()
            .with_summary(summary)
            .with_describes(describes)
            .into();

        assert_eq!(serde_json::to_string_pretty(&profile).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Object>(json_str.as_str()).unwrap(),
            profile
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Profile>(json_str).is_err());
    }
}
