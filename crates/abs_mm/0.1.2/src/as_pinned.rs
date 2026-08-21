extern crate alloc;

use core::pin::Pin;

pub trait TrAsPinned<'a, T: ?Sized>
where
    Self: 'a,
    T: 'a,
{
    fn as_pinned(self) -> Pin<&'a T>;
}

/// A trait specifically for `Pin` pointer 
pub trait TrAsPinnedMut<'a, T: ?Sized>
where
    Self: 'a,
    T: 'a,
{
    fn as_pinned_mut(self) -> Pin<&'a mut T>;
}

#[cfg(feature = "arc")]
impl<'a, T: ?Sized> TrAsPinned <'a, T> for Pin<alloc::sync::Arc<T>>
where
    Self: 'a,
    T: 'a,
{
    fn as_pinned(self) -> Pin<&'a T> {
        unsafe { 
            let t = self.as_ref().get_ref() as *const T;
            Pin::new_unchecked(&*t)
        }
    }
}

#[cfg(feature = "rc")]
impl<'a, T: ?Sized> TrAsPinned <'a, T> for Pin<alloc::rc::Rc<T>>
where
    Self: 'a,
    T: 'a,
{
    fn as_pinned(self) -> Pin<&'a T> {
        unsafe { 
            let t = self.as_ref().get_ref() as *const T;
            Pin::new_unchecked(&*t)
        }
    }
}

#[cfg(feature = "box")]
impl<'a, T: ?Sized> TrAsPinned <'a, T> for Pin<alloc::boxed::Box<T>>
where
    Self: 'a,
    T: 'a,
{
    fn as_pinned(self) -> Pin<&'a T> {
        unsafe { 
            let t = self.as_ref().get_ref() as *const T;
            Pin::new_unchecked(&*t)
        }
    }
}

#[cfg(feature = "box")]
impl<'a, T: ?Sized> TrAsPinnedMut <'a, T> for Pin<alloc::boxed::Box<T>>
where
    Self: 'a,
    T: 'a,
{
    fn as_pinned_mut(mut self) -> Pin<&'a mut T> {
        unsafe { 
            let t = self.as_mut().get_unchecked_mut() as *mut T;
            Pin::new_unchecked(&mut *t)
        }
    }
}
