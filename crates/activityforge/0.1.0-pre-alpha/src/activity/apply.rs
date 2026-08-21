use activitystreams_vocabulary::{create_activity, field_access};

use crate::GrantItem;

create_activity! {
    Apply: crate::ActivityType::Apply {
        capability: Option<GrantItem>,
    }
}

field_access! {
    Apply {
        /// Represents the [Grant](crate::Grant) capability of the `actor` who applied the merge request.
        capability: option_ref { GrantItem },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Branch, context};

    use activitystreams_vocabulary::{Iri, Name};

    #[test]
    fn test_apply() {
        let id = Iri::try_from("https://fig.fr33domlover.site/people/qn870/outbox/ZnqL0").unwrap();
        let actor = Iri::try_from("https://fig.fr33domlover.site/people/qn870").unwrap();
        let object =
            Iri::try_from("https://fig.fr33domlover.site/looms/9nOkn/cloths/mbWob/bundles/mbWob")
                .unwrap();
        let capability =
            Iri::try_from("https://fig.fr33domlover.site/looms/9nOkn/outbox/kDJx0").unwrap();

        let to0 = Iri::try_from("https://fig.fr33domlover.site/looms/9nOkn").unwrap();
        let to1 = Iri::try_from("https://fig.fr33domlover.site/people/qn870/followers").unwrap();
        let to2 = Iri::try_from("https://fig.fr33domlover.site/looms/9nOkn/followers").unwrap();
        let to3 = Iri::try_from("https://fig.fr33domlover.site/looms/9nOkn/cloths/mbWob/followers")
            .unwrap();

        let branch_context = Iri::try_from("https://fig.fr33domlover.site/repos/9nOkn").unwrap();
        let branch_name = Name::try_from("main").unwrap();
        let branch_refs = "/refs/heads/main";

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Apply",
  "id": "{id}",
  "to": [
    "{to0}",
    "{to1}",
    "{to2}",
    "{to3}"
  ],
  "actor": "{actor}",
  "object": "{object}",
  "target": {{
    "type": "Branch",
    "name": "{branch_name}",
    "context": "{branch_context}",
    "ref": "{branch_refs}"
  }},
  "capability": "{capability}"
}}"#
        );

        let context = context::forgefed_context();

        let branch = Branch::new_inner()
            .with_context(branch_context)
            .with_name(branch_name)
            .with_refs(branch_refs);

        let apply = Apply::new()
            .with_context_property(context)
            .with_id(id)
            .with_actor(actor)
            .with_object(object)
            .with_target(branch)
            .with_capability(capability)
            .with_to([to0, to1, to2, to3]);

        assert_eq!(serde_json::to_string_pretty(&apply).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Apply>(json_str.as_str()).unwrap(),
            apply
        );
    }
}
