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
}

unsafe impl<A: Composable> Composable for Freelist<A> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        self.base.has_allocated(ptr, layout)
    }
}

unsafe impl<A: Allocator> Allocator for Freelist<A> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if layout != self.layout {
            return self.base.allocate(layout);
        }
        if let Some(list) = self.list.get() {
            let next = unsafe { ptr::read(list.as_ptr() as *const Node) }.next;
            self.list.set(next);
            Ok(list)
        } else {
            self.base.allocate(layout)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        if layout != self.layout {
            return self.base.deallocate(ptr, layout);
        }
        ptr::write(ptr.as_ptr() as *mut Node, Node { next: self.list.get() });
        self.list.set(Some(NonNull::slice_from_raw_parts(ptr, layout.size())));
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.base.grow(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.base.shrink(ptr, old_layout, new_layout)
    }
}
