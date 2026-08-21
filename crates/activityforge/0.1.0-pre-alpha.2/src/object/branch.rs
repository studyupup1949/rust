use activitystreams_vocabulary::{create_object, field_access};

create_object! {
    /// Represents a named variable reference to a version of the [Repository](crate::Repository).
    ///
    /// Typically used for committing changes in parallel to other development, and usually eventually merging the changes into the main history line.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Branch, context};
    /// use activitystreams_vocabulary::{Iri, Name};
    ///
    /// # fn main() {
    /// let id = Iri::try_from("https://example.dev/luke/myrepo/branches/master").unwrap();
    /// let name = Name::try_from("master").unwrap();
    /// let context = Iri::try_from("https://example.dev/luke/myrepo").unwrap();
    /// let refs = "refs/heads/master";
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Branch",
    ///   "id": "{id}",
    ///   "name": "{name}",
    ///   "context": "{context}",
    ///   "ref": "{refs}"
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let branch = Branch::new()
    ///     .with_context_property(context_property)
    ///     .with_id(id)
    ///     .with_name(name)
    ///     .with_context(context)
    ///     .with_refs(refs);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&branch).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Branch>(json_str.as_str()).unwrap(),
    ///     branch,
    /// );
    /// # }
    /// ```
    Branch: crate::ObjectType::Branch {
        #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
        refs: Option<String>,
    }
}

field_access! {
    Branch {
        /// Specifies an identifier for a [Branch], that is used in the [Repository](crate::Repository) to uniquely refer to it.
        ///
        /// For example, in Git, "refs/heads/master" would be the ref of the master branch.
        refs: option_deref { &str, String },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::{Iri, Name};

    #[test]
    fn test_branch() {
        let id = Iri::try_from("https://example.dev/luke/myrepo/branches/master").unwrap();
        let name = Name::try_from("master").unwrap();
        let context = Iri::try_from("https://example.dev/luke/myrepo").unwrap();
        let refs = "refs/heads/master";

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Branch",
  "id": "{id}",
  "name": "{name}",
  "context": "{context}",
  "ref": "{refs}"
}}"#
        );

        let context_property = context::forgefed_context();

        let branch = Branch::new()
            .with_context_property(context_property)
            .with_id(id)
            .with_name(name)
            .with_context(context)
            .with_refs(refs);

        assert_eq!(serde_json::to_string_pretty(&branch).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Branch>(json_str.as_str()).unwrap(),
            branch,
        );
    }
}
