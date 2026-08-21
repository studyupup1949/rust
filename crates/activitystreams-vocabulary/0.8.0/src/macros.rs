//! Helper macros.

mod activity;
mod actor;
mod default;
mod display;
mod item;
mod link;
mod list;
mod object;
mod vocabulary;

/// Helper to define field access functions for types.
///
/// Helps to cut down boilerplate, especially for types with many fields.
#[macro_export]
macro_rules! field_access {
    // Field access for fields with the same getter and setter type.
    ($ty:ident$(<$vocab_ty:ident>)? {
        $(
            $(#[$field_doc:meta])*
            $field:ident: $field_ty:ty $(,)?
        )+
    }) => {
        $crate::paste! {
            impl$(<$vocab_ty: $crate::ActivityVocabulary>)? $ty$(<$vocab_ty>)? {
                $(
                    #[doc = "Gets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub const fn $field(&self) -> $field_ty {
                        self.$field
                    }

                    #[doc = "Sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<set_ $field>]<I: Into<$field_ty>>(&mut self, val: I) {
                        self.$field = val.into();
                    }

                    #[doc = "Builder function that sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    #[allow(clippy::needless_update)]
                    pub fn [<with_ $field>]<I: Into<$field_ty>>(self, val: I) -> Self {
                        Self {
                            $field: val.into(),
                            ..self
                        }
                    }
                )+
            }
        }
    };

    // Field access for fields with different getter and setter types.
    ($ty:ident$(<$vocab_ty:ident>)? {
        $(
            $(#[$field_doc:meta])*
            $field:ident: as_ref { $ref_ty:ty, $field_ty:ty } $(,)?
        )+
    }) => {
        $crate::paste! {
            impl$(<$vocab_ty: $crate::ActivityVocabulary>)? $ty$(<$vocab_ty>)? {
                $(
                    #[doc = "Gets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn $field(&self) -> $ref_ty {
                        &self.$field
                    }

                    #[doc = "Sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<set_ $field>]<I: Into<$field_ty>>(&mut self, val: I) {
                        self.$field = val.into();
                    }

                    #[doc = "Builder function that sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    #[allow(clippy::needless_update)]
                    pub fn [<with_ $field>]<I: Into<$field_ty>>(self, val: I) -> Self {
                        Self {
                            $field: val.into(),
                            ..self
                        }
                    }
                )+
            }
        }
    };

    // Field access for fields with the same getter and setter types.
    ($ty:ident$(<$vocab_ty:ident>)? {
        $(
            $(#[$field_doc:meta])*
            $field:ident: as_ref { $field_ty:ty } $(,)?
        )+
    }) => {
        $crate::paste! {
            impl$(<$vocab_ty: $crate::ActivityVocabulary>)? $ty$(<$vocab_ty>)? {
                $(
                    #[doc = "Gets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn $field(&self) -> &$field_ty {
                        &self.$field
                    }

                    #[doc = "Sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<set_ $field>]<I: Into<$field_ty>>(&mut self, val: I) {
                        self.$field = val.into();
                    }

                    #[doc = "Builder function that sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    #[allow(clippy::needless_update)]
                    pub fn [<with_ $field>]<I: Into<$field_ty>>(self, val: I) -> Self {
                        Self {
                            $field: val.into(),
                            ..self
                        }
                    }
                )+
            }
        }
    };

    // Field access for `Option`-wrapped types.
    ($ty:ident$(<$vocab_ty:ident>)? {
        $(
            $(#[$field_doc:meta])*
            $field:ident: option { $field_ty:ty } $(,)?
        )+
    }) => {
        $crate::paste! {
            impl$(<$vocab_ty: $crate::ActivityVocabulary>)? $ty$(<$vocab_ty>)? {
                $(
                    #[doc = "Gets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub const fn $field(&self) -> Option<$field_ty> {
                        self.$field
                    }

                    #[doc = "Sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<set_ $field>]<I: Into<$field_ty>>(&mut self, val: I) {
                        self.$field = Some(val.into());
                    }

                    #[doc = "Unsets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<unset_ $field>](&mut self) -> Option<$field_ty> {
                        self.$field.take()
                    }

                    #[doc = "Builder function that sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    #[allow(clippy::needless_update)]
                    pub fn [<with_ $field>]<I: Into<$field_ty>>(self, val: I) -> Self {
                        Self {
                            $field: Some(val.into()),
                            ..self
                        }
                    }
                )+
            }
        }
    };

    // Field access for types that return `Option::as_deref` for the getter.
    ($ty:ident$(<$vocab_ty:ident>)? {
        $(
            $(#[$field_doc:meta])*
            $field:ident: option_deref { $ref_ty:ty, $field_ty:ty } $(,)?
        )+
    }) => {
        $crate::paste! {
            impl$(<$vocab_ty: $crate::ActivityVocabulary>)? $ty$(<$vocab_ty>)? {
                $(
                    #[doc = "Gets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn $field(&self) -> Option<$ref_ty> {
                        self.$field.as_deref()
                    }

                    #[doc = "Sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<set_ $field>]<I: Into<$field_ty>>(&mut self, val: I) {
                        self.$field = Some(val.into());
                    }

                    #[doc = "Unsets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<unset_ $field>](&mut self) -> Option<$field_ty> {
                        self.$field.take()
                    }

                    #[doc = "Builder function that sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    #[allow(clippy::needless_update)]
                    pub fn [<with_ $field>]<I: Into<$field_ty>>(self, val: I) -> Self {
                        Self {
                            $field: Some(val.into()),
                            ..self
                        }
                    }
                )+
            }
        }
    };

    // Field access for types that return `Option::as_deref` for the getter.
    ($ty:ident$(<$vocab_ty:ident>)? {
        $(
            $(#[$field_doc:meta])*
            $field:ident: option_box_deref { $ref_ty:ty, $field_ty:ty } $(,)?
        )+
    }) => {
        $crate::paste! {
            impl$(<$vocab_ty: $crate::ActivityVocabulary>)? $ty$(<$vocab_ty>)? {
                $(
                    #[doc = "Gets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn $field(&self) -> Option<$ref_ty> {
                        self.$field.as_deref()
                    }

                    #[doc = "Sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<set_ $field>]<I: Into<$field_ty>>(&mut self, val: I) {
                        self.$field = Some(val.into());
                    }

                    #[doc = "Unsets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<unset_ $field>](&mut self) -> Option<$field_ty> {
                        self.$field.take()
                    }

                    #[doc = "Builder function that sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    #[allow(clippy::needless_update)]
                    pub fn [<with_ $field>]<I: Into<$field_ty>>(self, val: I) -> Self {
                        Self {
                            $field: Some(val.into()),
                            ..self
                        }
                    }
                )+
            }
        }
    };

    // Field access for types that return `Option::as_deref` for the getter.
    ($ty:ident$(<$vocab_ty:ident>)? {
        $(
            $(#[$field_doc:meta])*
            $field:ident: option_box_deref { $field_ty:ty } $(,)?
        )+
    }) => {
        $crate::paste! {
            impl$(<$vocab_ty: $crate::ActivityVocabulary>)? $ty$(<$vocab_ty>)? {
                $(
                    #[doc = "Gets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn $field(&self) -> Option<&$field_ty> {
                        self.$field.as_deref()
                    }

                    #[doc = "Sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<set_ $field>]<I: Into<$field_ty>>(&mut self, val: I) {
                        self.$field = Some(Box::new(val.into()));
                    }

                    #[doc = "Unsets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<unset_ $field>](&mut self) -> Option<Box<$field_ty>> {
                        self.$field.take()
                    }

                    #[doc = "Builder function that sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    #[allow(clippy::needless_update)]
                    pub fn [<with_ $field>]<I: Into<$field_ty>>(self, val: I) -> Self {
                        Self {
                            $field: Some(Box::new(val.into())),
                            ..self
                        }
                    }
                )+
            }
        }
    };

    // Field access for types that return `Option::as_ref` for the getter.
    ($ty:ident$(<$vocab_ty:ident>)? {
        $(
            $(#[$field_doc:meta])*
            $field:ident: option_ref { $ref_ty:ty, $field_ty:ty } $(,)?
        )+
    }) => {
        $crate::paste! {
            impl$(<$vocab_ty: $crate::ActivityVocabulary>)? $ty$(<$vocab_ty>)? {
                $(
                    #[doc = "Gets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn $field(&self) -> Option<$ref_ty> {
                        self.$field.as_ref()
                    }

                    #[doc = "Sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<set_ $field>]<I: Into<$field_ty>>(&mut self, val: I) {
                        self.$field = Some(val.into());
                    }

                    #[doc = "Unsets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<unset_ $field>](&mut self) -> Option<$field_ty> {
                        self.$field.take()
                    }

                    #[doc = "Builder function that sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    #[allow(clippy::needless_update)]
                    pub fn [<with_ $field>]<I: Into<$field_ty>>(self, val: I) -> Self {
                        Self {
                            $field: Some(val.into()),
                            ..self
                        }
                    }
                )+
            }
        }
    };

    // Field access for types that return `Option::as_ref` for the getter.
    ($ty:ident$(<$vocab_ty:ident>)? {
        $(
            $(#[$field_doc:meta])*
            $field:ident: option_ref { $field_ty:ty } $(,)?
        )+
    }) => {
        $crate::paste! {
            impl$(<$vocab_ty: $crate::ActivityVocabulary>)? $ty$(<$vocab_ty>)? {
                $(
                    #[doc = "Gets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn $field(&self) -> Option<&$field_ty> {
                        self.$field.as_ref()
                    }

                    #[doc = "Sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<set_ $field>]<I: Into<$field_ty>>(&mut self, val: I) {
                        self.$field = Some(val.into());
                    }

                    #[doc = "Unsets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    pub fn [<unset_ $field>](&mut self) -> Option<$field_ty> {
                        self.$field.take()
                    }

                    #[doc = "Builder function that sets the [" $ty "] `" $field "`."]
                    $(
                        #[doc = ""]
                        #[$field_doc]
                    )*
                    #[allow(clippy::needless_update)]
                    pub fn [<with_ $field>]<I: Into<$field_ty>>(self, val: I) -> Self {
                        Self {
                            $field: Some(val.into()),
                            ..self
                        }
                    }
                )+
            }
        }
    };
}
