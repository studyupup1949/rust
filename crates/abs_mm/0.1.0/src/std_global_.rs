use std::{
    alloc::{alloc, dealloc, Layout},
    ptr::{self, NonNull},
};

use crate::mem_alloc::{MemAddr, TrMalloc};

pub struct StdGlobalAlloc;

impl StdGlobalAlloc {
    #[cfg(feature = "support-std")]
    pub fn shared() -> &'static StdGlobalAlloc {
        static GLOBAL_ALLOC: StdGlobalAlloc = StdGlobalAlloc;
        &GLOBAL_ALLOC
    }

    /// It always returns true.
    pub fn can_support(&self, layout: Layout) -> bool {
        let _ = layout;
        true
    }

    pub fn allocate(
        &self,
        layout: Layout,
    ) -> Result<MemAddr, StdGlobalAllocError> {
        unsafe {
            if let Option::Some(p) = NonNull::new(alloc(layout)) {
                #[cfg(test)]
                log::trace!(
                    "[StdGlobalAlloc::allocate]({}, {}) returns {:?}",
                    layout.size(),
                    layout.align(),
                    p.as_ptr()
                );
                let slice = ptr::slice_from_raw_parts_mut(
                    p.as_ptr(),
                    layout.size(),
                );
                Result::Ok(NonNull::new_unchecked(slice))
            } else {
                Result::Err(StdGlobalAllocError)
            }
        }
    }

    pub unsafe fn deallocate(
        &self,
        ptr: MemAddr,
        layout: Layout,
    ) -> Result<usize, StdGlobalAllocError> {
        #[cfg(test)]
        log::trace!(
            "[StdGlobalAlloc::deallocate]({:?}) len: {}, layout: ({}, {})",
            ptr.as_ptr(),
            ptr.as_ref().len(),
            layout.size(),
            layout.align()
        );
        dealloc(ptr.as_ptr() as *mut _, layout);
        Result::Ok(layout.size())
    }
}

pub struct StdGlobalAllocError;

unsafe impl TrMalloc for StdGlobalAlloc {
    type Err = StdGlobalAllocError;

    #[inline(always)]
    fn can_support(&self, layout: Layout) -> bool {
        StdGlobalAlloc::can_support(self, layout)
    }

    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<MemAddr, StdGlobalAllocError> {
        StdGlobalAlloc::allocate(self, layout)
    }

    #[inline(always)]
    unsafe fn deallocate(
        &self,
        ptr: MemAddr,
        layout: Layout,
    ) -> Result<usize, StdGlobalAllocError> {
        StdGlobalAlloc::deallocate(self, ptr, layout)
    }
}
