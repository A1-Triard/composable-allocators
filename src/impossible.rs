use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::ptr::NonNull;

#[derive(Copy, Clone)]
pub struct Impossible(pub !);

unsafe impl Fallbackable for Impossible {
    unsafe fn has_allocated(&self, _ptr: NonNull<u8>, _layout: alloc::Layout) -> bool {
        self.0
    }

    fn allows_fallback(&self, _layout: alloc::Layout) -> bool {
        self.0
    }
}

unsafe impl Allocator for Impossible {
    fn allocate(&self, _layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.0
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: alloc::Layout) {
        self.0
    }

    unsafe fn grow(
        &self, 
        _ptr: NonNull<u8>, 
        _old_layout: alloc::Layout, 
        _new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.0
    }

    unsafe fn shrink(
        &self, 
        _ptr: NonNull<u8>, 
        _old_layout: alloc::Layout, 
        _new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.0
    }
}
