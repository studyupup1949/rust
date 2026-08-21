use activitystreams_vocabulary::{ContentItem, DateTime, LinkItems, create_object, field_access};

use crate::Hash;

create_object! {
    /// Represents a named set of changes in the history of a [Repository](crate::Repository).
    ///
    /// This is called "commit" in Git, Mercurial and Monotone; "patch" in Darcs; sometimes called "change set".
    ///
    /// Note that [Commit] is a set of changes that already exists in a repo’s history, while a Patch is a separate proposed change set, that could be applied and pushed to a repo, resulting with a [Commit].
    Commit: crate::ObjectType::Commit {
        #[serde(skip_serializing_if = "Option::is_none")]
        hash: Option<Hash>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<ContentItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        committed_by: Option<LinkItems>,
        #[serde(skip_serializing_if = "Option::is_none")]
        created: Option<DateTime>,
        #[serde(skip_serializing_if = "Option::is_none")]
        committed: Option<DateTime>,
    }
}

field_access! {
    Commit {
        /// Represents the commit hash used to check the integrity of the commit contents.
        hash: option { Hash },
    }
}

field_access! {
    Commit {
        /// Represents the description of the commit changes.
        description: option_ref { ContentItem },
        /// Represents the date-time of when the commit was created.
        created: option_ref { DateTime },
        /// Represents the date-time of when the commit was committed.
        committed: option_ref { DateTime },
        /// Represents the actor who authored the commit.
        committed_by: option_ref { LinkItems },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Sha1Hash, context};

    use activitystreams_vocabulary::{Content, Iri, MimeType};

    #[test]
    fn test_commit() {
        let id = Iri::try_from(
            "https://example.dev/alice/myrepo/commits/109ec9a09c7df7fec775d2ba0b9d466e5643ec8c",
        )
        .unwrap();
        let context = Iri::try_from("https://example.dev/alice/myrepo").unwrap();
        let attributed_to = Iri::try_from("https://example.dev/bob").unwrap();
        let committed_by = Iri::try_from("https://example.dev/alice").unwrap();
        let hash = Sha1Hash::try_from("109ec9a09c7df7fec775d2ba0b9d466e5643ec8c").unwrap();
        let summary = "Add an installation script, fixes issue #89";

        let media_type = MimeType::TextPlain;
        let content = "It's about time people can install on their computers!";

        let created = "2019-07-11T12:34:56Z";
        let committed = "2019-07-26T23:45:01Z";

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Commit",
  "id": "{id}",
  "attributedTo": "{attributed_to}",
  "summary": "{summary}",
  "context": "{context}",
  "hash": "{hash}",
  "description": {{
    "content": "{content}",
    "mediaType": "{media_type}"
  }},
  "committedBy": "{committed_by}",
  "created": "{created}",
  "committed": "{committed}"
}}"#
        );

        let context_property = context::forgefed_context();

        let description = Content::new()
            .with_content(content)
            .with_media_type(media_type);

        let commit = Commit::new()
            .with_context_property(context_property)
            .with_id(id)
            .with_context(context)
            .with_attributed_to(attributed_to)
            .with_committed_by(committed_by)
            .with_hash(hash)
            .with_summary(summary)
            .with_description(description)
            .with_created(created.parse::<DateTime>().unwrap())
            .with_committed(committed.parse::<DateTime>().unwrap());

        assert_eq!(serde_json::to_string_pretty(&commit).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Commit>(json_str.as_str()).unwrap(),
            commit
        );
    }
}
