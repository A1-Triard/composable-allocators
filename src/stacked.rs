use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
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
    pub fn with_size<const SIZE: usize, T>(f: impl for<'a> FnOnce(&'a Stacked) -> T) -> T {
        let mut buf: [MaybeUninit<u8>; SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        Self::with_buf(&mut buf, f)
    }

    pub fn with_buf<T>(buf: &mut [MaybeUninit<u8>], f: impl for<'a> FnOnce(&'a Stacked) -> T) -> T {
        let buf_len = buf.len();
        assert!(buf_len <= isize::MAX as usize && (isize::MAX as usize) - buf_len >= buf.as_ptr() as usize);
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
            offset < self.buf_len && self.buf_ptr.as_ptr().add(offset) == ptr.as_ptr()
        } else {
            false
        }
    }
}

unsafe impl Allocator for Stacked {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = unsafe { self.buf_ptr.as_ptr().add(self.allocated.get()) };
        let padding = (layout.align() - (ptr as usize) % layout.align()) % layout.align();
        let size = padding.checked_add(layout.size()).ok_or(AllocError)?;
        if size > self.buf_len - self.allocated.get() { return Err(AllocError); }
        self.allocations_count.set(self.allocations_count.get().checked_add(1).ok_or(AllocError)?);
        let res = NonNull::slice_from_raw_parts(
            unsafe { NonNull::new_unchecked(ptr.add(padding)) },
            layout.size()
        );
        self.allocated.set(self.allocated.get() + size);
        Ok(res)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        if
            ptr.as_ptr().add(layout.size()) ==
            self.buf_ptr.as_ptr().add(self.allocated.get())
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
            ptr.as_ptr().add(old_layout.size()) ==
            self.buf_ptr.as_ptr().add(self.allocated.get())
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
            ptr.as_ptr().add(old_layout.size()) ==
            self.buf_ptr.as_ptr().add(self.allocated.get())
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
