use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::cmp::min;
use core::ptr::NonNull;

pub unsafe trait Params {
    fn layout(&self) -> alloc::Layout;
}

pub struct UpTo<P: Params, A: Allocator> {
    params: P,
    base: A,
}

unsafe impl<P: Params, A: Allocator> Composable for UpTo<P, A> {
    unsafe fn has_allocated(&self, _ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        layout.size() <= self.params.layout().size() &&
        layout.align() <= self.params.layout().align()
    }

    fn manages_on_its_own(&self, layout: alloc::Layout) -> bool {
        layout.size() <= self.params.layout().size() &&
        layout.align() <= self.params.layout().align()
    }
}

unsafe impl<P: Params, A: Allocator> Allocator for UpTo<P, A> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if self.manages_on_its_own(layout) {
            if let Ok(block) = self.base.allocate(layout) {
                let len = min(block.len(), self.params.layout().size());
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
        if self.manages_on_its_own(new_layout) {
            if let Ok(block) = self.base.grow(ptr, old_layout, new_layout) {
                let len = min(block.len(), self.params.layout().size());
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
        if self.manages_on_its_own(new_layout) {
            if let Ok(block) = self.base.shrink(ptr, old_layout, new_layout) {
                let len = min(block.len(), self.params.layout().size());
                Ok(NonNull::slice_from_raw_parts(NonNull::new_unchecked(block.as_mut_ptr()), len))
            } else {
                Err(AllocError)
            }
        } else {
            Err(AllocError)
        }
    }
}
