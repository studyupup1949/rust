use crate::create_activity;

create_activity! {
    /// Indicates that the `actor` has viewed the `object`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Article, Name, Person, View};
    ///
    /// # fn main() {
    /// let summary = "Sally read an article";
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let object_name = Name::try_from("What You Should Know About Activity Streams").unwrap();
    ///
    /// let json_str = format!(
    ///     r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "View",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": {{
    ///     "type": "Article",
    ///     "name": "{object_name}"
    ///   }}
    /// }}"#);
    ///
    /// let actor = Person::new_inner().with_name(actor_name);
    /// let object = Article::new_inner().with_name(object_name);
    /// let view = View::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&view).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<View>(json_str.as_str()).unwrap(),
    ///     view
    /// );
    /// # }
    /// ```
    View {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Article, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally read an article";
        let actor_name = Name::try_from("Sally").unwrap();
        let object_name = Name::try_from("What You Should Know About Activity Streams").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "View",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": {{
    "type": "Article",
    "name": "{object_name}"
  }}
}}"#
        );

        let actor = Person::new_inner().with_name(actor_name);
        let object = Article::new_inner().with_name(object_name);
        let view = View::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        assert_eq!(serde_json::to_string_pretty(&view).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<View>(json_str.as_str()).unwrap(),
            view
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<View>(json_str).is_err());
    }
}
