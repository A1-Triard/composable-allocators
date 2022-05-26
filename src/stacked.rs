use crate::base::*;
use alloc_crate::alloc::{self, AllocError, Allocator};
use core::cell::Cell;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

pub struct Stacked {
    buf_ptr: NonNull<u8>,
    buf_len: usize,
    allocated: Cell<usize>,
    allocations_count: Cell<usize>,
}

impl Drop for Stacked {
    fn drop(&mut self) {
        assert!(self.allocations_count.get() == 0, "memory leaks in Stacked allocator");
    }
}

impl Stacked {
    pub fn with<T>(buf: &mut [MaybeUninit<u8>], f: impl for<'a> FnOnce(&'a Stacked) -> T) -> T {
        let buf_len = buf.len();
        assert!(buf_len < isize::MAX as usize);
        let stacked = Stacked {
            buf_ptr: unsafe { NonNull::new_unchecked(buf.as_mut_ptr() as *mut u8) },
            buf_len,
            allocated: Cell::new(0),
            allocations_count: Cell::new(0),
        };
        f(&stacked)
    }
}

unsafe impl Composable for Stacked {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, _layout: alloc::Layout) -> bool {
        if let Some(offset) = (ptr.as_ptr() as usize).checked_sub(self.buf_ptr.as_ptr() as usize) {
            offset < self.buf_len && self.buf_ptr.as_ptr().offset(offset as isize) == ptr.as_ptr()
        } else {
            false
        }
    }
}

unsafe impl Allocator for Stacked {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = unsafe { self.buf_ptr.as_ptr().offset(self.allocated.get() as isize) };
        let padding = (layout.align() - (ptr as usize) % layout.align()) % layout.align();
        let size = padding.checked_add(layout.size()).ok_or(AllocError)?;
        if size > self.buf_len - self.allocated.get() { return Err(AllocError); }
        self.allocations_count.set(self.allocations_count.get().checked_add(1).ok_or(AllocError)?);
        let res = NonNull::slice_from_raw_parts(
            unsafe { NonNull::new_unchecked(ptr.offset(padding as isize)) },
            layout.size()
        );
        self.allocated.set(self.allocated.get() + size);
        Ok(res)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        if
            ptr.as_ptr().offset(layout.size() as isize) ==
            self.buf_ptr.as_ptr().offset(self.allocated.get() as isize)
        {
            self.allocated.set(self.allocated.get() - layout.size());
        }
        self.allocations_count.set(self.allocations_count.get() - 1);
    }

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
}
