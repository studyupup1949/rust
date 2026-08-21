use core::{
    alloc::Layout,
    error,
    fmt,
    ptr::NonNull,
};

pub(crate) type MemAddr = NonNull<[u8]>;

/// A trait used by pool for internal memory alloc and dealloc when Allocator
/// trait is unstable.
///
/// The impl of this trait is usually a wrapper around the function pair of
/// `alloc` and `dealloc`.
///
/// # Safety
///
/// * Memory blocks returned from an allocator must point to valid memory and
///   retain their validity until the instance and all of its clones are
///   dropped,
///
/// * cloning or moving the allocator must not invalidate memory blocks returned
///   from this allocator. A cloned allocator must behave like the same
///   allocator, and
///
/// * any pointer to a memory block which is currently allocated may be passed
///   to any other method of the allocator.
pub unsafe trait TrMalloc {
    type Err: error::Error;

    fn can_support(&self, layout: Layout) -> bool;

    fn allocate(
        &self,
        layout: Layout,
    ) -> Result<MemAddr, Self::Err>;

    /// Deallocate memory pointed by the pointer
    ///
    /// # Safety
    ///
    /// No content yet
    unsafe fn deallocate(
        &self,
        ptr: MemAddr,
        layout: Layout,
    ) -> Result<usize, Self::Err>;
}

/// A dummy allocator that will do nothing but only return error.
#[derive(Debug, Clone, Default)]
pub struct FakeMalloc;

impl FakeMalloc {
    pub fn shared() -> &'static FakeMalloc {
        static FAKE_ALLOC: FakeMalloc = FakeMalloc;
        &FAKE_ALLOC
    }

    /// Will always returns false for any layout
    pub fn can_support(&self, _: Layout) -> bool {
        false
    }

    /// It will always return `Err`
    pub fn allocate(&self, _: Layout) -> Result<MemAddr, FakeMallocError> {
        Result::Err(FakeMallocError)
    }

    /// Do nothing but return `Result::Ok(0)`
    ///
    /// # Safety
    ///
    /// - This is not designed to be called manually.
    pub unsafe fn deallocate(
        &self,
        _: MemAddr,
        _: Layout,
    ) -> Result<usize, FakeMallocError> {
        Result::Ok(0usize)
    }
}

/// No matter `allocate` or `deallocate`, you are doing it all wrong!
#[derive(Debug, Clone, Copy, Default)]
pub struct FakeMallocError;

unsafe impl TrMalloc for FakeMalloc {
    type Err = FakeMallocError;

    #[inline(always)]
    fn can_support(&self, layout: Layout) -> bool {
        FakeMalloc::can_support(self, layout)
    }

    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<MemAddr, FakeMallocError> {
        FakeMalloc::allocate(self, layout)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, ptr: MemAddr, layout: Layout) -> Result<usize, FakeMallocError> {
        unsafe { FakeMalloc::deallocate(self, ptr, layout) }
    }
}

impl fmt::Display for FakeMallocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FakeMallocError")
    }
}

impl error::Error for FakeMallocError {}

#[cfg(any(test, feature = "core_alloc"))]
pub use crate::core_alloc_::{CoreAlloc, CoreAllocError};
