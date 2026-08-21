/// Helper macro to implement the [ActivityVocabulary](crate::ActivityVocabulary) trait.
#[macro_export]
macro_rules! impl_activity_vocabulary {
    ($ty:ident) => {
        impl $crate::ActivityVocabulary for $ty {
            type Type = $ty;

            fn kind(&self) -> String {
                self.as_str().into()
            }

            fn contains(&self, kind: &str) -> bool {
                self.as_str() == kind
            }
        }
    };
}

/// Helper macro to convert a type into `VocabularyType` and `VocabularyTypes`.
#[macro_export]
macro_rules! impl_into_vocabulary {
    ($ty:ident) => {
        impl From<$ty> for $crate::VocabularyType {
            fn from(val: $ty) -> Self {
                use $crate::ActivityVocabulary;

                Self::Iri(val.kind().try_into().unwrap_or_default())
            }
        }
    };
}
