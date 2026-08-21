use activitystreams_vocabulary::{create_activity, field_access};

use crate::GrantItem;

create_activity! {
    /// Indicates that the `actor` likes, recommends or endorses the `object`.
    ///
    /// The `target` and `origin` typically have no defined meaning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::Like;
    /// use activitystreams_vocabulary::{Iri, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally liked a note";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let object = Iri::try_from("http://example.org/notes/1").unwrap();
    /// let grant = Iri::try_from("http://example.org/sally/grants/1").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Like",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": "{object}",
    ///   "capability": "{grant}"
    /// }}"#);
    ///
    /// let like = Like::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object)
    ///     .with_capability(grant);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&like).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Like>(json_str.as_str()).unwrap(),
    ///     like
    /// );
    /// # }
    /// ```
    Like {
        #[serde(skip_serializing_if = "Option::is_none")]
        capability: Option<GrantItem>,
    }
}

field_access! {
    Like {
        /// Specifies a previously published [Grant](crate::Grant) activity providing relevant access permissions.
        ///
        /// See [ForgeFed/#capability](https://forgefed.org/spec/#capability) for details.
        capability: option_ref { GrantItem },
    }
}
