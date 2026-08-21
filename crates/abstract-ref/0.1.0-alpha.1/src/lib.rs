use std::borrow::Borrow;

pub mod types;

pub trait Type<T: ?Sized> {
    type RefT;
}
pub trait MakeMutType<T: ?Sized>: Type<T> {
    fn make_mut(p: &mut Self::RefT) -> &mut T;
}

pub struct Ref<Tp: Type<T>, T: ?Sized>(Tp::RefT);

impl<Tp: Type<T>, T: ?Sized> AsRef<T> for Ref<Tp, T>
where
    Tp::RefT: Borrow<T>,
{
    fn as_ref(&self) -> &T {
        self.0.borrow()
    }
}
impl<Tp: Type<T>, T: ?Sized> std::borrow::Borrow<T> for Ref<Tp, T>
where
    Tp::RefT: Borrow<T>,
{
    fn borrow(&self) -> &T {
        self.0.borrow()
    }
}
impl<Tp: Type<T>, T: ?Sized> std::ops::Deref for Ref<Tp, T>
where
    Tp::RefT: Borrow<T>,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.0.borrow()
    }
}
impl<Tp: MakeMutType<T>, T: ?Sized> Ref<Tp, T> {
    pub fn make_mut(&mut self) -> &mut T {
        Tp::make_mut(&mut self.0)
    }
}
impl<Tp: Type<T>, T: ?Sized> Clone for Ref<Tp, T>
where
    Tp::RefT: Clone,
{
    fn clone(&self) -> Self {
        Ref(self.0.clone())
    }
}
impl<Tp: Type<T>, T> From<T> for Ref<Tp, T>
where
    Tp::RefT: From<T>
{
    fn from(v: T) -> Self {
        Ref(Tp::RefT::from(v))
    }
}

#[cfg(test)]
mod tests;
