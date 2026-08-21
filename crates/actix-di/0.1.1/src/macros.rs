#[macro_export]
macro_rules! inject_service {
    ($state:expr, $service:ty) => {
        $state.get::<$service>()
            .ok_or_else(|| $crate::error::ServiceError::ServiceNotFound(std::any::TypeId::of::<$service>()))
    };
}

#[macro_export]
macro_rules! provide_dependencies {
    ($($dep:ty),*) => {
        fn required_services() -> Vec<std::any::TypeId> {
            vec![
                $(std::any::TypeId::of::<$dep>()),*
            ]
        }
    };
}

/// Helper macro for creating modules
#[macro_export]
macro_rules! define_module {
    ($name:ident { $($field:ident: $type:ty),* $(,)? }) => {
        pub struct $name {
            $($field: $type,)*
        }

        impl $name {
            pub fn new($($field: $type),*) -> Self {
                Self {
                    $($field,)*
                }
            }
        }
    };
}