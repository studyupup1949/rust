/// Helper macro to define ActivityStream Object & Object-derived types.
#[macro_export]
macro_rules! create_object {
    (
        $(#[$doc:meta])*
        $ty:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::create_object! {
            $ty: $crate::CoreType::$ty {
                $(
                    $(#[$field_serde])*
                    $field: $field_ty,
                )*
            }
        }
    };

    (
        $(#[$doc:meta])*
        $ty:ident:
        $(#[$vocab_serde:meta])?
        $vocab_ty:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::create_object! {
            $(#[$doc])*
            $ty:
            $(#[$vocab_serde])?
            $crate::$vocab_ty::$ty {
            $(
                $(#[$field_serde])*
                $field: $field_ty,
            )*
        }}
    };

    (
        $(#[$doc:meta])*
        $ty:ident:
        $(#[$vocab_serde:meta])?
        $vocab_path:ident :: $vocab_ty:ident :: $vocab_var:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $(#[$doc])*
        #[derive(Clone, Debug, Eq, PartialEq, $crate::serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        pub struct $ty<Vocab: $crate::ActivityVocabulary = $crate::VocabularyTypes> {
            #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
            pub(crate) context_property: Option<$crate::Context>,
            #[serde(rename = "type")]
            $(#[$vocab_serde])?
            pub(crate) kind: Vocab,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) id: Option<$crate::Iri>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) name: Option<$crate::Name>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) name_map: Option<$crate::NameMap>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) attributed_to: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) audience: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) summary: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) summary_map: Option<$crate::LanguageMap>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) content: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) content_map: Option<$crate::LanguageMap>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) context: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) generator: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) icon: Option<Box<$crate::ImageItem>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) image: Option<Box<$crate::ImageItem>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) in_reply_to: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) location: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) url: Option<Box<$crate::IriItem>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) preview: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) replies: Option<Box<$crate::Collection>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) tag: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) to: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) bto: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) cc: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) bcc: Option<Box<$crate::Item>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) media_type: Option<$crate::MimeType>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) start_time: Option<$crate::DateTime>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) end_time: Option<$crate::DateTime>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) published: Option<$crate::DateTime>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) updated: Option<$crate::DateTime>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub(crate) duration: Option<$crate::Duration>,
            #[serde(flatten, skip_serializing_if = "Option::is_none")]
            pub(crate) extra_fields: Option<Box<$crate::Map<String, $crate::serde_json::Value>>>,
            $(
                $(#[$field_serde])*
                pub(crate) $field: $field_ty,
            )*
        }

        impl<Vocab: $crate::ActivityVocabulary + From<$vocab_path::$vocab_ty>>  $ty<Vocab> {
            $crate::paste! {
                #[doc = "Creates a new [" $ty "]."]
                pub fn new() -> Self {
                    Self::new_kind(Vocab::from($vocab_path::$vocab_ty::$vocab_var))
                }

                #[doc = "Creates a new [" $ty "] for use as an inner member of another object."]
                #[doc = ""]
                #[doc = "Encodes the type without the `@context` field."]
                pub fn new_inner() -> Self {
                    Self::new_kind(Vocab::from($vocab_path::$vocab_ty::$vocab_var))
                        .withoutcontext_property_property()
                }

                #[doc = "Creates a new [" $ty "]."]
                pub fn new_kind(kind: Vocab) -> Self {
                    Self {
                        context_property: Some($crate::Context::new()),
                        kind,
                        id: None,
                        name: None,
                        name_map: None,
                        attributed_to: None,
                        audience: None,
                        summary: None,
                        summary_map: None,
                        content: None,
                        content_map: None,
                        context: None,
                        generator: None,
                        icon: None,
                        image: None,
                        in_reply_to: None,
                        location: None,
                        url: None,
                        preview: None,
                        replies: None,
                        tag: None,
                        to: None,
                        bto: None,
                        cc: None,
                        bcc: None,
                        media_type: None,
                        start_time: None,
                        end_time: None,
                        published: None,
                        updated: None,
                        duration: None,
                        extra_fields: None,
                        $(
                            $field: Default::default(),
                        )*
                    }
                }

                /// Builder function that unsets the `@context` field.
                pub fn withoutcontext_property_property(self) -> Self {
                    Self {
                        context_property: None,
                        ..self
                    }
                }
            }
        }

        $crate::object_field_access!($ty);
        $crate::impl_default!($ty);
        $crate::impl_display!($ty, json);
        $crate::derive_object! {
            $(#[$vocab_serde])?
            $ty { $( $field )* }
        }
    };
}

