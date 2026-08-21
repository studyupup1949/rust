use std::{borrow::BorrowMut, marker::PhantomData};

pub struct Cow<'a>(PhantomData<&'a ()>);
impl<'a, T> crate::Type<T> for Cow<'a>
where
    T: ?Sized + ToOwned + 'a,
{
    type RefT = std::borrow::Cow<'a, T>;
}
impl<'a, T> crate::MakeMutType<T> for Cow<'a>
where
    T: ?Sized + ToOwned + 'a,
    T::Owned: BorrowMut<T>,
{
    fn make_mut(p: &mut Self::RefT) -> &mut T {
        p.to_mut().borrow_mut()
    }
}

pub struct Rc;
impl<T: ?Sized> crate::Type<T> for Rc {
    type RefT = std::rc::Rc<T>;
}
impl<T: ?Sized + Clone> crate::MakeMutType<T> for Rc {
    fn make_mut(p: &mut Self::RefT) -> &mut T {
        std::rc::Rc::make_mut(p)
    }
}

pub struct Arc;
impl<T: ?Sized> crate::Type<T> for Arc {
    type RefT = std::sync::Arc<T>;
}
impl<T: ?Sized + Clone> crate::MakeMutType<T> for Arc {
    fn make_mut(p: &mut Self::RefT) -> &mut T {
        std::sync::Arc::make_mut(p)
    }
}

pub struct Box;
impl<T: ?Sized> crate::Type<T> for Box {
    type RefT = std::boxed::Box<T>;
}
impl<T: ?Sized> crate::MakeMutType<T> for Box {
    fn make_mut(p: &mut Self::RefT) -> &mut T {
        &mut *p
    }
}

pub type Own = ();
impl<T> crate::Type<T> for () {
    type RefT = T;
}
impl<T> crate::MakeMutType<T> for () {
    fn make_mut(p: &mut Self::RefT) -> &mut T {
        p
    }
}

pub struct Brw<'a>(PhantomData<&'a ()>);
impl<'a, T: ?Sized + 'a> crate::Type<T> for Brw<'a> {
    type RefT = &'a T;
}

pub struct BrwMut<'a>(PhantomData<&'a ()>);
impl<'a, T: ?Sized + 'a> crate::Type<T> for BrwMut<'a> {
    type RefT = &'a mut T;
}
impl<'a, T: ?Sized + 'a> crate::MakeMutType<T> for BrwMut<'a> {
    fn make_mut(p: &mut Self::RefT) -> &mut T {
        p
    }
}
