//use crate::base::*;
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

/*
unsafe impl Composable for Freelist {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, _layout: alloc::Layout) -> bool {
        if let Some(offset) = (ptr.as_ptr() as usize).checked_sub(self.buf_ptr.as_ptr() as usize) {
            offset < self.buf_len && self.buf_ptr.as_ptr().offset(offset as isize) == ptr.as_ptr()
        } else {
            false
        }
    }
}
*/

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

    /*
    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        if
            ptr.as_ptr().offset(old_layout.size() as isize) ==
            self.buf_ptr.as_ptr().offset(self.allocated.get() as isize)
        {
            self.allocated.set(self.allocated.get() - old_layout.size());
            if let Ok(block) = self.allocate(new_layout) {
                Ok(block)
            } else {
                self.allocated.set(self.allocated.get() + old_layout.size());
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
        if
            ptr.as_ptr().offset(old_layout.size() as isize) ==
            self.buf_ptr.as_ptr().offset(self.allocated.get() as isize)
        {
            self.allocated.set(self.allocated.get() - old_layout.size());
            if let Ok(block) = self.allocate(new_layout) {
                Ok(block)
            } else {
                self.allocated.set(self.allocated.get() + old_layout.size());
                Err(AllocError)
            }
        } else {
            if new_layout.align() > old_layout.align() {
                Err(AllocError)
            } else {
                Ok(NonNull::slice_from_raw_parts(ptr, old_layout.size()))
            }
        }
    }
    */
}
