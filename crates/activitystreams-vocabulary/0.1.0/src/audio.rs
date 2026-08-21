use crate::{ObjectType, create_object, derived_kind_serde};

create_object! {
    /// Represents an audio document of any kind.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Audio, Iri, Link, MimeType, Name};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Interview With A Famous Technologist").unwrap();
    /// let url_href = Iri::try_from("http://example.org/podcast.mp3").unwrap();
    /// let media_type = MimeType::AudioMp3;
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Audio",
    ///   "name": "{name}",
    ///   "url": {{
    ///     "type": "Link",
    ///     "href": "{url_href}",
    ///     "mediaType": "{media_type}"
    ///   }}
    /// }}"#);
    ///
    /// let url = Link::new_inner()
    ///     .with_href(url_href)
    ///     .with_media_type(media_type);
    /// let audio = Audio::new().with_name(name).with_url(url);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&audio).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Audio>(json_str.as_str()).unwrap(),
    ///     audio
    /// );
    /// # }
    /// ```
    Audio:
        #[serde(serialize_with = "obj_serde::ser")]
        ObjectType {}
}

derived_kind_serde!(crate::ObjectType::Audio);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Link, MimeType, Name};

    #[test]
    fn test_valid() {
        let name = Name::try_from("Interview With A Famous Technologist").unwrap();
        let url_href = Iri::try_from("http://example.org/podcast.mp3").unwrap();
        let media_type = MimeType::AudioMp3;

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Audio",
  "name": "{name}",
  "url": {{
    "type": "Link",
    "href": "{url_href}",
    "mediaType": "{media_type}"
  }}
}}"#
        );

        let url = Link::new_inner()
            .with_href(url_href)
            .with_media_type(media_type);
        let audio = Audio::new().with_name(name).with_url(url);

        assert_eq!(serde_json::to_string_pretty(&audio).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Audio>(json_str.as_str()).unwrap(),
            audio
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Audio>(json_str).is_err());
    }
}
