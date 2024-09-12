use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::cmp::min;
use core::ptr::NonNull;

pub struct LimitedUpTo<A: Allocator> {
    layout: alloc::Layout,
    base: A,
}

unsafe impl<A: NonUnwinding> NonUnwinding for LimitedUpTo<A> { }

impl<A: Allocator> LimitedUpTo<A> {
    pub const fn new(layout: alloc::Layout, base: A) -> Self {
        LimitedUpTo { layout, base }
    }

    fn manages(&self, layout: alloc::Layout) -> bool {
        layout.size() <= self.layout.size() &&
        layout.align() <= self.layout.align()
    }
}

unsafe impl<A: Allocator> Fallbackable for LimitedUpTo<A> {
    unsafe fn has_allocated(&self, _ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        self.manages(layout)
    }

    fn allows_fallback(&self, layout: alloc::Layout) -> bool {
        !self.manages(layout)
    }
}

unsafe impl<A: Allocator> Allocator for LimitedUpTo<A> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if self.manages(layout) {
            if let Ok(block) = self.base.allocate(layout) {
                let len = min(block.len(), self.layout.size());
                Ok(unsafe { NonNull::slice_from_raw_parts(NonNull::new_unchecked(block.as_mut_ptr()), len) })
            } else {
                Err(AllocError)
            }
        } else {
            Err(AllocError)
        }
    }

    fn allocate_zeroed(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if self.manages(layout) {
            if let Ok(block) = self.base.allocate_zeroed(layout) {
                let len = min(block.len(), self.layout.size());
                Ok(unsafe { NonNull::slice_from_raw_parts(NonNull::new_unchecked(block.as_mut_ptr()), len) })
            } else {
                Err(AllocError)
            }
        } else {
            Err(AllocError)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        self.base.deallocate(ptr, layout);
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self.manages(new_layout) {
            if let Ok(block) = self.base.grow(ptr, old_layout, new_layout) {
                let len = min(block.len(), self.layout.size());
                Ok(NonNull::slice_from_raw_parts(NonNull::new_unchecked(block.as_mut_ptr()), len))
            } else {
                Err(AllocError)
            }
        } else {
            Err(AllocError)
        }
    }

    unsafe fn grow_zeroed(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self.manages(new_layout) {
            if let Ok(block) = self.base.grow_zeroed(ptr, old_layout, new_layout) {
                let len = min(block.len(), self.layout.size());
                Ok(NonNull::slice_from_raw_parts(NonNull::new_unchecked(block.as_mut_ptr()), len))
            } else {
                Err(AllocError)
            }
        } else {
            Err(AllocError)
        }
    }

    unsafe fn shrink(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self.manages(new_layout) {
            if let Ok(block) = self.base.shrink(ptr, old_layout, new_layout) {
                let len = min(block.len(), self.layout.size());
                Ok(NonNull::slice_from_raw_parts(NonNull::new_unchecked(block.as_mut_ptr()), len))
            } else {
                Err(AllocError)
            }
        } else {
            Err(AllocError)
        }
    }
}
