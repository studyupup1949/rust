macro_rules! classes {
    ($($var:ident: $id:literal, $public_name:literal, $subclass_registry_owner:literal, $registrar:literal;)*) => {

        macro_rules! doc_public_name{
            $(($var) => {$public_name};)*
        }

        /// Enum for all known Device Classes
        ///
        /// This enum is strongly typed. Only registered device classes are present
        #[cfg_attr(feature = "fixed-repr", repr(u16))]
        #[non_exhaustive]
        pub enum DeviceClass{

            $(#[doc = concat!("The ", $public_name, " device class")] $var = $id),*
        }

        impl DeviceClass{
            /// Returns the numeric id associated with this [`DeviceClass`]
            pub const fn id(&self) -> u16{
                match self{
                    $(Self:: $var => $id),*
                }
            }

            /// Obtains the [`DeviceClass`] associated with the given numeric `id` if one is provided, otherwise returns `None`.
            ///
            ///
            /// ## Stability
            /// A `None` result of this function should not be considered stable, as a new registration may cause a previous `None` result to return a value.
            /// A `Some` result is guaranteed to remain stable permanently (including accross major versions)
            pub const fn from_id(id: u16) -> Option<Self>{
                match id{
                    $($id => Some(Self:: $var),)*
                    _ => None
                }
            }

            /// Retrieves the public (display) name from the registration associated with this [`DeviceClass`].
            ///
            /// This function requires the `extended-info` crate feature to be enabled.
            ///
            /// ## Stability
            ///
            /// As the public name of a registration may be amended, the return value of this function is not considered stable.
            #[cfg(feature = "extended-info")]
            pub const fn public_name(&self) -> &'static str{
                match self{
                    $(Self:: $var => $public_name),*
                }
            }
        }
    }
}

macro_rules! vendors {
    ($($var:ident: $id:literal, $public_name:literal, $vendor_registry_owner:literal, $registrar:literal;)*) => {

        /// Enum for all known Device Vendors
        ///
        /// This enum is strongly typed. Only registered device vendors are present
        #[cfg_attr(feature = "fixed-repr", repr(u16))]
        #[non_exhaustive]
        pub enum DeviceVendor{
            $(#[doc = concat!("The ", $public_name, " Device Vendor")] $var = $id),*
        }

        impl DeviceVendor{
            /// Returns the numeric id associated with this [`DeviceVendor`]
            pub const fn id(&self) -> u16{
                match self{
                    $(Self:: $var => $id),*
                }
            }

            /// Obtains the [`DeviceVendor`] associated with the given numeric `id` if one is provided, otherwise returns `None`.
            ///
            ///
            /// ## Stability
            /// A `None` result of this function should not be considered stable, as a new registration may cause a previous `None` result to return a value.
            /// A `Some` result is guaranteed to remain stable permanently (including accross major versions)
            pub const fn from_id(id: u16) -> Option<Self>{
                match id{
                    $($id => Some(Self:: $var),)*
                    _ => None
                }
            }


            /// Retrieves the public (display) name from the registration associated with this [`DeviceVendor`].
            ///
            /// This function requires the `extended-info` crate feature to be enabled.
            ///
            /// ## Stability
            ///
            /// As the public name of a registration may be amended, the return value of this function is not considered stable.
            #[cfg(feature = "extended-info")]
            pub const fn public_name(&self) -> &'static str{
                match self{
                    $(Self:: $var => $public_name),*
                }
            }
        }
    }
}

macro_rules! well_known_subclass {
    ($enum:ident @ $class:ident: {$($var:ident: $id:literal, $public_name:literal, $specification:literal, $registrar:literal;)*}) => {

        #[doc = concat!("Enum for all known subclasses of the ", doc_public_name!($class)," device class")]
        #[cfg_attr(feature = "fixed-repr", repr(u16))]
        #[non_exhaustive]
        pub enum $enum{
            $(#[doc = concat!("The ", $public_name, " subclass")] $var = $id),*
        }

        impl core::convert::From<$enum> for crate::SubclassId{
            fn from(x: $enum) -> Self{
                x.into_generic()
            }
        }

        impl $enum{
            #[doc = concat!("Returns the numeric id associated with the [`", stringify!($enum),"`].")]
            pub const fn id(&self) -> u16{
                match self{
                    $(Self:: $var => $id),*
                }
            }

            #[doc = concat!("Returns the [`", stringify!($enum),"`] associated with the given id if it exists, or otherwise returns `None`.")]
            /// ## Stability
            /// A `None` result of this function should not be considered stable, as a new registration may cause a previous `None` result to return a value.
            /// A `Some` result is guaranteed to remain stable permanently (including accross major versions)
            pub const fn from_id(id: u16) -> Option<Self>{
                match id{
                    $($id => Some(Self:: $var),)*
                    _ => None
                }
            }

            #[doc = concat!("Converts the [`", stringify!($enum),"`] into a [`SubclassId`][crate::SubclassId]")]
            pub const fn into_generic(self) -> crate::SubclassId{
                crate::SubclassId::from_id(self.id())
            }

            #[cfg(feature = "extended-info")]
            #[doc = concat!("Retrieves the public (display) name from the registration associated with this [`", stringify!($enum), "`].")]
            ///
            /// This requires the `extened-info` crate feature to be enabled.
            ///
            /// ## Stability
            ///
            /// As the public name of a registration may be amended, the return value of this function is not considered stable.
            pub const fn public_name(&self) -> &'static str{
                match self{
                    $(Self:: $var => $public_name),*
                }
            }

            #[cfg(feature = "extended-info")]
            #[doc = concat!("Retrieves the Uniform Resource Location of the Specification from the registration associated with this [`", stringify!($enum), "`].")]
            ///
            /// This requires the `extened-info` crate feature to be enabled.
            ///
            /// ## Stability
            ///
            /// As the Specification URL of a registration may be amended, the return value of this function is not considered stable.
            pub const fn specification(&self) -> &'static str{
                match self{
                    $(Self:: $var => $specification),*
                }
            }
        }
    }
}

include!(env!("TABLES_GENERATED"));