#[macro_export]
macro_rules! derive_object {
    (
        $(#[$vocab_serde:meta])?
        $ty:ident {
        $(
            $field:ident $(,)?
        )*
    }) => {
        impl<'de, T: $crate::ActivityVocabulary> $crate::serde::de::Deserialize<'de> for $ty<T> {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
                where D: $crate::serde::de::Deserializer<'de>,
            {
                use ::core::marker::PhantomData;
                use $crate::serde::de;

                struct Visitor<T>(PhantomData<T>);

                impl<'vde, VT: $crate::ActivityVocabulary> de::Visitor<'vde> for Visitor<VT> {
                    type Value = $ty<VT>;

                    fn expecting(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        f.write_str(stringify!($ty<VT>))
                    }

                    fn visit_map<V>(self, mut map: V) -> ::core::result::Result<Self::Value, V::Error>
                        where V: de::MapAccess<'vde>,
                    {
                        #[allow(unused)]
                        use $crate::heck::ToLowerCamelCase;

                        let mut context_property = None;
                        let mut kind = None;
                        let mut id = None;
                        let mut name = None;
                        let mut name_map = None;
                        let mut attributed_to = None;
                        let mut audience = None;
                        let mut summary = None;
                        let mut summary_map = None;
                        let mut content = None;
                        let mut content_map = None;
                        let mut context = None;
                        let mut generator = None;
                        let mut icon = None;
                        let mut image = None;
                        let mut in_reply_to = None;
                        let mut location = None;
                        let mut url = None;
                        let mut preview = None;
                        let mut replies = None;
                        let mut tag = None;
                        let mut to = None;
                        let mut bto = None;
                        let mut cc = None;
                        let mut bcc = None;
                        let mut media_type = None;
                        let mut start_time = None;
                        let mut end_time = None;
                        let mut published = None;
                        let mut updated = None;
                        let mut duration = None;
                        let mut extra_fields: Option<Box<$crate::Map<String, $crate::serde_json::Value>>> = None;
                        $(
                        let mut $field = None;
                        )*

                        while let Some(key) = map.next_key::<String>()? {
                            match key.as_str() {
                                "@context" => {
                                    if context_property.is_some() {
                                        return Err(de::Error::duplicate_field("@context"));
                                    }
                                    context_property = Some(map.next_value()?);
                                }
                                "type" => {
                                    if kind.is_some() {
                                        return Err(de::Error::duplicate_field("type"));
                                    }

                                    kind = Some(map.next_value()?);
                                }
                                "id" => {
                                    if id.is_some() {
                                        return Err(de::Error::duplicate_field("id"));
                                    }
                                    id = Some(map.next_value()?);
                                }
                                "name" => {
                                    if name.is_some() {
                                        return Err(de::Error::duplicate_field("name"));
                                    }
                                    name = Some(map.next_value()?);
                                }
                                "nameMap" => {
                                    if name_map.is_some() {
                                        return Err(de::Error::duplicate_field("nameMap"));
                                    }
                                    name_map = Some(map.next_value()?);
                                }
                                "attributedTo" => {
                                    if attributed_to.is_some() {
                                        return Err(de::Error::duplicate_field("attributedTo"));
                                    }
                                    attributed_to = Some(map.next_value()?);
                                }
                                "audience" => {
                                    if audience.is_some() {
                                        return Err(de::Error::duplicate_field("audience"));
                                    }
                                    audience = Some(map.next_value()?);
                                }
                                "content" => {
                                    if content.is_some() {
                                        return Err(de::Error::duplicate_field("content"));
                                    }
                                    content = Some(map.next_value()?);
                                }
                                "contentMap" => {
                                    if content_map.is_some() {
                                        return Err(de::Error::duplicate_field("contentMap"));
                                    }
                                    content_map = Some(map.next_value()?);
                                }
                                "summary" => {
                                    if summary.is_some() {
                                        return Err(de::Error::duplicate_field("summary"));
                                    }
                                    summary = Some(map.next_value()?);
                                }
                                "summaryMap" => {
                                    if summary_map.is_some() {
                                        return Err(de::Error::duplicate_field("summaryMap"));
                                    }
                                    summary_map = Some(map.next_value()?);
                                }
                                "context" => {
                                    if context.is_some() {
                                        return Err(de::Error::duplicate_field("context"));
                                    }
                                    context = Some(map.next_value()?);
                                }
                                "generator" => {
                                    if generator.is_some() {
                                        return Err(de::Error::duplicate_field("generator"));
                                    }
                                    generator = Some(map.next_value()?);
                                }
                                "icon" => {
                                    if icon.is_some() {
                                        return Err(de::Error::duplicate_field("icon"));
                                    }
                                    icon = Some(map.next_value()?);
                                }
                                "image" => {
                                    if image.is_some() {
                                        return Err(de::Error::duplicate_field("image"));
                                    }
                                    image = Some(map.next_value()?);
                                }
                                "inReplyTo" => {
                                    if in_reply_to.is_some() {
                                        return Err(de::Error::duplicate_field("inReplyTo"));
                                    }
                                    in_reply_to = Some(map.next_value()?);
                                }
                                "location" => {
                                    if location.is_some() {
                                        return Err(de::Error::duplicate_field("location"));
                                    }
                                    location = Some(map.next_value()?);
                                }
                                "url" => {
                                    if url.is_some() {
                                        return Err(de::Error::duplicate_field("url"));
                                    }
                                    url = Some(map.next_value()?);
                                }
                                "preview" => {
                                    if preview.is_some() {
                                        return Err(de::Error::duplicate_field("preview"));
                                    }
                                    preview = Some(map.next_value()?);
                                }
                                "replies" => {
                                    if replies.is_some() {
                                        return Err(de::Error::duplicate_field("replies"));
                                    }
                                    replies = Some(map.next_value()?);
                                }
                                "tag" => {
                                    if tag.is_some() {
                                        return Err(de::Error::duplicate_field("tag"));
                                    }
                                    tag = Some(map.next_value()?);
                                }
                                "to" => {
                                    if to.is_some() {
                                        return Err(de::Error::duplicate_field("to"));
                                    }
                                    to = Some(map.next_value()?);
                                }
                                "bto" => {
                                    if bto.is_some() {
                                        return Err(de::Error::duplicate_field("bto"));
                                    }
                                    bto = Some(map.next_value()?);
                                }
                                "cc" => {
                                    if cc.is_some() {
                                        return Err(de::Error::duplicate_field("cc"));
                                    }
                                    cc = Some(map.next_value()?);
                                }
                                "bcc" => {
                                    if bcc.is_some() {
                                        return Err(de::Error::duplicate_field("bcc"));
                                    }
                                    bcc = Some(map.next_value()?);
                                }
                                "mediaType" => {
                                    if media_type.is_some() {
                                        return Err(de::Error::duplicate_field("mediaType"));
                                    }
                                    media_type = Some(map.next_value()?);
                                }
                                "startTime" => {
                                    if start_time.is_some() {
                                        return Err(de::Error::duplicate_field("startTime"));
                                    }
                                    start_time = Some(map.next_value()?);
                                }
                                "endTime" => {
                                    if end_time.is_some() {
                                        return Err(de::Error::duplicate_field("endTime"));
                                    }
                                    end_time = Some(map.next_value()?);
                                }
                                "published" => {
                                    if published.is_some() {
                                        return Err(de::Error::duplicate_field("published"));
                                    }
                                    published = Some(map.next_value()?);
                                }
                                "updated" => {
                                    if updated.is_some() {
                                        return Err(de::Error::duplicate_field("updated"));
                                    }
                                    updated = Some(map.next_value()?);
                                }
                                "duration" => {
                                    if duration.is_some() {
                                        return Err(de::Error::duplicate_field("duration"));
                                    }
                                    duration = Some(map.next_value()?);
                                }
                                "href" | "hreflang" | "rel" | "height" | "width" => return Err(de::Error::unknown_field(key.as_str(), &[])),
                                $(
                                s if s == stringify!($field).to_lower_camel_case().as_str() => {
                                    if $field.is_some() {
                                        return Err(de::Error::duplicate_field(stringify!($field)));
                                    }
                                    $field = Some(map.next_value()?);
                                }
                                )*
                                ex_key => {
                                    if let Some(extra) = extra_fields.as_mut() {
                                        if extra.contains_key(ex_key) {
                                            return Err(de::Error::duplicate_field(stringify!(key)));
                                        }
                                        extra.insert(ex_key.to_string(), map.next_value()?);
                                    } else {
                                        let mut extra = $crate::Map::new();
                                        extra.insert(ex_key.to_string(), map.next_value()?);
                                        extra_fields = Some(Box::new(extra));
                                    }
                                }
                            }
                        }

                        let kind: VT = kind.ok_or(de::Error::missing_field("type"))?;

                        $(
                            // FIXME(HACK): declarative-macro hack to optionally require the
                            // vocabulary type to strictly match the object type:
                            // - e.g. note.kind() == "Note"
                            //
                            // The permissive deserialization allows parsing
                            // `Object`-derived types as an `Object`.
                            //
                            // A proper fix requires rewriting as a proc-macro.
                            let _ = stringify!($vocab_serde);
                            if !kind.contains(stringify!($ty)) {
                                return Err($crate::serde::de::Error::unknown_variant(stringify!($ty), &[]));
                            }
                        )?

                        Ok(Self::Value {
                            context_property,
                            kind,
                            id,
                            name,
                            name_map,
                            attributed_to,
                            audience,
                            content,
                            content_map,
                            summary,
                            summary_map,
                            context,
                            generator,
                            icon,
                            image,
                            in_reply_to,
                            location,
                            url,
                            preview,
                            replies,
                            tag,
                            to,
                            bto,
                            cc,
                            bcc,
                            media_type,
                            start_time,
                            end_time,
                            published,
                            updated,
                            duration,
                            extra_fields,
                            $(
                            $field,
                            )*
                        })
                    }
                }

                deserializer.deserialize_map(Visitor(PhantomData))
            }
        }
    };
}

/// Helper macro to implement field access for [Object](crate::Object)-derived types.
///
/// # Parameters
///
/// - `ty`: represents the type to implement the field access functions.
#[macro_export]
macro_rules! object_field_access {
    ($ty:ident) => {
        $crate::field_access! {
            $ty<Vocab> {
                /// Provides the ActivityStream Vocabulary `type`.
                kind: as_ref { Vocab },
            }
        }

        $crate::field_access! {
            $ty {
                /// Provides the globally unique identifier for an [Object](crate::Object).
                id: option_ref { $crate::Iri },
                /// Represents the special `@context` property to define the processing context.
                ///
                /// The value of the `@context` property is defined by the [JSON-LD](https://www.w3.org/TR/json-ld/#the-context) specification.
                context_property: option_ref { $crate::Context },
                /// A simple, human-readable, plain-text name for the object.
                ///
                /// HTML markup **MUST NOT** be included.
                ///
                /// The name **MAY** be expressed using multiple language-tagged values.
                name: option_ref { $crate::Name },
                /// A simple, human-readable, plain-text name for the object expressed using multiple language-tagged values.
                ///
                /// HTML markup **MUST NOT** be included.
                name_map: option_ref { $crate::NameMap },
                /// The content or textual representation of the Object encoded as a JSON string, expressed using multiple language-tagged values.
                ///
                /// By default, the value of content is HTML.
                ///
                /// The [`mediaType`](Self::media_type) property can be used in the object to indicate a different content type.
                content_map: option_ref { $crate::LanguageMap },
                /// A natural language summarization of the object encoded as HTML, expressed as multiple language-tagged summaries.
                summary_map: option_ref { $crate::LanguageMap },
                /// When the object describes a time-bound resource, such as an audio or video, a meeting, etc, the duration property indicates the object's approximate duration.
                ///
                /// The value **MUST** be expressed as an `xsd:duration` as defined by [xmlschema11-2](https://www.w3.org/TR/xmlschema11-2/#duration).
                duration: option_ref { $crate::Duration },
            }
        }

        $crate::field_access! {
            $ty {
                /// The content or textual representation of the Object encoded as a JSON string.
                ///
                /// By default, the value of content is HTML.
                ///
                /// The [`mediaType`](Self::media_type) property can be used in the object to indicate a different content type.
                ///
                /// The content **MAY** be expressed using multiple language-tagged values.
                content: option_deref { &str, String },
                /// A natural language summarization of the object encoded as HTML.
                ///
                /// Multiple language tagged summaries MAY be provided.
                summary: option_deref { &str, String },
            }
        }

        $crate::field_access! {
            $ty {
                /// Identifies one or more entities to which this object is attributed.
                ///
                /// The attributed entities might not be Actors.
                ///
                /// For instance, an object might be attributed to the completion of another activity.
                attributed_to: option_box_deref { $crate::Item },
                /// Identifies one or more entities that represent the total population of entities for which the object can be considered to be relevant.
                audience: option_box_deref { $crate::Item },
                /// Identifies the context within which the object exists or an activity was performed.
                ///
                /// The notion of "context" used is intentionally vague.
                /// The intended function is to serve as a means of grouping objects and activities that share a common originating context or purpose.
                /// An example could be all activities relating to a common project or event.
                context: option_box_deref { $crate::Item },
                /// Identifies the entity (e.g. an application) that generated the object.
                generator: option_box_deref { $crate::Item },
                /// Indicates an entity that describes an icon for this object.
                ///
                /// The image should have an aspect ratio of one (horizontal) to one (vertical) and should be suitable for presentation at a small size.
                icon: option_box_deref { $crate::ImageItem },
                /// Indicates an entity that describes an image for this object.
                ///
                /// Unlike the icon property, there are no aspect ratio or display size limitations assumed.
                image: option_box_deref { $crate::ImageItem },
                /// Indicates one or more entities for which this object is considered a response.
                in_reply_to: option_box_deref { $crate::Item },
                /// Indicates one or more physical or logical locations associated with the object.
                location: option_box_deref { $crate::Item },
                /// Identifies one or more links to representations of the object
                url: option_box_deref { $crate::IriItem },
                /// Identifies an entity that provides a preview of this object.
                preview: option_box_deref { $crate::Item },
                /// Identifies a [Collection](crate::Collection) containing objects considered to be responses to this object.
                replies: option_box_deref { $crate::Collection },
                /// One or more "tags" that have been associated with an objects.
                ///
                /// A tag can be any kind of [Object](crate::Object).
                ///
                /// The key difference between `attachment` and `tag` is that the former implies association by inclusion,
                /// while the latter implies associated by reference.
                tag: option_box_deref { $crate::Item },
                /// Identifies one or more entities that are part of the public primary audience of this [Object](crate::Object).
                to: option_box_deref { $crate::Item },
                /// Identifies one or more entities that are part of the private primary audience of this [Object](crate::Object).
                bto: option_box_deref { $crate::Item },
                /// Identifies one or more entities that are part of the public secondary audience of this [Object](crate::Object).
                cc: option_box_deref { $crate::Item },
                /// Identifies one or more entities that are part of the private secondary audience of this [Object](crate::Object).
                bcc: option_box_deref { $crate::Item },
                extra_fields: option_box_deref { $crate::Map<String, $crate::serde_json::Value> },
            }
        }

        $crate::field_access! {
            $ty {
                /// When used on a [Link](crate::Link), identifies the MIME media type of the referenced resource.
                ///
                /// When used on an Object, identifies the MIME media type of the value of the [content](Self::content) property.
                ///
                /// If not specified, the content property is assumed to contain `text/html` content.
                media_type: option { $crate::MimeType },
                /// The date and time describing the actual or expected starting time of the object.
                ///
                /// When used with an `Activity` object, for instance, the `startTime` property specifies the moment the activity began or is scheduled to begin.
                start_time: option { $crate::DateTime },
                /// The date and time describing the actual or expected ending time of the object.
                ///
                /// When used with an `Activity` object, for instance, the `endTime` property specifies the moment the activity concluded or is expected to conclude.
                end_time: option { $crate::DateTime },
                /// The date and time at which the object was published.
                published: option { $crate::DateTime },
                /// The date and time at which the object was updated.
                updated: option { $crate::DateTime },
            }
        }
    };
}

/// Helper macro to implement custom serde (de)serializer function for an [Object](crate::Object)-derived type.
///
/// Checks that the derived object contains the correct vocabulary type in its `type` field.
#[macro_export]
macro_rules! derived_kind_serde {
    ($vocab_ty:ty, $kind:ty) => {
        $crate::paste! {
            mod obj_serde {
                pub(crate) fn ser<S>(kind: &$crate::VocabularyTypes, s: S) -> ::core::result::Result<S::Ok, S::Error>
                where
                    S: $crate::serde::ser::Serializer,
                {
                    use $crate::serde::ser::Serialize;

                    if kind.contains($crate::$vocab_ty::$kind) {
                        kind.serialize(s)
                    } else {
                        Err($crate::serde::ser::Error::custom(format!(
                            "invalid vocabulary type: {kind}",
                        )))
                    }
                }

                pub(crate) fn de<'de, D>(d: D) -> ::core::result::Result<$crate::VocabularyTypes, D::Error>
                where
                    D: $crate::serde::de::Deserializer<'de>,
                {
                    use $crate::serde::de::Deserialize;

                    $crate::VocabularyTypes::deserialize(d).and_then(|k| {
                        if k.contains($crate::$vocab_ty::$kind) {
                            Ok(k)
                        } else {
                            Err($crate::serde::de::Error::custom(format!(
                                "invalid vocabulary type: {k}",
                            )))
                        }
                    })
                }
            }
        }
    };

    ($vocab_ty:ty) => {
        $crate::paste! {
            pub(crate) mod obj_serde {
                pub(crate) fn ser<S, V: $crate::ActivityVocabulary>(kind: &V, s: S) -> ::core::result::Result<S::Ok, S::Error>
                where
                    S: $crate::serde::ser::Serializer,
                {
                    if kind.contains($vocab_ty.as_str()) {
                        kind.serialize(s)
                    } else {
                        Err($crate::serde::ser::Error::custom(format!(
                            "invalid vocabulary type: {}", kind.kind(),
                        )))
                    }
                }
            }
        }
    };

    ($($vocab_path:ident ::)* $vocab_ty:ident :: $vocab_var:ident ) => {
        $crate::paste! {
            pub(crate) mod obj_serde {
                pub(crate) fn ser<S, V: $crate::ActivityVocabulary>(kind: &V, s: S) -> ::core::result::Result<S::Ok, S::Error>
                where
                    S: $crate::serde::ser::Serializer,
                {
                    if kind.contains($($vocab_path::)*$vocab_ty::$vocab_var.as_str()) {
                        kind.serialize(s)
                    } else {
                        Err($crate::serde::ser::Error::custom(format!(
                            "invalid vocabulary type: {}", kind.kind(),
                        )))
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_into_object {
    ($ty:ident $({
        $($field:ident $(,)?)*
    })?) => {
        $crate::impl_into_object! {
            $ty<$crate::VocabularyTypes> $({
                $($field ,)*
            })?
        }

        $crate::impl_into_item!($ty, object);
        $crate::impl_into_items!($ty);
        $crate::impl_into_ordered_items!($ty);
    };

    ($ty:ident<$vocab_ty:ty> $({
        $($field:ident $(,)?)*
    })?) => {
        impl From<$ty<$vocab_ty>> for $crate::Object<$vocab_ty> {
            #[allow(unused_mut)]
            fn from(val: $ty<$vocab_ty>) -> Self {
                let mut extra_fields = val.extra_fields;
                $(
                use $crate::heck::ToLowerCamelCase;
                use $crate::serde_json::json;
                $(
                let field = stringify!($field).to_lower_camel_case();
                if let Some(extra) = extra_fields.as_mut() && let Some(field_val) = val.$field.as_ref()  {
                    extra.insert(field, json!(field_val));
                } else if let Some(field_val) = val.$field.as_ref() {
                    let mut extra = $crate::Map::new();
                    extra.insert(field, json!(field_val));
                    extra_fields = Some(Box::new(extra));
                }
                )*
                )?

                let mut obj = Self::new().with_kind(val.kind);

                if let Some(ctx) = val.context_property {
                    obj.set_context_property(ctx);
                } else {
                    obj.unset_context_property();
                }

                if let Some(id) = val.id {
                    obj.set_id(id);
                }

                if let Some(name) = val.name {
                    obj.set_name(name);
                }

                if let Some(name_map) = val.name_map {
                    obj.set_name_map(name_map);
                }

                if let Some(attr) = val.attributed_to {
                    obj.set_attributed_to(*attr);
                }

                if let Some(audience) = val.audience {
                    obj.set_audience(*audience);
                }

                if let Some(content) = val.content {
                    obj.set_content(content);
                }

                if let Some(content_map) = val.content_map {
                    obj.set_content_map(content_map);
                }

                if let Some(summary) = val.summary {
                    obj.set_summary(summary);
                }

                if let Some(summary_map) = val.summary_map {
                    obj.set_summary_map(summary_map);
                }

                if let Some(context) = val.context {
                    obj.set_context(*context);
                }

                if let Some(generator) = val.generator {
                    obj.set_generator(*generator);
                }

                if let Some(icon) = val.icon {
                    obj.set_icon(*icon);
                }

                if let Some(image) = val.image {
                    obj.set_image(*image);
                }

                if let Some(in_reply_to) = val.in_reply_to {
                    obj.set_in_reply_to(*in_reply_to);
                }

                if let Some(location) = val.location {
                    obj.set_location(*location);
                }

                if let Some(url) = val.url {
                    obj.set_url(*url);
                }

                if let Some(preview) = val.preview {
                    obj.set_preview(*preview);
                }

                if let Some(replies) = val.replies {
                    obj.set_replies(*replies);
                }

                if let Some(tag) = val.tag {
                    obj.set_tag(*tag);
                }

                if let Some(to) = val.to {
                    obj.set_to(*to);
                }

                if let Some(bto) = val.bto {
                    obj.set_bto(*bto);
                }

                if let Some(cc) = val.cc {
                    obj.set_cc(*cc);
                }

                if let Some(bcc) = val.bcc {
                    obj.set_bcc(*bcc);
                }

                if let Some(media_type) = val.media_type {
                    obj.set_media_type(media_type);
                }

                if let Some(start_time) = val.start_time {
                    obj.set_start_time(start_time);
                }

                if let Some(end_time) = val.end_time {
                    obj.set_end_time(end_time);
                }

                if let Some(published) = val.published {
                    obj.set_published(published);
                }

                if let Some(updated) = val.updated {
                    obj.set_updated(updated);
                }

                if let Some(duration) = val.duration {
                    obj.set_duration(duration);
                }

                if let Some(extra_fields) = extra_fields {
                    obj.set_extra_fields(*extra_fields);
                }

                obj
            }
        }
    };
}

/// Helper macro to convert [Object](crate::Object)-like types into an [Item](crate::Item) variant.
#[macro_export]
macro_rules! impl_into_item {
    ($ty:ident, object) => {
        impl From<$ty> for $crate::Item {
            fn from(val: $ty) -> Self {
                Self::Object(Box::new(val.into()))
            }
        }

        impl From<$ty> for Option<Box<$crate::Item>> {
            fn from(val: $ty) -> Self {
                Some(Box::new(val.into()))
            }
        }
    };

    ($ty:ident, link) => {
        impl From<$ty> for $crate::Item {
            fn from(val: $ty) -> Self {
                Self::Link(Box::new(val.into()))
            }
        }

        impl From<$ty> for Option<Box<$crate::Item>> {
            fn from(val: $ty) -> Self {
                Some(Box::new(val.into()))
            }
        }
    };

    ($ty:ident, iri) => {
        impl From<$ty> for $crate::Item {
            fn from(val: $ty) -> Self {
                Self::Iri(Box::new(val.into()))
            }
        }

        impl From<$ty> for Option<Box<$crate::Item>> {
            fn from(val: $ty) -> Self {
                Some(Box::new(val.into()))
            }
        }
    };
}

/// Helper macro to convert [Object](crate::Object)-like types into an [Items](crate::Items) list.
#[macro_export]
macro_rules! impl_into_items {
    ($ty:ident) => {
        impl From<$ty> for Option<Box<$crate::Items>> {
            fn from(val: $ty) -> Self {
                Some(Box::new(val.into()))
            }
        }
    };
}

/// Helper macro to convert [Object](crate::Object)-like types into an [OrderedItems](crate::OrderedItems) list.
#[macro_export]
macro_rules! impl_into_ordered_items {
    ($ty:ident) => {
        impl From<$ty> for Option<Box<$crate::OrderedItems>> {
            fn from(val: $ty) -> Self {
                Some(Box::new(val.into()))
            }
        }
    };
}
