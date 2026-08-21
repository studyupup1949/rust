/// Helper macro to create a new item type that can be a object, link, or IRI.
#[macro_export]
macro_rules! create_item {
    (
        $(#[$ty_meta:meta])*
        $ty:ident base $(default: $default:expr,)? {
        $($variant:ident ( $variant_ty:ident ) $(,)?)+
    }) => {
        $(#[$ty_meta])*
        #[derive(Clone, Debug, Eq, PartialEq, $crate::serde::Deserialize, $crate::serde::Serialize)]
        #[serde(untagged)]
        pub enum $ty {
            $(
                $variant($variant_ty),
            )+
        }

        impl $ty {
            $crate::paste! {
                $(
                    #[doc = "Creates a new [" $ty "]."]
                    pub fn new() -> Self {
                        $default
                    }
                )?

                $(
                    #[doc = "Creates a new [" $variant "](Self::" $variant ") variant."]
                    #[inline]
                    pub fn [<$variant:lower>]<I: Into<$variant_ty>>(val: I) -> Self {
                        Self::$variant(val.into())
                    }

                    #[doc = "Gets whether the [" $ty "] is an [" $variant "](Self::" $variant ") variant."]
                    #[inline]
                    pub const fn [<is_ $variant:lower>](&self) -> bool {
                        matches!(self, Self::$variant(_))
                    }

                    #[doc = "Attempts get a reference to a [" $variant "](Self::" $variant ") variant."]
                    pub fn [<as_ $variant:lower>](&self) -> $crate::Result<&$variant_ty> {
                        match self {
                            Self::$variant(ty) => Ok(ty),
                            _ => Err($crate::Error::item(format!("invalid {} variant", stringify!($ty)))),
                        }
                    }

                    #[doc = "Attempts to convert into a [" $variant "](Self::" $variant ") variant."]
                    pub fn [<into_ $variant:lower>](self) -> $crate::Result<$variant_ty> {
                        match self {
                            Self::$variant(ty) => Ok(ty),
                            _ => Err($crate::Error::item(format!("invalid {} variant", stringify!($ty)))),
                        }
                    }
                )+
            }
        }

        $crate::impl_default!($ty);
        $crate::impl_display!($ty, json);
    };

    (
        $(#[$ty_meta:meta])*
        $ty:ident base boxed $(default: $default:expr,)? {
        $($variant:ident ( $variant_ty:ident ) $(,)?)+
    }) => {
        $(#[$ty_meta])*
        #[derive(Clone, Debug, Eq, PartialEq, $crate::serde::Deserialize, $crate::serde::Serialize)]
        #[serde(untagged)]
        pub enum $ty {
            $(
                $variant(Box<$variant_ty>),
            )+
        }

        impl $ty {
            $crate::paste! {
                $(
                    #[doc = "Creates a new [" $ty "]."]
                    pub fn new() -> Self {
                        $default
                    }
                )?

                $(
                    #[doc = "Creates a new [" $variant "](Self::" $variant ") variant."]
                    #[inline]
                    pub fn [<$variant:lower>]<I: Into<$variant_ty>>(val: I) -> Self {
                        Self::$variant(Box::new(val.into()))
                    }

                    #[doc = "Gets whether the [" $ty "] is an [" $variant "](Self::" $variant ") variant."]
                    #[inline]
                    pub const fn [<is_ $variant:lower>](&self) -> bool {
                        matches!(self, Self::$variant(_))
                    }

                    #[doc = "Attempts get a reference to a [" $variant "](Self::" $variant ") variant."]
                    pub fn [<as_ $variant:lower>](&self) -> $crate::Result<&$variant_ty> {
                        match self {
                            Self::$variant(ty) => Ok(ty.as_ref()),
                            _ => Err($crate::Error::item(format!("invalid {} variant", stringify!($ty)))),
                        }
                    }

                    #[doc = "Attempts to convert into a [" $variant "](Self::" $variant ") variant."]
                    pub fn [<into_ $variant:lower>](self) -> $crate::Result<$variant_ty> {
                        match self {
                            Self::$variant(ty) => Ok(*ty),
                            _ => Err($crate::Error::item(format!("invalid {} variant", stringify!($ty)))),
                        }
                    }
                )+
            }
        }

        $crate::impl_default!($ty);
        $crate::impl_display!($ty, json);
    };

    (
        $(#[$ty_meta:meta])*
        $ty:ident $(default: $default:expr,)? {
        $($variant:ident ( $variant_ty:ident ) $(,)?)+
    }) => {
        $crate::create_item! {
            $(#[$ty_meta])*
            $ty base $(default: $default,)? {
                $($variant ( $variant_ty ),)+
            }
        }

        $crate::create_item! {
            $ty: impl From {
                $($variant ( $variant_ty ),)+
            }
        }
    };

    (
        $(#[$ty_meta:meta])*
        $ty:ident boxed $(default: $default:expr,)? {
        $($variant:ident ( $variant_ty:ident ) $(,)?)+
    }) => {
        $crate::create_item! {
            $(#[$ty_meta])*
            $ty base boxed $(default: $default,)? {
                $($variant ( $variant_ty ),)+
            }
        }

        $crate::create_item! {
            $ty: impl From {
                $($variant ( $variant_ty ),)+
            }
        }
    };

    (
        $ty:ident: impl From {
        $($variant:ident ( $variant_ty:ident ) $(,)?)+
    }) => {
        $crate::paste! {
            $(
                impl From<$variant_ty> for $ty {
                    fn from(val: $variant_ty) -> Self {
                        Self::[<$variant:lower>](val)
                    }
                }

                impl<'a> TryFrom<&'a $ty> for &'a $variant_ty {
                    type Error = $crate::Error;

                    fn try_from(val: &'a $ty) -> $crate::Result<Self> {
                        val.[<as_ $variant:lower>]()
                    }
                }

                impl TryFrom<$ty> for $variant_ty {
                    type Error = $crate::Error;

                    fn try_from(val: $ty) -> $crate::Result<Self> {
                        val.[<into_ $variant:lower>]()
                    }
                }
            )+
        }
    };
}
