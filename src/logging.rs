use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::ptr::NonNull;

pub struct Logging<A: Allocator> {
    base: A,
}

unsafe impl<A: NonUnwinding> NonUnwinding for Logging<A> { }

impl<A: Allocator> Logging<A> {
    pub const fn new(base: A) -> Self {
        Logging { base }
    }
}

unsafe impl<A: Fallbackable> Fallbackable for Logging<A> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        self.base.has_allocated(ptr, layout)
    }

    fn allows_fallback(&self, layout: alloc::Layout) -> bool {
        self.base.allows_fallback(layout)
    }
}

unsafe impl<A: Allocator> Allocator for Logging<A> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        eprintln!("allocate: {:?}", layout);
        self.base.allocate(layout)
    }

    fn allocate_zeroed(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        eprintln!("allocate_zeroed: {:?}", layout);
        self.base.allocate_zeroed(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        eprintln!("deallocate: {:?}", layout);
        self.base.deallocate(ptr, layout)
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        eprintln!("grow: {:?} -> {:?}", old_layout, new_layout);
        self.base.grow(ptr, old_layout, new_layout)
    }

    unsafe fn grow_zeroed(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        eprintln!("grow_zeroed: {:?} -> {:?}", old_layout, new_layout);
        self.base.grow_zeroed(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        eprintln!("shrink: {:?} -> {:?}", old_layout, new_layout);
        self.base.shrink(ptr, old_layout, new_layout)
    }
}
