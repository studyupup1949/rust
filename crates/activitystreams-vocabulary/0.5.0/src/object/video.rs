use crate::create_object;

create_object! {
    /// Represents an video document of any kind.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Duration, Iri, Name, Video};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Puppy Plays With Ball").unwrap();
    /// let url = Iri::try_from("http://example.org/video.mkv").unwrap();
    /// let duration = Duration::try_from("PT2H").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Video",
    ///   "name": "{name}",
    ///   "url": "{url}",
    ///   "duration": "{duration}"
    /// }}"#
    ///     );
    ///
    /// let video = Video::new().with_name(name).with_url(url).with_duration(duration);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&video).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Video>(json_str.as_str()).unwrap(),
    ///     video
    /// );
    /// # }
    /// ```
    Video: ObjectType {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Duration, Iri, Name};

    #[test]
    fn test_valid() {
        let name = Name::try_from("Puppy Plays With Ball").unwrap();
        let url = Iri::try_from("http://example.org/video.mkv").unwrap();
        let duration = Duration::try_from("PT2H").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Video",
  "name": "{name}",
  "url": "{url}",
  "duration": "{duration}"
}}"#
        );

        let video = Video::new()
            .with_name(name)
            .with_url(url)
            .with_duration(duration);

        assert_eq!(serde_json::to_string_pretty(&video).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Video>(json_str.as_str()).unwrap(),
            video
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Video>(json_str).is_err());
    }
}
