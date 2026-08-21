extern crate alloc;

use core::ops::{Deref, DerefMut};

use crate::mem_alloc::{CoreAlloc, TrMalloc};

/// Smart pointers that can retrieve its memory allocator.
pub trait TrBoxed
where
    Self: Sized + Deref,
{
    type Malloc: TrMalloc;

    fn malloc(&self) -> &Self::Malloc;
}

/// A trait describing smart pointers that share the ownership of the resource
/// with reference counting.
pub trait TrShared
where
    Self: TrBoxed + Clone,
{
    type Item: ?Sized;
    type Downgraded: TrWeak<Item = Self::Item>;

    fn strong_count(&self) -> usize;

    fn weak_count(&self) -> usize;

    fn downgrade(&self) -> Self::Downgraded;
}

pub trait TrWeak {
    type Item: ?Sized;
    type Upgraded: TrShared<Item = Self::Item>;

    fn strong_count(&self) -> usize;

    fn weak_count(&self) -> usize;

    fn upgrade(&self) -> Option<Self::Upgraded>;
}

/// A trait describing smart pointers with a unique owner.
pub trait TrUnique
where
    Self: TrBoxed + DerefMut,
{
    type Item: ?Sized;
}

#[cfg(feature = "arc")]
impl<T: ?Sized> TrBoxed for alloc::sync::Arc<T> {
    type Malloc = CoreAlloc;

    #[inline]
    fn malloc(&self) -> &Self::Malloc {
        CoreAlloc::shared()
    }
}

#[cfg(feature = "arc")]
impl<T: ?Sized> TrShared for alloc::sync::Arc<T> {
    type Item = T;
    type Downgraded = alloc::sync::Weak<T>;

    #[inline]
    fn strong_count(&self) -> usize {
        alloc::sync::Arc::strong_count(self)
    }

    #[inline]
    fn weak_count(&self) -> usize {
        alloc::sync::Arc::weak_count(self)
    }

    #[inline]
    fn downgrade(&self) -> Self::Downgraded {
        alloc::sync::Arc::downgrade(self)
    }
}

#[cfg(feature = "arc")]
impl<T: ?Sized> TrWeak for alloc::sync::Weak<T> {
    type Item = T;
    type Upgraded = alloc::sync::Arc<T>;

    #[inline]
    fn strong_count(&self) -> usize {
        alloc::sync::Weak::strong_count(self)
    }

    #[inline]
    fn weak_count(&self) -> usize {
        alloc::sync::Weak::weak_count(self)
    }

    #[inline]
    fn upgrade(&self) -> Option<Self::Upgraded> {
        alloc::sync::Weak::upgrade(self)
    }
}

#[cfg(feature = "rc")]
impl<T: ?Sized> TrBoxed for alloc::rc::Rc<T> {
    type Malloc = CoreAlloc;

    #[inline]
    fn malloc(&self) -> &Self::Malloc {
        CoreAlloc::shared()
    }
}

#[cfg(feature = "rc")]
impl<T: ?Sized> TrShared for alloc::rc::Rc<T> {
    type Item = T;
    type Downgraded = alloc::rc::Weak<T>;

    #[inline]
    fn strong_count(&self) -> usize {
        alloc::rc::Rc::strong_count(self)
    }

    #[inline]
    fn weak_count(&self) -> usize {
        alloc::rc::Rc::weak_count(self)
    }

    #[inline]
    fn downgrade(&self) -> Self::Downgraded {
        alloc::rc::Rc::downgrade(self)
    }
}

#[cfg(feature = "rc")]
impl<T: ?Sized> TrWeak for alloc::rc::Weak<T> {
    type Item = T;
    type Upgraded = alloc::rc::Rc<T>;

    #[inline]
    fn strong_count(&self) -> usize {
        alloc::rc::Weak::strong_count(self)
    }

    #[inline]
    fn weak_count(&self) -> usize {
        alloc::rc::Weak::weak_count(self)
    }

    #[inline]
    fn upgrade(&self) -> Option<Self::Upgraded> {
        alloc::rc::Weak::upgrade(self)
    }
}

#[cfg(feature = "box")]
impl<T: ?Sized> TrBoxed for alloc::boxed::Box<T> {
    type Malloc = CoreAlloc;

    fn malloc(&self) -> &Self::Malloc {
        CoreAlloc::shared()
    }
}

#[cfg(feature = "box")]
impl<T: ?Sized> TrUnique for alloc::boxed::Box<T> {
    type Item = T;
}

#[cfg(test)]
mod tests_ {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "arc")]
    #[test]
    fn arc_should_impl_shared() {
        use alloc::sync::{Arc, Weak};

        let arc = Arc::new(());
        let weak = TrShared::downgrade(&arc);
        assert_eq!(Arc::strong_count(&arc), TrShared::strong_count(&arc));
        assert_eq!(Weak::weak_count(&weak), TrWeak::weak_count(&weak));

        let upgraded = TrWeak::upgrade(&weak).unwrap();
        assert_eq!(Arc::strong_count(&arc), upgraded.strong_count());
        assert_eq!(upgraded.strong_count(), 2);
    }

    #[cfg(feature = "rc")]
    #[test]
    fn rc_should_impl_shared() {
        use alloc::rc::{Rc, Weak};

        let rc = Rc::new(());
        let weak = TrShared::downgrade(&rc);
        assert_eq!(Rc::strong_count(&rc), TrShared::strong_count(&rc));
        assert_eq!(Weak::weak_count(&weak), TrWeak::weak_count(&weak));

        let upgraded = TrWeak::upgrade(&weak).unwrap();
        assert_eq!(Rc::strong_count(&rc), upgraded.strong_count());
        assert_eq!(upgraded.strong_count(), 2);
    }

    #[cfg(feature = "box")]
    #[test]
    fn box_should_impl_unique() {
        let p = alloc::boxed::Box::new(());
        assert!(std::ptr::eq(p.malloc(), CoreAlloc::shared()));
    }
}
