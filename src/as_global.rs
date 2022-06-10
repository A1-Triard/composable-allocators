use crate::base::*;
use core::alloc::{self, GlobalAlloc};
use core::ptr::{NonNull, null_mut};

#[derive(Debug, Copy, Clone)]
pub struct AsGlobal<A: NonUnwinding + ?Sized>(pub A);

impl<A: NonUnwinding + ~const Default> const Default for AsGlobal<A> {
    fn default() -> Self { AsGlobal(A::default()) }
}

unsafe impl<A: NonUnwinding + ?Sized> GlobalAlloc for AsGlobal<A> {
    unsafe fn alloc(&self, layout: alloc::Layout) -> *mut u8 {
        self.0.allocate(layout).map_or_else(|_| null_mut(), |x| x.as_mut_ptr())
    }

    unsafe fn alloc_zeroed(&self, layout: alloc::Layout) -> *mut u8 {
        self.0.allocate_zeroed(layout).map_or_else(|_| null_mut(), |x| x.as_mut_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: alloc::Layout) {
        self.0.deallocate(NonNull::new_unchecked(ptr), layout)
    }

    unsafe fn realloc(
        &self, 
        ptr: *mut u8, 
        layout: alloc::Layout, 
        new_size: usize
    ) -> *mut u8 {
        let ptr = NonNull::new_unchecked(ptr);
        let new_layout = alloc::Layout::from_size_align_unchecked(new_size, layout.align());
        if new_size > layout.size() {
            self.0.grow(ptr, layout, new_layout)
        } else {
            self.0.shrink(ptr, layout, new_layout)
        }.map_or_else(|_| null_mut(), |x| x.as_mut_ptr())
    }
}
