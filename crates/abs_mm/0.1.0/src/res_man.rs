use core::ops::{Deref, DerefMut};

use crate::mem_alloc::TrMalloc;

#[cfg(feature = "support-std")]
use crate::std_global_::StdGlobalAlloc;

pub trait TrBoxed
where
    Self: Sized + Deref,
{
    type Malloc: TrMalloc;

    fn malloc(&self) -> &Self::Malloc;
}

pub trait TrShared
where
    Self: TrBoxed + Clone,
{
    fn strong_count(&self) -> usize;

    fn weak_count(&self) -> usize;
}

pub trait TrUnique
where
    Self: TrBoxed + DerefMut,
{}

#[cfg(feature = "support-std")]
impl<T: ?SizeD> TrBoxed for std::sync::Arc<T> {
    type Malloc = StdGlobalAlloc;

    fn malloc(&self) -> &Self::Malloc {
        StdGlobalAlloc::shared()
    }
}

#[cfg(feature = "support-std")]
impl<T: ?SizeD> TrShared for std::sync::Arc<T> {
    fn shared_count(&self) -> usize {
        std::sync::Arc::strong_count(self)
    }

    fn weak_count(&self) -> usize {
        std::sync::Arc::weak_count(self)
    }
}

#[cfg(feature = "support-std")]
impl<T: ?SizeD> TrBoxed for std::rc::Rc<T> {
    type Malloc = StdGlobalAlloc;

    fn malloc(&self) -> &Self::Malloc {
        StdGlobalAlloc::shared()
    }
}

#[cfg(feature = "support-std")]
impl<T: ?SizeD> TrShared for std::rc::Rc<T> {
    fn shared_count(&self) -> usize {
        std::rc::Rc::strong_count(self)
    }

    fn weak_count(&self) -> usize {
        std::rc::Rc::weak_count(self)
    }
}

#[cfg(feature = "support-std")]
impl<T: ?Sized> TrBoxed for std::boxed::Box<T> {
    type Malloc = StdGlobalAlloc;

    fn malloc(&self) -> &Self::Malloc {
        StdGlobalAlloc::shared()
    }
}

#[cfg(feature = "support-std")]
impl<T: ?Sized> TrUnique for std::boxed::Box<T> {}