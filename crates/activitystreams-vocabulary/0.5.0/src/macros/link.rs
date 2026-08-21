/// Helper macro to define ActivityStream Link & Link-derived types.
#[macro_export]
macro_rules! create_link {
    (
        $(#[$doc:meta])*
        $ty:ident: $vocab_ty:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::create_link! {
            $(#[$doc])*
            $ty: $crate::$vocab_ty::$ty {
            $(
                $(#[$field_serde])*
                $field: $field_ty,
            )*
        }}
    };

    (
        $(#[$doc:meta])*
        $ty:ident:
        $vocab_path:ident :: $vocab_ty:ident :: $vocab_var:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::create_link! {
            $(#[$doc])*
            base $ty:
            #[serde(serialize_with = "obj_serde::ser")]
            $vocab_path::$vocab_ty::$vocab_var {
            $(
                $(#[$field_serde])*
                $field: $field_ty,
            )*
        }}

        $crate::impl_into_link!($ty {
            $(
                $(#[$field_serde])?
                $field,
            )*
        });
    };

    (
        $(#[$doc:meta])*
        base $ty:ident:
        $(#[$vocab_serde:meta])?
        $vocab_path:ident :: $vocab_ty:ident :: $vocab_var:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident: $field_ty:ty $(,)?
        )*
    }) => {
        $crate::paste! {
            $(#[$doc])*
            #[derive(Clone, Debug, Eq, PartialEq, $crate::serde::Serialize)]
            #[serde(rename_all = "camelCase")]
            pub struct $ty<Vocab: $crate::ActivityVocabulary = $crate::VocabularyTypes> {
                #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
                context_property: Option<$crate::Context>,
                #[serde(rename = "type")]
                $(#[$vocab_serde])?
                kind: Vocab,
                href: $crate::Iri,
                #[serde(skip_serializing_if = "Option::is_none")]
                name: Option<$crate::Name>,
                #[serde(skip_serializing_if = "Option::is_none")]
                name_map: Option<$crate::NameMap>,
                #[serde(skip_serializing_if = "Option::is_none")]
                rel: Option<$crate::Iri>,
                #[serde(skip_serializing_if = "Option::is_none")]
                hreflang: Option<$crate::LanguageTag>,
                #[serde(skip_serializing_if = "Option::is_none")]
                media_type: Option<$crate::MimeType>,
                #[serde(skip_serializing_if = "Option::is_none")]
                preview: Option<Box<$crate::Item>>,
                #[serde(skip_serializing_if = "Option::is_none")]
                height: Option<u64>,
                #[serde(skip_serializing_if = "Option::is_none")]
                width: Option<u64>,
                #[serde(flatten, skip_serializing_if = "Option::is_none")]
                extra_fields: Option<Box<$crate::Map<String, $crate::serde_json::Value>>>,
                $(
                    $(#[$field_serde])*
                    $field: $field_ty,
                )*
            }

            impl<Vocab: $crate::ActivityVocabulary + From<$vocab_path::$vocab_ty>> $ty<Vocab> {
                #[doc = "Creates a new [" $ty "]."]
                #[inline]
                pub fn new() -> Self {
                    Self::new_kind(Vocab::from($vocab_path::$vocab_ty::$vocab_var))
                }

                #[doc = "Creates a new [" $ty "] for use as an inner member of another object."]
                #[doc = ""]
                #[doc = "Encodes the type without the `@context` field."]
                #[inline]
                pub fn new_inner() -> Self {
                    Self::new_kind(Vocab::from($vocab_path::$vocab_ty::$vocab_var)).without_context_property()
                }

                #[doc = "Creates a new [" $ty "]."]
                #[inline]
                pub fn new_kind(kind: Vocab) -> Self {
                    Self {
                        context_property: Some($crate::Context::new()),
                        kind,
                        href: $crate::Iri::new(),
                        name: None,
                        name_map: None,
                        rel: None,
                        media_type: None,
                        hreflang: None,
                        preview: None,
                        height: None,
                        width: None,
                        extra_fields: None,
                        $(
                            $(#[$field_serde])*
                            $field: None,
                        )*
                    }
                }

                /// Builder function that unsets the context.
                pub fn without_context_property(self) -> Self {
                    Self {
                        context_property: None,
                        ..self
                    }
                }
            }

            $crate::field_access! {
                $ty<Vocab> {
                    /// Provides the ActivityStream Vocabulary `type`.
                    kind: as_ref { Vocab },
                }
            }

            $crate::field_access! {
                $ty<Vocab> {
                    /// Provides the globally unique identifier for an [Link](crate::Link).
                    href: as_ref { $crate::Iri },
                }
            }

            $crate::field_access! {
                $ty<Vocab> {
                    /// Represents the special `@context` property to define the processing context.
                    ///
                    /// The value of the `@context` property is defined by the [JSON-LD](https://www.w3.org/TR/json-ld/#the-context) specification.
                    context_property: option_ref { $crate::Context },
                    /// A simple, human-readable, plain-text name for the link.
                    ///
                    /// HTML markup **MUST NOT** be included.
                    ///
                    /// The name **MAY** be expressed using multiple language-tagged values.
                    name: option_ref { $crate::Name },
                    /// A simple, human-readable, plain-text name for the link, expressed using multiple language-tagged values.
                    ///
                    /// HTML markup **MUST NOT** be included.
                    name_map: option_ref { $crate::NameMap },
                    /// A link relation associated with a Link.
                    ///
                    /// The value **MUST** conform to both the `HTML5` and `RFC5988` "link relation" definitions.
                    rel: option_ref { $crate::Iri },
                    /// Hints as to the language used by the target resource.
                    hreflang: option_ref { $crate::LanguageTag },
                }
            }

            $crate::field_access! {
                $ty<Vocab> {
                    /// Identifies the MIME media type of the referenced resource.
                    media_type: option { $crate::MimeType },
                    /// Specifies a hint as to the rendering height in device-independent pixels of the linked resource.
                    height: option { u64 },
                    /// Specifies a hint as to the rendering width in device-independent pixels of the linked resource.
                    width: option { u64 },
                }
            }

            $crate::field_access! {
                $ty<Vocab> {
                    /// Identifies an entity that provides a preview of this object.
                    preview: option_box_deref { $crate::Item },
                    /// Extra fields defined by a derived `Link` type, or other extension.
                    extra_fields: option_box_deref { $crate::Map<String, $crate::serde_json::Value> },
                }
            }

            $crate::derived_kind_serde!($vocab_path::$vocab_ty::$vocab_var);

            $crate::impl_default!($ty);
            $crate::impl_display!($ty, json);

            $crate::impl_into_item!($ty, link);
            $crate::impl_into_items!($ty);
            $crate::impl_into_ordered_items!($ty);

            $crate::derive_link!($ty {
                $(
                    $(#[$field_serde])*
                    $field,
                )*
            });
        }
    };
}

/// Helper macro to implement the [`Deserialize`](serde::de::Deserialize) trait for `Link` types.
#[macro_export]
macro_rules! derive_link {
    ($ty:ident {
        $(
            $(#[$field_serde:meta])*
            $field:ident $(,)?
        )*
    }) => {
        impl<'de, T: $crate::ActivityVocabulary> $crate::serde::de::Deserialize<'de> for $ty<T> {
            fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
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

                    #[allow(unused)]
                    fn visit_map<V>(self, mut map: V) -> core::result::Result<Self::Value, V::Error>
                        where V: de::MapAccess<'vde>,
                    {
                        $crate::paste! {
                            let mut context_property = None;
                            let mut kind = None;
                            let mut href = None;
                            let mut name = None;
                            let mut name_map = None;
                            let mut rel = None;
                            let mut hreflang = None;
                            let mut preview = None;
                            let mut media_type = None;
                            let mut height = None;
                            let mut width = None;
                            let mut extra_fields: Option<Box<$crate::Map<String, $crate::serde_json::Value>>> = None;
                            $(
                            let [<$field _name>] = $crate::field_rename! {
                                $(#[$field_serde])*
                                $field
                            };
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

                                        if kind.is_none() {
                                            kind = Some(map.next_value()?);
                                        }
                                    }
                                    "href" => {
                                        if href.is_some() {
                                            return Err(de::Error::duplicate_field("href"));
                                        }
                                        href = Some(map.next_value()?);
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
                                    "rel" => {
                                        if rel.is_some() {
                                            return Err(de::Error::duplicate_field("rel"));
                                        }
                                        rel = Some(map.next_value()?);
                                    }
                                    "hreflang" => {
                                        if hreflang.is_some() {
                                            return Err(de::Error::duplicate_field("hreflang"));
                                        }
                                        hreflang = Some(map.next_value()?);
                                    }
                                    "preview" => {
                                        if preview.is_some() {
                                            return Err(de::Error::duplicate_field("preview"));
                                        }
                                        preview = Some(map.next_value()?);
                                    }
                                    "mediaType" => {
                                        if media_type.is_some() {
                                            return Err(de::Error::duplicate_field("mediaType"));
                                        }
                                        media_type = Some(map.next_value()?);
                                    }
                                    "height" => {
                                        if height.is_some() {
                                            return Err(de::Error::duplicate_field("height"));
                                        }
                                        height = Some(map.next_value()?);
                                    }
                                    "width" => {
                                        if width.is_some() {
                                            return Err(de::Error::duplicate_field("width"));
                                        }
                                        width = Some(map.next_value()?);
                                    }
                                    "id" | "attributedTo" | "audience"  | "content" | "contentMap" | "summary" | "summaryMap" | "context" | "generator" | "icon" | "image" | "inReplyTo" | "location" | "url" | "replies" | "tag" | "to" | "bto" | "cc" | "bcc" | "startTime" | "endTime" | "published" | "updated" | "duration" => return Err(de::Error::unknown_field(key.as_str(), &[])),
                                    $(
                                    [<$field _name>] => {
                                        if $field.is_some() {
                                            return Err(de::Error::duplicate_field(stringify!([<$field _name>])));
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
                                    _ => (),
                                }
                            }

                            let kind: VT = kind.ok_or(de::Error::missing_field("type"))?;
                            let href = href.ok_or(de::Error::missing_field("href"))?;

                            if !kind.contains(stringify!($ty)) {
                                return Err($crate::serde::de::Error::custom(stringify!($ty)));
                            }

                            Ok(Self::Value {
                                context_property,
                                kind,
                                href,
                                name,
                                name_map,
                                rel,
                                media_type,
                                hreflang,
                                preview,
                                height,
                                width,
                                extra_fields,
                                $(
                                $field,
                                )*
                            })
                        }
                    }
                }

                deserializer.deserialize_map(Visitor(PhantomData))
            }
        }
    };
}

/// Helper macro to convert a `Link`-derived type into a base `Link` type.
#[macro_export]
macro_rules! impl_into_link {
    ($ty:ident $({
        $(
            $(#[$field_serde:meta])?
            $field:ident $(,)?
        )*
    })?) => {
        $crate::impl_into_link! {
            $ty<$crate::VocabularyTypes> $({
                $(
                $(#[$field_serde])*
                $field,
                )*
            })?
        }
    };

    ($ty:ident<$vocab_ty:ty> $({
        $(
            $(#[$field_serde:meta])?
            $field:ident $(,)?
        )*
    })?) => {
        impl From<$ty<$vocab_ty>> for $crate::Link<$vocab_ty> {
            #[allow(unused, unused_mut)]
            fn from(val: $ty<$vocab_ty>) -> Self {
                let mut extra_fields = val.extra_fields;
                $(
                use $crate::heck::ToLowerCamelCase;
                use $crate::serde_json::json;
                $(
                let [<$field _name>] = $crate::field_rename! {
                    $(#[$field_serde])*
                    $field
                };
                if let Some(extra) = extra_fields.as_mut() && let Some(field_val) = val.$field.as_ref()  {
                    extra.insert([<$field _name>], json!(field_val));
                } else if let Some(field_val) = val.$field.as_ref() {
                    let mut extra = $crate::Map::new();
                    extra.insert([<$field _name>], json!(field_val));
                    extra_fields = Some(Box::new(extra));
                }
                )*
                )?

                let mut link = Self::new()
                    .with_kind(val.kind)
                    .with_href(val.href);

                if let Some(ctx) = val.context_property {
                    link.set_context_property(ctx);
                } else {
                    link.unset_context_property();
                }

                if let Some(name) = val.name {
                    link.set_name(name);
                }

                if let Some(name_map) = val.name_map {
                    link.set_name_map(name_map);
                }

                if let Some(rel) = val.rel {
                    link.set_rel(rel);
                }

                if let Some(hreflang) = val.hreflang {
                    link.set_hreflang(hreflang);
                }

                if let Some(media_type) = val.media_type {
                    link.set_media_type(media_type);
                }

                if let Some(preview) = val.preview {
                    link.set_preview(*preview);
                }

                if let Some(width) = val.width {
                    link.set_width(width);
                }

                if let Some(height) = val.height {
                    link.set_height(height);
                }

                if let Some(extra) = extra_fields {
                    link.set_extra_fields(*extra);
                }

                link
            }
        }
    };
}
