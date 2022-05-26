use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::cell::Cell;
use core::ptr::{self, NonNull};

#[derive(Clone, Copy)]
struct Node {
    next: Option<NonNull<[u8]>>,
}

pub struct Freelist<A: Allocator> {
    base: A,
    list: Cell<Option<NonNull<[u8]>>>,
    layout: alloc::Layout,
    low_bound: alloc::Layout,
}

impl<A: Composable> Freelist<A> {
    /// # Safety
    ///
    /// Arguments should satisfy
    /// `low_bound.size() <= layout.size() && low_bound.align() <= layout.align()`.
    pub unsafe fn new_unchecked(base: A, layout: alloc::Layout, low_bound: alloc::Layout) -> Self {
        Freelist { base, list: Cell::new(None), layout, low_bound }
    }

    pub fn new(base: A, layout: alloc::Layout, low_bound: alloc::Layout) -> Self {
        let low_bound = unsafe { alloc::Layout::from_size_align_unchecked(
            min(low_bound.size(), layout.size()),
            min(low_bound.align(), layout.align()),
        ) };
        Freelist { base, list: Cell::new(None), layout, low_bound }
    }

    fn manage(&self, layout: alloc::Layout) -> bool {
        layout.size() in self.low_bound.size() ..= self.layout.size() &&
        layout.align() in self.low_bound.align() ..= self.layout.size()
    }
}

unsafe impl<A: Composable> Composable for Freelist<A> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        self.base.has_allocated(ptr, layout)
    }
}

unsafe impl<A: Allocator> Allocator for Freelist<A> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if !self.manage(layout) {
            return self.base.allocate(layout);
        }
        if let Some(list) = self.list.get() {
            let next = unsafe { ptr::read(list.as_ptr() as *const Node) }.next;
            self.list.set(next);
            Ok(list)
        } else {
            self.base.allocate(self.layout)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        if !self.manage(layout) {
            return self.base.deallocate(ptr, layout);
        }
        ptr::write(ptr.as_ptr() as *mut Node, Node { next: self.list.get() });
        self.list.set(Some(NonNull::slice_from_raw_parts(ptr, self.layout.size())));
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_layout = if self.manage(old_layout) { self.layout } else { old_layout };
        self.base.grow(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_layout = if self.manage(old_layout) {
            if self.manage(new_layout) {
                return Ok(NonNull::slice_from_raw_parts(ptr, self.layout.size()));
            }
            self.layout
        } else {
            old_layout
        };
        self.base.shrink(ptr, old_layout, new_layout)
    }
}
