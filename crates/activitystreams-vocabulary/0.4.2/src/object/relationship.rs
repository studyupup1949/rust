use crate::{Item, Objects, create_object, field_access};

create_object! {
    /// Describes a relationship between two individuals.
    ///
    /// The `subject` and `object` properties are used to identify the connected individuals.
    Relationship: ObjectType {
        #[serde(skip_serializing_if = "Option::is_none")]
        subject: Option<Box<Item>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        relationship: Option<Box<Objects>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        object: Option<Box<Item>>,
    }
}

field_access! {
    Relationship<Vocab> {
        /// On a [Relationship] object, the `subject` property identifies one of the connected individuals.
        ///
        /// For instance, for a [Relationship] object describing "John is related to Sally", `subject` would refer to John.
        subject: option_box_deref { Item },
        /// On a [Relationship] object, the `relationship` property identifies the kind of relationship that exists between `subject` and `object`.
        relationship: option_box_deref { Objects },
        /// When used within a [Relationship], `object` describes the entity to which the `subject` is related.
        object: option_box_deref { Item },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name, Person};

    #[test]
    fn test_valid() {
        let summary = "Sally is an acquaintance of John";
        let subject_name = Name::try_from("Sally").unwrap();
        let relationship =
            Iri::try_from("http://purl.org/vocab/relationship/acquaintanceOf").unwrap();
        let object_name = Name::try_from("John").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Relationship",
  "summary": "{summary}",
  "subject": {{
    "type": "Person",
    "name": "{subject_name}"
  }},
  "relationship": "{relationship}",
  "object": {{
    "type": "Person",
    "name": "{object_name}"
  }}
}}"#
        );

        let subject = Person::new_inner().with_name(subject_name);
        let object = Person::new_inner().with_name(object_name);

        let relationship = Relationship::new()
            .with_summary(summary)
            .with_subject(subject)
            .with_relationship(relationship)
            .with_object(object);

        assert_eq!(
            serde_json::to_string_pretty(&relationship).unwrap(),
            json_str
        );
        assert_eq!(
            serde_json::from_str::<Relationship>(json_str.as_str()).unwrap(),
            relationship
        );
    }

    #[test]
    fn test_valid_tag() {
        let summary = "Sally is an acquaintance of John";
        let subject_name = Name::try_from("Sally").unwrap();
        let relationship =
            Iri::try_from("http://purl.org/vocab/relationship/acquaintanceOf").unwrap();
        let object_name = Name::try_from("John").unwrap();
        let tag = Iri::try_from("https://purl.org/vocab/tag").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Relationship",
  "summary": "{summary}",
  "tag": "{tag}",
  "subject": {{
    "type": "Person",
    "name": "{subject_name}"
  }},
  "relationship": "{relationship}",
  "object": {{
    "type": "Person",
    "name": "{object_name}"
  }}
}}"#
        );

        let subject = Person::new_inner().with_name(subject_name);
        let object = Person::new_inner().with_name(object_name);

        let relationship = Relationship::new()
            .with_summary(summary)
            .with_tag(tag)
            .with_subject(subject)
            .with_relationship(relationship)
            .with_object(object);

        assert_eq!(
            serde_json::to_string_pretty(&relationship).unwrap(),
            json_str
        );
        assert_eq!(
            serde_json::from_str::<Relationship>(json_str.as_str()).unwrap(),
            relationship
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Relationship>(json_str).is_err());
    }
}
