use crate::{
    CoreType, create_link, derived_kind_serde, impl_into_item, impl_into_items,
    impl_into_ordered_items,
};

create_link! {
    /// Represents an ActivityStream [Link](https://www.w3.org/TR/activitystreams-vocabulary/#dfn-link).
    ///
    /// A `Link` is an indirect, qualified reference to a resource identified by a URL.
    ///
    /// The fundamental model for links is established by [RFC5988](https://www.rfc-editor.org/rfc/rfc5988).
    ///
    /// Many of the properties defined by the Activity Vocabulary allow values that are either instances of
    /// `Object` or `Link`.
    ///
    /// When a `Link` is used, it establishes a qualified relation connecting the subject (the containing object)
    /// to the resource identified by the `href`.
    ///
    /// Properties of the Link are properties of the reference as opposed to properties of the resource.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, LanguageTag, Link, MimeType, Name};
    ///
    /// # fn main() {
    /// let name = Name::try_from("An example link").unwrap();
    /// let href = Iri::try_from("http://example.org/abc").unwrap();
    /// let hreflang = LanguageTag::try_from("en").unwrap();
    /// let media_type = MimeType::TextHtml;
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Link",
    ///   "href": "{href}",
    ///   "name": "{name}",
    ///   "hreflang": "{hreflang}",
    ///   "mediaType": "{media_type}"
    /// }}"#
    ///         );
    ///
    /// let link = Link::new()
    ///     .with_href(href)
    ///     .with_hreflang(hreflang)
    ///     .with_name(name)
    ///     .with_media_type(media_type);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&link).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Link>(json_str.as_str()).unwrap(),
    ///     link
    /// );
    /// # }
    /// ```
    Link:
        #[serde(serialize_with = "obj_serde::ser")]
        CoreType {}
}

derived_kind_serde!(crate::CoreType::Link);

impl_into_item!(Link, link);
impl_into_items!(Link);
impl_into_ordered_items!(Link);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, LanguageTag, MimeType, Name};

    #[test]
    fn test_link_required() {
        let name = Name::try_from("An example link").unwrap();
        let href = Iri::try_from("http://example.org/abc").unwrap();
        let hreflang = LanguageTag::try_from("en").unwrap();
        let media_type = MimeType::TextHtml;

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Link",
  "href": "{href}",
  "name": "{name}",
  "hreflang": "{hreflang}",
  "mediaType": "{media_type}"
}}"#
        );

        let link = Link::new()
            .with_href(href)
            .with_hreflang(hreflang)
            .with_name(name)
            .with_media_type(media_type);

        assert_eq!(serde_json::to_string_pretty(&link).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Link>(json_str.as_str()).unwrap(),
            link
        );
    }

    #[test]
    fn test_link_full() {
        let href = Iri::try_from("http://example.org/abc").unwrap();
        let name = Name::try_from("An example link").unwrap();
        let rel = Iri::try_from("http://exampl.org/abc/relation#test").unwrap();
        let hreflang = LanguageTag::try_from("en").unwrap();
        let media_type = MimeType::TextHtml;
        let height = 1u64;
        let width = 1u64;

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Link",
  "href": "{href}",
  "name": "{name}",
  "rel": "{rel}",
  "hreflang": "{hreflang}",
  "mediaType": "{media_type}",
  "height": {height},
  "width": {width}
}}"#
        );

        let link = Link::new()
            .with_href(href)
            .with_name(name)
            .with_media_type(media_type)
            .with_rel(rel)
            .with_hreflang(hreflang)
            .with_height(height)
            .with_width(width);

        println!(
            "{}\n{}",
            serde_json::to_string_pretty(&link).unwrap(),
            json_str
        );
        assert_eq!(serde_json::to_string_pretty(&link).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Link>(json_str.as_str()).unwrap(),
            link
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Link>(json_str).is_err());
        assert!(
            serde_json::to_string(&Link::new().with_kind(CoreType::Object.to_vocabulary_types()))
                .is_err()
        );
    }
}
