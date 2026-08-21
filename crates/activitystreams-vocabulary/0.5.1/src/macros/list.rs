/// Helper macro to create list types.
///
/// Useful for when a field can either be a single object, or list of objects.
///
/// # Example
///
/// ```rust
/// use activitystreams_vocabulary::create_list;
///
/// #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
/// #[serde(rename_all = "camelCase")]
/// pub struct SomeType {
///     some_field: bool,
/// }
///
/// impl SomeType {
///     pub const fn new() -> Self {
///         Self { some_field: false }
///     }
/// }
///
/// create_list!(SomeTypes: SomeType);
///
/// let field = SomeType { some_field: true };
/// let single = SomeTypes::single(field);
///
/// assert!(single.is_single());
/// assert_eq!(single.as_single(), Ok(&field));
/// assert!(single.as_list().is_err());
/// assert_eq!(single.clone().into_single(), Ok(field));
/// assert_eq!(single.clone().into_list(), vec![field]);
///
/// let json_str = r#"{
///   "someField": true
/// }"#;
///
/// assert_eq!(serde_json::to_string_pretty(&single).unwrap(), json_str);
/// assert_eq!(serde_json::from_str::<SomeTypes>(json_str).unwrap(), single);
///
/// let field1 = SomeType { some_field: false };
///
/// let list = SomeTypes::list([field, field1]);
///
/// assert!(list.is_list());
/// assert_eq!(list.as_single(), Ok(&field));
/// assert_eq!(list.as_list(), Ok([field, field1].as_ref()));
/// assert_eq!(list.clone().into_single(), Ok(field));
/// assert_eq!(list.clone().into_list(), vec![field, field1]);
///
/// let json_str = r#"[
///   {
///     "someField": true
///   },
///   {
///     "someField": false
///   }
/// ]"#;
///
/// assert_eq!(serde_json::to_string_pretty(&list).unwrap(), json_str);
/// assert_eq!(serde_json::from_str::<SomeTypes>(json_str).unwrap(), list);
/// ```
#[macro_export]
macro_rules! create_list {
    (
        $(#[$ty_meta:meta])*
        $ty:ident: $item:ident $(,)?
    ) => {
        $(#[$ty_meta])*
        #[derive(Clone, Debug, Eq, PartialEq, $crate::serde::Deserialize, $crate::serde::Serialize)]
        #[serde(untagged)]
        pub enum $ty {
            Single($item),
            List(Vec<$item>),
        }

        impl $ty {
            $crate::paste! {
                #[doc = "Creates a new [" $ty "]."]
                pub fn new() -> Self {
                    Self::Single($item::new())
                }

                #[doc = "Creates a new [" $ty "] list variant."]
                pub fn new_list() -> Self {
                    Self::List(Vec::new())
                }

                #[doc = "Creates a new [" $ty "] [Single](Self::Single) variant."]
                pub fn single<I: Into<$item>>(val: I) -> Self {
                    Self::Single(val.into())
                }

                #[doc = "Gets whether the [" $ty "] contains a [Single](Self::Single) variant."]
                pub const fn is_single(&self) -> bool {
                    matches!(self, Self::Single(_))
                }

                /// Attempts to get a reference to the [Single](Self::Single) variant.
                pub fn as_single(&self) -> $crate::Result<&$item> {
                    match self {
                        Self::Single(ty) => Ok(ty),
                        Self::List(tys) => tys
                            .first()
                            .ok_or($crate::Error::list(format!("empty {} list", stringify!($ty)))),
                    }
                }

                /// Attempts to convert to a [Single](Self::Single) variant.
                ///
                /// If it contains a [List](Self::List) variant, the first item is returned, and the list is consumed.
                ///
                /// If it contains an empty [List](Self::List) variant, an [Error](crate::Error) is returned.
                pub fn into_single(self) -> $crate::Result<$item> {
                    match self {
                        Self::Single(ty) => Ok(ty),
                        Self::List(tys) => tys
                            .into_iter()
                            .next()
                            .ok_or($crate::Error::list(format!("empty {} list", stringify!($ty)))),
                    }
                }

                #[doc = "Creates a new [" $ty "] [Single](Self::Single) variant."]
                pub fn list<T, I>(val: I) -> Self
                where
                    T: Into<$item>,
                    I: IntoIterator<Item = T>,
                {
                    Self::List(val.into_iter().map(|i| i.into()).collect())
                }

                #[doc = "Gets whether the [" $ty "] contains a [List](Self::List) variant."]
                pub const fn is_list(&self) -> bool {
                    matches!(self, Self::List(_))
                }

                /// Attempts to get a reference to the [List](Self::List) variant.
                pub fn as_list(&self) -> $crate::Result<&[$item]> {
                    match self {
                        Self::List(tys) => Ok(tys),
                        _ => Err($crate::Error::list("invalid keys type")),
                    }
                }

                /// Converts to a [List](Self::List) variant.
                ///
                /// If it contains a [Single](Self::Single) variant, a single-item list is returned.
                pub fn into_list(self) -> Vec<$item> {
                    match self {
                        Self::Single(ty) => vec![ty],
                        Self::List(tys) => tys,
                    }
                }
            }
        }

        impl<T: Into<$item>> From<T> for $ty {
            fn from(val: T) -> Self {
                Self::single(val)
            }
        }

        impl<T: Into<$item>> From<Vec<T>> for $ty {
            fn from(val: Vec<T>) -> Self {
                Self::list(val)
            }
        }

        impl<T: Into<$item> + Clone> From<&[T]> for $ty {
            fn from(val: &[T]) -> Self {
                Self::list(val.iter().cloned())
            }
        }

        impl<T: Into<$item> + Clone, const N: usize> From<&[T; N]> for $ty {
            fn from(val: &[T; N]) -> Self {
                Self::list(val.iter().cloned())
            }
        }

        impl<T: Into<$item>, const N: usize> From<[T; N]> for $ty {
            fn from(val: [T; N]) -> Self {
                Self::list(val)
            }
        }

        impl<'a> TryFrom<&'a $ty> for &'a $item {
            type Error = $crate::Error;

            fn try_from(val: &'a $ty) -> $crate::Result<Self> {
                val.as_single()
            }
        }

        impl TryFrom<$ty> for $item {
            type Error = $crate::Error;

            fn try_from(val: $ty) -> $crate::Result<Self> {
                val.into_single()
            }
        }

        impl<'a> TryFrom<&'a $ty> for &'a [$item] {
            type Error = $crate::Error;

            fn try_from(val: &'a $ty) -> $crate::Result<Self> {
                val.as_list()
            }
        }

        impl From<$ty> for Vec<$item> {
            fn from(val: $ty) -> Self {
                val.into_list()
            }
        }

        $crate::impl_default!($ty);
        $crate::impl_display!($ty, json);
    };

    (
        $(#[$ty_meta:meta])*
        $ty:ident: ordered { $item:ident } $(,)?
    ) => {
        $(#[$ty_meta])*
        #[derive(Clone, Debug, Eq, PartialEq, $crate::serde::Deserialize, $crate::serde::Serialize)]
        #[serde(untagged)]
        pub enum $ty {
            Single($item),
            List($crate::OrderedList<$item>),
        }

        impl $ty {
            $crate::paste! {
                #[doc = "Creates a new [" $ty "]."]
                pub fn new() -> Self {
                    Self::Single($item::new())
                }

                #[doc = "Creates a new [" $ty "] list variant."]
                pub fn new_list() -> Self {
                    Self::List($crate::OrderedList::new())
                }

                #[doc = "Creates a [" $ty "] [Single](Self::Single) variant."]
                pub fn single<I: Into<$item>>(val: I) -> Self {
                    Self::Single(val.into())
                }

                #[doc = "Gets whether the [" $ty "] contains a [Single](Self::Single) variant."]
                pub const fn is_single(&self) -> bool {
                    matches!(self, Self::Single(_))
                }

                /// Attempts to get a reference to the [Single](Self::Single) variant.
                pub fn as_single(&self) -> $crate::Result<&$item> {
                    match self {
                        Self::Single(ty) => Ok(ty),
                        Self::List(tys) => tys
                            .as_ref()
                            .first()
                            .ok_or($crate::Error::list(format!("empty {} list", stringify!($ty)))),
                    }
                }

                /// Attempts to convert into a [Single](Self::Single) variant.
                ///
                /// If it contains a [List](Self::List) variant, the first item is returned, and the list is consumed.
                ///
                /// If it contains an empty [List](Self::List) variant, an [Error](crate::Error) is returned.
                pub fn into_single(self) -> $crate::Result<$item> {
                    match self {
                        Self::Single(ty) => Ok(ty),
                        Self::List(tys) => tys
                            .into_iter()
                            .next()
                            .ok_or($crate::Error::list(format!("empty {} list", stringify!($ty)))),
                    }
                }

                #[doc = "Creates a [" $ty "] [List](Self::List) variant."]
                pub fn list<T, I>(val: I) -> Self
                where
                    T: Into<$item>,
                    I: IntoIterator<Item = T>,
                {
                    Self::List($crate::OrderedList::from_items(val))
                }

                #[doc = "Gets whether the [" $ty "] contains a [List](Self::List) variant."]
                pub const fn is_list(&self) -> bool {
                    matches!(self, Self::List(_))
                }

                /// Attempts to get a reference to the [List](Self::List) variant.
                pub fn as_list(&self) -> $crate::Result<&[$item]> {
                    match self {
                        Self::List(tys) => Ok(tys.as_ref()),
                        _ => Err($crate::Error::list(format!("invalid {} variant", stringify!($ty)))),
                    }
                }

                /// Converts into a [List](Self::List) variant.
                ///
                /// If it contains a [Single](Self::Single) variant, a single-item list is returned.
                pub fn into_list(self) -> $crate::OrderedList<$item> {
                    match self {
                        Self::Single(ty) => $crate::OrderedList::from_items([ty]),
                        Self::List(tys) => tys,
                    }
                }
            }
        }

        impl<I: Into<$item>> From<I> for $ty {
            fn from(val: I) -> Self {
                Self::single(val)
            }
        }

        impl<I: Into<$item>> From<Vec<I>> for $ty {
            fn from(val: Vec<I>) -> Self {
                Self::list(val)
            }
        }

        impl<I: Into<$item> + Clone> From<&[I]> for $ty {
            fn from(val: &[I]) -> Self {
                Self::list(val.iter().cloned())
            }
        }

        impl<I: Into<$item> + Clone, const N: usize> From<&[I; N]> for $ty {
            fn from(val: &[I; N]) -> Self {
                Self::list(val.iter().cloned())
            }
        }

        impl<I: Into<$item>, const N: usize> From<[I; N]> for $ty {
            fn from(val: [I; N]) -> Self {
                Self::list(val)
            }
        }

        impl<'a> TryFrom<&'a $ty> for &'a $item {
            type Error = $crate::Error;

            fn try_from(val: &'a $ty) -> $crate::Result<Self> {
                val.as_single()
            }
        }

        impl TryFrom<$ty> for $item {
            type Error = $crate::Error;

            fn try_from(val: $ty) -> $crate::Result<Self> {
                val.into_single()
            }
        }

        impl<'a> TryFrom<&'a $ty> for &'a [$item] {
            type Error = $crate::Error;

            fn try_from(val: &'a $ty) -> $crate::Result<Self> {
                val.as_list()
            }
        }

        impl From<$ty> for $crate::OrderedList<$item> {
            fn from(val: $ty) -> Self {
                val.into_list()
            }
        }

        $crate::impl_default!($ty);
        $crate::impl_display!($ty, json);
    };

    (
        $(#[$ty_meta:meta])*
        $ty:ident: boxed { $item:ident } $(,)?
    ) => {
        $(#[$ty_meta])*
        #[derive(Clone, Debug, Eq, PartialEq, $crate::serde::Deserialize, $crate::serde::Serialize)]
        #[serde(untagged)]
        pub enum $ty {
            Single(Box<$item>),
            List(Box<Vec<$item>>),
        }

        impl $ty {
            $crate::paste! {
                #[doc = "Creates a new [" $ty "]."]
                pub fn new() -> Self {
                    Self::Single(Box::default())
                }

                #[doc = "Creates a new [" $ty "] list variant."]
                pub fn new_list() -> Self {
                    Self::List(Box::default())
                }

                #[doc = "Creates a [" $ty "] [Single](Self::Single) variant."]
                pub fn single<I: Into<$item>>(val: I) -> Self {
                    Self::Single(Box::new(val.into()))
                }

                #[doc = "Gets whether the [" $ty "] contains a [Single](Self::Single) variant."]
                pub const fn is_single(&self) -> bool {
                    matches!(self, Self::Single(_))
                }

                /// Attempts to get a reference to the [Single](Self::Single) variant.
                pub fn as_single(&self) -> $crate::Result<&$item> {
                    match self {
                        Self::Single(ty) => Ok(ty),
                        Self::List(tys) => tys
                            .as_ref()
                            .first()
                            .ok_or($crate::Error::list(format!("empty {} list", stringify!($ty)))),
                    }
                }

                /// Attempts to convert into a [Single](Self::Single) variant.
                ///
                /// If it contains a [List](Self::List) variant, the first item is returned, and the list is consumed.
                ///
                /// If it contains an empty [List](Self::List) variant, an [Error](crate::Error) is returned.
                pub fn into_single(self) -> $crate::Result<$item> {
                    match self {
                        Self::Single(ty) => Ok(*ty),
                        Self::List(tys) => tys
                            .into_iter()
                            .next()
                            .ok_or($crate::Error::list(format!("empty {} list", stringify!($ty)))),
                    }
                }

                #[doc = "Creates a [" $ty "] [List](Self::List) variant."]
                pub fn list<T, I>(val: I) -> Self
                where
                    T: Into<$item>,
                    I: IntoIterator<Item = T>,
                {
                    Self::List(Box::new(val.into_iter().map(|i| i.into()).collect()))
                }

                #[doc = "Gets whether the [" $ty "] contains a [List](Self::List) variant."]
                pub const fn is_list(&self) -> bool {
                    matches!(self, Self::List(_))
                }

                /// Attempts to get a reference to the [List](Self::List) variant.
                pub fn as_list(&self) -> $crate::Result<&[$item]> {
                    match self {
                        Self::List(tys) => Ok(tys.as_ref()),
                        _ => Err($crate::Error::list(format!("invalid {} variant", stringify!($ty)))),
                    }
                }

                /// Converts into a [List](Self::List) variant.
                ///
                /// If it contains a [Single](Self::Single) variant, a single-item list is returned.
                pub fn into_list(self) -> Vec<$item> {
                    match self {
                        Self::Single(ty) => vec![*ty],
                        Self::List(tys) => *tys,
                    }
                }
            }
        }

        impl<I: Into<$item>> From<I> for $ty {
            fn from(val: I) -> Self {
                Self::single(val)
            }
        }

        impl<I: Into<$item>> From<Vec<I>> for $ty {
            fn from(val: Vec<I>) -> Self {
                Self::list(val)
            }
        }

        impl<I: Into<$item> + Clone> From<&[I]> for $ty {
            fn from(val: &[I]) -> Self {
                Self::list(val.iter().cloned())
            }
        }

        impl<I: Into<$item> + Clone, const N: usize> From<&[I; N]> for $ty {
            fn from(val: &[I; N]) -> Self {
                Self::list(val.iter().cloned())
            }
        }

        impl<I: Into<$item>, const N: usize> From<[I; N]> for $ty {
            fn from(val: [I; N]) -> Self {
                Self::list(val)
            }
        }

        impl<'a> TryFrom<&'a $ty> for &'a $item {
            type Error = $crate::Error;

            fn try_from(val: &'a $ty) -> $crate::Result<Self> {
                val.as_single()
            }
        }

        impl TryFrom<$ty> for $item {
            type Error = $crate::Error;

            fn try_from(val: $ty) -> $crate::Result<Self> {
                val.into_single()
            }
        }

        impl<'a> TryFrom<&'a $ty> for &'a [$item] {
            type Error = $crate::Error;

            fn try_from(val: &'a $ty) -> $crate::Result<Self> {
                val.as_list()
            }
        }

        impl From<$ty> for Vec<$item> {
            fn from(val: $ty) -> Self {
                val.into_list()
            }
        }

        $crate::impl_default!($ty);
        $crate::impl_display!($ty, json);
    };
}
