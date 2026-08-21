use crate::create_activity;

create_activity! {
    /// Indicates that the `actor` has added the `object` to the target.
    ///
    /// If the target property is not explicitly specified, the target would need to be determined implicitly by context.
    ///
    /// The origin can be used to identify the context from which the object originated.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Add, Collection, Image, Iri, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally added a picture of her cat to her cat picture collection";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let object_name = Name::try_from("A picture of my cat").unwrap();
    /// let object_url = Iri::try_from("http://example.org/img/cat.png").unwrap();
    /// let object = Image::new_inner().with_name(object_name.clone()).with_url(object_url.clone());
    ///
    /// let origin_name = Name::try_from("Camera Roll").unwrap();
    /// let origin = Collection::new_inner().with_name(origin_name.clone());
    ///
    /// let target_name = Name::try_from("My Cat Pictures").unwrap();
    /// let target = Collection::new_inner().with_name(target_name.clone());
    ///
    /// let add = Add::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object)
    ///     .with_origin(origin)
    ///     .with_target(target);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Add",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": {{
    ///     "type": "Image",
    ///     "name": "{object_name}",
    ///     "url": "{object_url}"
    ///   }},
    ///   "origin": {{
    ///     "type": "Collection",
    ///     "name": "{origin_name}"
    ///   }},
    ///   "target": {{
    ///     "type": "Collection",
    ///     "name": "{target_name}"
    ///   }}
    /// }}"#);
    ///
    /// println!("{}", serde_json::to_string_pretty(&add).unwrap());
    /// println!("{json_str}");
    ///
    /// assert_eq!(serde_json::to_string_pretty(&add).unwrap(), json_str);
    /// assert_eq!(serde_json::from_str::<Add>(json_str.as_str()).unwrap(), add);
    /// # }
    /// ```
    Add {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Collection, Image, Iri, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally added a picture of her cat to her cat picture collection";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let object_name = Name::try_from("A picture of my cat").unwrap();
        let object_url = Iri::try_from("http://example.org/img/cat.png").unwrap();
        let object = Image::new_inner()
            .with_name(object_name.clone())
            .with_url(object_url.clone());

        let origin_name = Name::try_from("Camera Roll").unwrap();
        let origin = Collection::new_inner().with_name(origin_name.clone());

        let target_name = Name::try_from("My Cat Pictures").unwrap();
        let target = Collection::new_inner().with_name(target_name.clone());

        let add = Add::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object)
            .with_origin(origin)
            .with_target(target);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Add",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": {{
    "type": "Image",
    "name": "{object_name}",
    "url": "{object_url}"
  }},
  "origin": {{
    "type": "Collection",
    "name": "{origin_name}"
  }},
  "target": {{
    "type": "Collection",
    "name": "{target_name}"
  }}
}}"#
        );

        println!("{}", serde_json::to_string_pretty(&add).unwrap());
        println!("{json_str}");

        assert_eq!(serde_json::to_string_pretty(&add).unwrap(), json_str);
        assert_eq!(serde_json::from_str::<Add>(json_str.as_str()).unwrap(), add);
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Add>(json_str).is_err());
    }
}
