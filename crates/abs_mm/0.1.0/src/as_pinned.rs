use core::pin::Pin;

pub trait TrAsPinned<T: ?Sized> {
    fn as_pinned(self: Pin<&mut Self>) -> Pin<&mut T>;
}
