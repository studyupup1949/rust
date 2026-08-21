use std::ops::{Deref, DerefMut};

pub trait AsTraitRef<T: ?Sized>: Sized {
    fn as_trait_ref(&self) -> &T;
}

pub trait AsTraitRefMut<T: ?Sized>: Sized {
    fn as_trait_mut(&mut self) -> &T;
}

macro_rules! impl_as_trait_ref {
    ($trait:path) => {
        impl<T> AsTraitRef<dyn $trait> for T
        where
            T: Sized + $trait + 'static,
        {
            fn as_trait_ref(&self) -> &(dyn $trait + 'static) {
                self
            }
        }
        impl<T> AsTraitRefMut<dyn $trait> for T
        where
            T: Sized + $trait + 'static,
        {
            fn as_trait_mut(&mut self) -> &(dyn $trait + 'static) {
                self
            }
        }
    };
}

impl_as_trait_ref!(std::any::Any);
impl_as_trait_ref!(std::fmt::Debug);
impl_as_trait_ref!(std::fmt::Display);
impl_as_trait_ref!(std::error::Error);
impl_as_trait_ref!(std::io::Read);
impl_as_trait_ref!(std::io::Write);
impl_as_trait_ref!(std::io::BufRead);
impl_as_trait_ref!(std::io::Seek);
impl_as_trait_ref!(std::fmt::Binary);
impl_as_trait_ref!(std::fmt::Octal);
impl_as_trait_ref!(std::fmt::LowerHex);
impl_as_trait_ref!(std::fmt::UpperHex);
impl_as_trait_ref!(std::fmt::Pointer);
impl_as_trait_ref!(std::fmt::LowerExp);
impl_as_trait_ref!(std::fmt::UpperExp);
impl_as_trait_ref!(std::string::ToString);
impl_as_trait_ref!(std::convert::AsRef<[T]>);
impl_as_trait_ref!(std::convert::AsMut<[T]>);
impl_as_trait_ref!(std::borrow::Borrow<[T]>);
impl_as_trait_ref!(std::borrow::BorrowMut<[T]>);

impl<T, U> AsTraitRef<dyn Deref<Target = U>> for T
where
    T: Deref<Target = U> + 'static,
{
    fn as_trait_ref(&self) -> &(dyn Deref<Target = U> + 'static) {
        self
    }
}

impl<T, U> AsTraitRefMut<dyn Deref<Target = U>> for T
where
    T: Deref<Target = U> + 'static,
{
    fn as_trait_mut(&mut self) -> &(dyn Deref<Target = U> + 'static) {
        self
    }
}

impl<T, U> AsTraitRef<dyn DerefMut<Target = U>> for T
where
    T: DerefMut<Target = U> + 'static,
{
    fn as_trait_ref(&self) -> &(dyn DerefMut<Target = U> + 'static) {
        self
    }
}

impl<T, U> AsTraitRefMut<dyn DerefMut<Target = U>> for T
where
    T: DerefMut<Target = U> + 'static,
{
    fn as_trait_mut(&mut self) -> &(dyn DerefMut<Target = U> + 'static) {
        self
    }
}
