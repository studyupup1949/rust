use core::{
    cell::SyncUnsafeCell,
    future::{self, IntoFuture},
    marker::PhantomPinned,
    pin::Pin,
};

pub trait TrCancellationToken: Clone {
    fn is_cancelled(&self) -> bool;

    fn can_be_cancelled(&self) -> bool;

    fn cancellation(self: Pin<&mut Self>) -> impl IntoFuture<Output = ()>;
}

/// An instance of [IntoFuture] for an async task that may or may not be
/// cancelled by an optional cancellation token.
///
/// Note: the lifetime here is required by `rustc` when implementing 
/// [TrMayCancel] for your type. Along with future release of rustc, the `<'a>`
/// may be removed.
pub trait TrMayCancel<'a>
where
    Self: 'a + Sized + IntoFuture
{
    type MayCancelOutput;

    fn may_cancel_with<'f, C: TrCancellationToken>(
        self,
        cancel: Pin<&'f mut C>,
    ) -> impl IntoFuture<Output = Self::MayCancelOutput>
    where
        Self: 'f;
}

/// A token that is already cancelled and will no never reset.
#[derive(Debug, Default, Clone, Copy)]
pub struct CancelledToken(PhantomPinned);

impl CancelledToken {
    /// Create an instance of `CancelledToken`
    pub const fn new() -> Self {
        CancelledToken(PhantomPinned)
    }

    /// Get a pin pointer to the global shared instance of `CancelledToken`.
    /// 
    /// ## Example
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use abs_sync::cancellation::CancelledToken;
    /// 
    /// let mut token = CancelledToken::pinned();
    /// assert!(token.is_cancelled());
    /// assert!(!token.can_be_cancelled());
    /// 
    /// token.as_mut().cancellation().await;
    /// # })
    /// ```
    pub fn pinned() -> Pin<&'static mut Self> {
        static mut SHARED: SyncUnsafeCell<CancelledToken> =
            SyncUnsafeCell::new(CancelledToken::new());
        unsafe {
            #[allow(static_mut_refs)]
            Pin::new_unchecked(SHARED.get_mut())
        }
    }

    /// Always true
    pub const fn is_cancelled(&self) -> bool {
        true
    }
    /// Always false
    pub const fn can_be_cancelled(&self) -> bool {
        false
    }

    /// Always return a ready future.
    pub fn cancellation(self: Pin<&mut Self>) -> future::Ready<()> {
        future::ready(())
    }
}

impl TrCancellationToken for CancelledToken {
    #[inline]
    fn is_cancelled(&self) -> bool {
        CancelledToken::is_cancelled(self)
    }

    #[inline]
    fn can_be_cancelled(&self) -> bool {
        CancelledToken::can_be_cancelled(self)
    }

    #[inline]
    fn cancellation(self: Pin<&mut Self>) -> impl IntoFuture<Output = ()> {
        CancelledToken::cancellation(self)
    }
}

/// A cancellation token that will never be cancelled, usually used
/// as a dummy for `TrCancellationToken`.
#[derive(Debug, Default, Clone, Copy)]
pub struct NonCancellableToken(PhantomPinned);

impl NonCancellableToken {
    pub const fn new() -> Self {
        NonCancellableToken(PhantomPinned)
    }

    /// Get a pin pointer to the global shared instance of `NonCancelledToken`.
    /// 
    /// ## Example
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use abs_sync::{
    ///     cancellation::NonCancellableToken,
    ///     ok_or::XtOkOr,
    /// };
    /// 
    /// let mut token = NonCancellableToken::pinned();
    /// assert!(!token.is_cancelled());
    /// assert!(!token.can_be_cancelled());
    /// 
    /// let signal = token.as_mut().cancellation();
    /// let answer = async { 42 };
    /// assert!(signal.ok_or(answer).await.is_err());
    /// # })
    /// ```
    pub fn pinned() -> Pin<&'static mut Self> {
        static mut SHARED: SyncUnsafeCell<NonCancellableToken> =
            SyncUnsafeCell::new(NonCancellableToken::new());
        unsafe {
            #[allow(static_mut_refs)]
            Pin::new_unchecked(SHARED.get_mut())
        }
    }

    /// Always false
    pub const fn is_cancelled(&self) -> bool {
        false
    }

    /// Always false
    pub const fn can_be_cancelled(&self) -> bool {
        false
    }

    /// Always returns a pending future.
    pub fn cancellation(self: Pin<&mut Self>) -> future::Pending<()> {
        future::pending()
    }
}

impl TrCancellationToken for NonCancellableToken {
    #[inline]
    fn is_cancelled(&self) -> bool {
        NonCancellableToken::is_cancelled(self)
    }

    #[inline]
    fn can_be_cancelled(&self) -> bool {
        NonCancellableToken::can_be_cancelled(self)
    }

    #[inline]
    fn cancellation(self: Pin<&mut Self>) -> impl IntoFuture<Output = ()> {
        NonCancellableToken::cancellation(self)
    }
}
