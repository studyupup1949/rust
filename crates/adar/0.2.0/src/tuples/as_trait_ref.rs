use core::ops::{Deref, DerefMut};

pub trait AsTraitRef<T: ?Sized> {
    fn as_trait_ref(value: &T) -> &Self;
}
pub trait AsTraitMut<T: ?Sized> {
    fn as_trait_mut(value: &mut T) -> &mut Self;
}

macro_rules! impl_as_trait_ref {
    ($trait:path) => {
        impl<T> AsTraitRef<T> for dyn $trait
        where
            T: $trait + 'static,
        {
            fn as_trait_ref(value: &T) -> &Self {
                value
            }
        }
        impl<T> AsTraitMut<T> for dyn $trait
        where
            T: $trait + 'static,
        {
            fn as_trait_mut(value: &mut T) -> &mut Self {
                value
            }
        }
    };
}

impl_as_trait_ref!(core::any::Any);
impl_as_trait_ref!(core::fmt::Debug);
impl_as_trait_ref!(core::fmt::Display);
impl_as_trait_ref!(core::error::Error);
impl_as_trait_ref!(core::fmt::Binary);
impl_as_trait_ref!(core::fmt::Octal);
impl_as_trait_ref!(core::fmt::LowerHex);
impl_as_trait_ref!(core::fmt::UpperHex);
impl_as_trait_ref!(core::fmt::Pointer);
impl_as_trait_ref!(core::fmt::LowerExp);
impl_as_trait_ref!(core::fmt::UpperExp);
impl_as_trait_ref!(core::convert::AsRef<[T]>);
impl_as_trait_ref!(core::convert::AsMut<[T]>);
impl_as_trait_ref!(core::borrow::Borrow<[T]>);
impl_as_trait_ref!(core::borrow::BorrowMut<[T]>);
#[cfg(feature = "std")]
impl_as_trait_ref!(std::io::Read);
#[cfg(feature = "std")]
impl_as_trait_ref!(std::io::Write);
#[cfg(feature = "std")]
impl_as_trait_ref!(std::io::BufRead);
#[cfg(feature = "std")]
impl_as_trait_ref!(std::io::Seek);
#[cfg(feature = "std")]
impl_as_trait_ref!(std::string::ToString);

impl<T, U> AsTraitRef<T> for dyn Deref<Target = U>
where
    T: Deref<Target = U> + 'static,
{
    fn as_trait_ref(value: &T) -> &Self {
        value
    }
}

impl<T, U> AsTraitMut<T> for dyn Deref<Target = U>
where
    T: Deref<Target = U> + 'static,
{
    fn as_trait_mut(value: &mut T) -> &mut Self {
        value
    }
}

impl<T, U> AsTraitRef<T> for dyn DerefMut<Target = U>
where
    T: DerefMut<Target = U> + 'static,
{
    fn as_trait_ref(value: &T) -> &(dyn DerefMut<Target = U> + 'static) {
        value
    }
}

impl<T, U> AsTraitMut<T> for dyn DerefMut<Target = U>
where
    T: DerefMut<Target = U> + 'static,
{
    fn as_trait_mut(value: &mut T) -> &mut Self {
        value
    }
}
