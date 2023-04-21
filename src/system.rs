use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::ptr::NonNull;

#[cfg(windows)]
const IMPL: crate::winapi::WinApi = crate::winapi::WinApi;

#[cfg(not(windows))]
const IMPL: crate::posix::Posix = crate::posix::Posix;

#[derive(Debug, Copy, Clone)]
pub struct System;

impl const ConstDefault for System {
    fn default_const() -> Self { System }
}

unsafe impl NonUnwinding for System { }

unsafe impl Allocator for System {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        IMPL.allocate(layout)
    }

    fn allocate_zeroed(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        IMPL.allocate_zeroed(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        IMPL.deallocate(ptr, layout)
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>,
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        IMPL.grow(ptr, old_layout, new_layout)
    }

    unsafe fn grow_zeroed(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        IMPL.grow_zeroed(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        IMPL.shrink(ptr, old_layout, new_layout)
    }
}
