use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::ptr::NonNull;
use print_no_std::Stderr;

pub struct Logging<A: Allocator>(pub A);

unsafe impl<A: NonUnwinding> NonUnwinding for Logging<A> { }

unsafe impl<A: Fallbackable> Fallbackable for Logging<A> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        self.0.has_allocated(ptr, layout)
    }

    fn allows_fallback(&self, layout: alloc::Layout) -> bool {
        self.0.allows_fallback(layout)
    }
}

unsafe impl<A: Allocator> Allocator for Logging<A> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let _ = writeln!(Stderr { panic: false }, "allocate: {layout:?}");
        self.0.allocate(layout)
    }

    fn allocate_zeroed(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let _ = writeln!(Stderr { panic: false }, "allocate_zeroed: {layout:?}");
        self.0.allocate_zeroed(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        let _ = writeln!(Stderr { panic: false }, "deallocate: {layout:?}");
        self.0.deallocate(ptr, layout)
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let _ = writeln!(Stderr { panic: false }, "grow: {old_layout:?} -> {new_layout:?}");
        self.0.grow(ptr, old_layout, new_layout)
    }

    unsafe fn grow_zeroed(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let _ = writeln!(Stderr { panic: false }, "grow_zeroed: {old_layout:?} -> {new_layout:?}");
        self.0.grow_zeroed(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let _ = writeln!(Stderr { panic: false }, "shrink: {old_layout:?} -> {new_layout:?}");
        self.0.shrink(ptr, old_layout, new_layout)
    }
}
