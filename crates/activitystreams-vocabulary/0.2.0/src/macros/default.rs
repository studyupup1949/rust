/// Helper macro to implement the [`Default`] trait.
#[macro_export]
macro_rules! impl_default {
    ($ty:ident) => {
        impl Default for $ty {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}
