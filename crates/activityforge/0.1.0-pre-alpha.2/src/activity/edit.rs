use activitystreams_vocabulary::create_activity;

create_activity! {
    /// Indicates that `actor` is editing or asking to edit the object specified by `object`.
    ///
    /// In that `object`, `id` **MUST** be specified, and every other field is a field being set to a new value.
    Edit: crate::ActivityType::Edit {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;
    use activitystreams_vocabulary::{Iri, Object};

    #[test]
    fn test_edit() {
        let id = Iri::try_from("http://dev.example/people/JABld/outbox/AZ01d").unwrap();
        let actor = Iri::try_from("http://dev.example/people/JABld").unwrap();

        let object_id = Iri::try_from("http://dev.example/people/JABld/messages/PAQlD").unwrap();
        let object_source = "Hello world";
        let object_content = "<p>Hello world</p>";

        let to0 = Iri::try_from("http://dev.example/decks/ld6zd").unwrap();
        let to1 = Iri::try_from("http://dev.example/people/JABld/followers").unwrap();
        let to2 = Iri::try_from("http://dev.example/decks/ld6zd/followers").unwrap();
        let to3 = Iri::try_from("http://dev.example/decks/ld6zd/tickets/kd9ed/followers").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Edit",
  "id": "{id}",
  "to": [
    "{to0}",
    "{to1}",
    "{to2}",
    "{to3}"
  ],
  "actor": "{actor}",
  "object": {{
    "type": "Object",
    "id": "{object_id}",
    "content": "{object_content}",
    "source": "{object_source}"
  }}
}}"#
        );
        let context = context::forgefed_context();

        let object = Object::new_inner()
            .with_id(object_id)
            .with_source(object_source)
            .with_content(object_content);

        let to = [to0, to1, to2, to3];

        let edit = Edit::new()
            .with_context_property(context)
            .with_id(id)
            .with_actor(actor)
            .with_object(object)
            .with_to(to);

        assert_eq!(serde_json::to_string_pretty(&edit).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Edit>(json_str.as_str()).unwrap(),
            edit
        );
    }
}
