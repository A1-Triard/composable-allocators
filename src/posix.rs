use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::mem::size_of;
use core::ptr::{self, NonNull, null_mut};
use libc::{c_int, free, malloc, posix_memalign, realloc};

#[derive(Debug, Copy, Clone)]
pub struct Posix;

impl Default for Posix {
    fn default() -> Self { Posix }
}

fn zero(r: c_int) -> Result<(), AllocError> {
    if r == 0 {
        Ok(())
    } else {
        Err(AllocError)
    }
}

fn is_native_align(align: usize) -> bool {
    align <= 2 * size_of::<usize>()
}

unsafe impl NonUnwinding for Posix { }

unsafe impl Allocator for Posix {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = if layout.size() == 0 {
            unsafe { NonNull::new_unchecked(ptr::invalid_mut(layout.align())) }
        } else if !is_native_align(layout.align()) {
            let mut ptr = null_mut();
            zero(unsafe { posix_memalign(&raw mut ptr, layout.align(), layout.size()) })?;
            unsafe { NonNull::new_unchecked(ptr as *mut u8) }
        } else {
            let ptr = NonNull::new(unsafe { malloc(layout.size()) } as *mut u8).ok_or(AllocError)?;
            if ptr.as_ptr() as usize % layout.align() != 0 { return Err(AllocError); }
            ptr
        };
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    fn allocate_zeroed(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.allocate(layout)?;
        unsafe { ptr.as_mut_ptr().write_bytes(0, ptr.len()); }
        Ok(ptr)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        if layout.size() != 0 {
            free(ptr.as_ptr() as _);
        }
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: alloc::Layout,
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        if old_layout.size() != 0 && is_native_align(old_layout.align()) && is_native_align(new_layout.align()) {
            let ptr = NonNull::new(realloc(ptr.as_ptr() as _, new_layout.size()) as *mut u8).ok_or(AllocError)?;
            if ptr.as_ptr() as usize % new_layout.align() != 0 { return Err(AllocError); }
            Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
        } else {
            let new = self.allocate(new_layout)?;
            ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_mut_ptr(), old_layout.size());
            self.deallocate(ptr, old_layout);
            Ok(new)
        }
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: alloc::Layout,
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.grow(ptr, old_layout, new_layout)?;
        ptr.as_mut_ptr().map_addr(|x| x + old_layout.size()).write_bytes(0, ptr.len() - old_layout.size());
        Ok(ptr)
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: alloc::Layout,
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        if new_layout.size() != 0 && is_native_align(old_layout.align()) && is_native_align(new_layout.align()) {
            let ptr = NonNull::new(realloc(ptr.as_ptr() as _, new_layout.size()) as *mut u8).ok_or(AllocError)?;
            if ptr.as_ptr() as usize % new_layout.align() != 0 { return Err(AllocError); }
            Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
        } else {
            let new = self.allocate(new_layout)?;
            ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_mut_ptr(), new_layout.size());
            self.deallocate(ptr, old_layout);
            Ok(new)
        }
    }
}
