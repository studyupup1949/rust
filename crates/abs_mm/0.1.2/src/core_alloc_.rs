extern crate alloc;

use alloc::alloc::{alloc, dealloc};

use core::{
    alloc::{Layout, LayoutError},
    error, fmt,
    ptr::{self, NonNull},
};

use crate::mem_alloc::TrMalloc;

type MemAddr = NonNull<[u8]>;

#[derive(Debug, Clone, Copy)]
pub enum AllocErrorMessage {
    NullPtrReturned,
    Unknown,
}

impl Default for AllocErrorMessage {
    fn default() -> Self {
        AllocErrorMessage::Unknown
    }
}

impl fmt::Display for AllocErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let m = match self {
            AllocErrorMessage::NullPtrReturned => "Allocator_Returns_Null_Ptr",
            AllocErrorMessage::Unknown => "Allocator_Unknown_Err",
        };
        write!(f, "{m}")
    }
}

#[derive(Debug, Clone)]
pub enum CoreAllocError {
    LayoutErr(LayoutError),
    AllocErr(AllocErrorMessage),
}

impl CoreAllocError {
    pub const fn from_alloc_err(msg: AllocErrorMessage) -> Self {
        CoreAllocError::AllocErr(msg)
    }

    pub const fn from_layout_err(err: LayoutError) -> Self {
        CoreAllocError::LayoutErr(err)
    }
}

impl From<LayoutError> for CoreAllocError {
    fn from(value: LayoutError) -> Self {
        Self::from_layout_err(value)
    }
}

impl From<AllocErrorMessage> for CoreAllocError {
    fn from(value: AllocErrorMessage) -> Self {
        Self::from_alloc_err(value)
    }
}

impl fmt::Display for CoreAllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "alloc returns null pointer.")
    }
}

impl error::Error for CoreAllocError {}

/// A wrapper for `alloc::alloc` and `alloc::dealloc`
#[derive(Debug, Default, Clone, Copy)]
pub struct CoreAlloc;

impl CoreAlloc {
    pub fn shared() -> &'static Self {
        static CORE_ALLOC: CoreAlloc = CoreAlloc;
        &CORE_ALLOC
    }

    pub const fn new() -> Self {
        CoreAlloc
    }

    pub fn can_support(&self, layout: Layout) -> bool {
        let _ = layout;
        true
    }

    pub fn allocate(&self, layout: Layout) -> Result<MemAddr, CoreAllocError> {
        unsafe {
            let Option::Some(p) = NonNull::new(alloc(layout)) else {
                return Result::Err(AllocErrorMessage::NullPtrReturned.into())
            };
            #[cfg(test)]
            log::trace!(
                "[CoreAlloc::allocate]({}, {}) returns {:?}",
                layout.size(),
                layout.align(),
                p.as_ptr()
            );
            let slice = ptr::slice_from_raw_parts_mut(
                p.as_ptr(),
                layout.size(),
            );
            Result::Ok(NonNull::new_unchecked(slice))
        }
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory [*currently allocated*] via this 
    ///   allocator, and
    /// * `layout` must [*fit*] that block of memory.
    ///
    /// [*currently allocated*]: #currently-allocated-memory
    /// [*fit*]: #memory-fitting
    pub unsafe fn deallocate(
        &self,
        ptr: MemAddr,
        layout: Layout,
    ) -> Result<usize, CoreAllocError> {
        #[cfg(test)]
        log::trace!(
            "[CoreAlloc::deallocate]({:?}) len: {}, layout: ({}, {})",
            ptr.as_ptr(),
            unsafe { ptr.as_ref().len() },
            layout.size(),
            layout.align()
        );
        unsafe { dealloc(ptr.as_ptr() as *mut _, layout); }
        Result::Ok(layout.size())
    }
}

unsafe impl TrMalloc for CoreAlloc {
    type Err = CoreAllocError;

    #[inline(always)]
    fn can_support(&self, layout: Layout) -> bool {
        CoreAlloc::can_support(self, layout)
    }

    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<MemAddr, Self::Err> {
        CoreAlloc::allocate(self, layout)
    }

    #[inline(always)]
    unsafe fn deallocate(
        &self,
        ptr: MemAddr,
        layout: Layout,
    ) -> Result<usize, Self::Err> {
        unsafe { CoreAlloc::deallocate(self, ptr, layout) }
    }
}
