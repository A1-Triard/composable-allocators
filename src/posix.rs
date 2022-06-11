use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::cmp::max;
use core::mem::size_of;
use core::ptr::{self, NonNull, null_mut};
use libc::{c_int, free, posix_memalign};

#[derive(Debug, Copy, Clone)]
pub struct Posix;

impl const Default for Posix {
    fn default() -> Self { Posix }
}

fn zero(r: c_int) -> Result<(), AllocError> {
    if r == 0 {
        Ok(())
    } else {
        Err(AllocError)
    }
}

unsafe impl NonUnwinding for Posix { }

unsafe impl Allocator for Posix {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = if layout.size() == 0 {
            unsafe { NonNull::new_unchecked(ptr::invalid_mut(layout.align())) }
        } else {
            let align = max(size_of::<usize>(), layout.align());
            let mut ptr = null_mut();
            zero(unsafe { posix_memalign(&raw mut ptr, align, layout.size()) })?;
            unsafe { NonNull::new_unchecked(ptr as *mut u8) }
        };
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        if layout.size() == 0 { return; }
        free(ptr.as_ptr() as _);
    }
}
