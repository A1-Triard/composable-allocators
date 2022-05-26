use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::ptr::NonNull;

#[derive(Debug, Copy, Clone)]
pub struct NonWorking;

impl const Default for NonWorking {
    fn default() -> Self { NonWorking }
}

unsafe impl Composable for NonWorking {
    unsafe fn has_allocated(&self, _ptr: NonNull<u8>, _layout: alloc::Layout) -> bool {
        false
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
        unreachable!()
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
