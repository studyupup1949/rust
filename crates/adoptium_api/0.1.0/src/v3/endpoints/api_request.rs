#[macro_export]
macro_rules! api_request {
    (
        $(#[$meta:meta])*
        struct $name:ident {
            endpoint: $endpoint:literal,
            request: Get => $response_type:ty,

            $(path: {
                $( $path_name:ident : $path_type:ty ),+ $(,)?
            })?

            $(query: {
                $( $query_name:ident : $query_type:ty ),+ $(,)?
            })?
        }
    ) => {
        api_request!{
            $(#[$meta])*
            struct $name {
                endpoint: $endpoint,

                $(path: {
                    $( $path_name: $path_type ),*
                })?

                $(query: {
                    $( $query_name: $query_type ),*
                })?
            }
        }

        impl $crate::v3::adoptium::GetRequest for $name {
            type Response = $response_type;
        }
    };


    (
        $(#[$meta:meta])*
        struct $name:ident {
            endpoint: $endpoint:literal,

            $(path: {
                $( $path_name:ident : $path_type:ty ),+ $(,)?
            })?

            $(query: {
                $( $query_name:ident : $query_type:ty ),+ $(,)?
            })?
        }
    ) => {
        api_request!{
            $(#[$meta])*
            struct $name {
                $(path: {
                    $( $path_name: $path_type ),*
                })?

                $(query: {
                    $( $query_name : $query_type ),*
                })?
            }
        }

        impl $crate::v3::adoptium::TryAsUrl for $name {
            fn try_as_url(&self, base: &$crate::v3::adoptium::ApiBase) -> Result<url::Url, $crate::v3::adoptium::Error> {
                let endpoint = $endpoint
                    $($( .replace(concat!("{", stringify!($path_name), "}"), &self.$path_name.to_string()) )*)?;

                let url = format!("{base}{}", endpoint);
                #[allow(unused_mut)]
                let mut url = url::Url::parse(&url)?;

                $(
                if $( self.$query_name.is_some() ) || * {
                    let mut query = url.query_pairs_mut();

                    $(
                        if let Some(value) = &self.$query_name {
                            query.append_pair(stringify!($query_name), &value.to_string());
                        }
                    )*
                }
                )?

                Ok(url)
            }
        }
    };


    (
        $(#[$meta:meta])*
        struct $name:ident {
            $(path: {
                $( $path_name:ident : $path_type:ty ),+ $(,)?
            })?

            $(query: {
                $( $query_name:ident : $query_type:ty ),+ $(,)?
            })?
        }
    ) => {
        $(#[$meta])*
        pub struct $name {
            $($( pub $path_name: $path_type, )*)?
            $($( pub $query_name: Option<$query_type>, )*)?
        }

        impl $name {
            #[allow(clippy::too_many_arguments, clippy::new_without_default)]
            pub fn new($($( $path_name: $path_type ),*)?) -> Self {
                Self {
                    $($( $path_name, )*)?
                    $($( $query_name: None, )*)?
                }
            }

            $($(
                pub fn $query_name(mut self, value: $query_type) -> Self {
                    self.$query_name = Some(value);
                    self
                }
            )*)?
        }
    }
}
