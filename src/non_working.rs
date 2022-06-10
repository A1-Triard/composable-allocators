use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::hint::unreachable_unchecked;
use core::ptr::NonNull;

#[derive(Debug, Copy, Clone)]
pub struct NonWorking;

unsafe impl NonUnwinding for NonWorking { }

impl const Default for NonWorking {
    fn default() -> Self { NonWorking }
}

unsafe impl Fallbackable for NonWorking {
    unsafe fn has_allocated(&self, _ptr: NonNull<u8>, _layout: alloc::Layout) -> bool {
        false
    }

    fn allows_fallback(&self, _layout: alloc::Layout) -> bool {
        true
    }
}

unsafe impl Allocator for NonWorking {
    fn allocate(&self, _layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    fn allocate_zeroed(&self, _layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: alloc::Layout) {
        unreachable_unchecked()
    }

    unsafe fn grow(
        &self, 
        _ptr: NonNull<u8>, 
        _old_layout: alloc::Layout, 
        _new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    unsafe fn grow_zeroed(
        &self, 
        _ptr: NonNull<u8>, 
        _old_layout: alloc::Layout, 
        _new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    unsafe fn shrink(
        &self, 
        _ptr: NonNull<u8>, 
        _old_layout: alloc::Layout, 
        _new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }
}
