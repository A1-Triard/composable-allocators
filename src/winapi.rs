use core::alloc::{self, AllocError, Allocator};
use core::mem::size_of;
use core::ptr::{self, NonNull};
use errno_no_std::{Errno, errno};
use winapi::shared::minwindef::{BOOL, DWORD};
use winapi::shared::ntdef::MEMORY_ALLOCATION_ALIGNMENT;
use winapi::um::heapapi::{GetProcessHeap, HeapAlloc, HeapFree, HeapReAlloc};
use winapi::um::winnt::HEAP_ZERO_MEMORY;

#[derive(Debug, Copy, Clone)]
pub struct WinApi;

impl const Default for WinApi {
    fn default() -> Self { WinApi }
}

fn non_zero(r: BOOL) -> Result<BOOL, Errno> {
    if r == 0 {
        Err(errno())
    } else {
        Ok(r)
    }
}

fn non_null<T: ?Sized>(p: *mut T) -> Result<NonNull<T>, Errno> {
    NonNull::new(p).ok_or_else(errno)
}

fn is_native_align(align: usize) -> bool {
    align <= MEMORY_ALLOCATION_ALIGNMENT
}

unsafe fn allocate(layout: alloc::Layout, flags: DWORD) -> Result<NonNull<[u8]>, AllocError> {
    assert!(MEMORY_ALLOCATION_ALIGNMENT >= size_of::<usize>());
    let heap = non_null(GetProcessHeap()).map_err(|_| AllocError);
    let align = if !is_native_align(layout.align()) { layout.align() } else { 0 };
    let mut size = layout.size().checked_add(align).ok_or(AllocError)?;
    let p = non_null(HeapAlloc(heap.as_ptr(), flags, size)).map_err(|_| AllocError)?;
    let p = if align != 0 {
        let mut p = p.as_ptr().add(MEMORY_ALLOCATION_ALIGNMENT);
        size -= MEMORY_ALLOCATION_ALIGNMENT;
        let offset = (layout.align() - (p as usize) % layout.align()) % layout.align();
        p = p.add(offset);
        size -= offset;
        ptr::write(p.offset(-(MEMORY_ALLOCATION_ALIGNMENT as isize)) as *mut usize, offset);
        p
    } else {
        p.as_ptr()
    };
    Ok(NonNull::slice_from_raw_parts(NonNull::new_unchecked(p as *mut u8), size))
}

unsafe fn deallocate(ptr: NonNull<u8>, layout: alloc::Layout) -> Result<(), Errno> {
    let ptr = if !is_native_align(layout.align()) {
        let ptr = ptr.as_ptr().offset(-(MEMORY_ALLOCATION_ALIGNMENT as isize));
        let offset = ptr::read(ptr as *mut usize);
        ptr.offset(-(offset as isize))
    } else {
        ptr.as_ptr()
    };
    let heap = non_null(GetProcessHeap())?;
    non_zero(HeapFree(heap.as_ptr(), 0, ptr as _))?;
    Ok(())
}

unsafe fn realloc(
    ptr: NonNull<u8>,
    old_layout: alloc::Layout, 
    new_layout: alloc::Layout,
    min_size: usize,
    flags: DWORD
) -> Result<NonNull<[u8]>, AllocError> {
    if is_native_align(old_layout.align()) && is_native_align(new_layout.align()) {
        let heap = non_null(GetProcessHeap()).map_err(|_| AllocError);
        let ptr = non_null(HeapReAlloc(heap.as_ptr(), flags, ptr.as_ptr() as _, new_layout.size()))
            .map_err(|_| AllocError)?;
        let ptr = NonNull::new_unchecked(ptr.as_ptr() as *mut u8);
        Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
    } else {
        let new = allocate(new_layout, flags)?;
        ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_mut_ptr(), min_size);
        deallocate(ptr, old_layout);
        Ok(new)
    }
}

unsafe impl NonUnwinding for WinApi { }

unsafe impl Allocator for WinApi {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { allocate(layout, 0) }
    }

    fn allocate_zeroed(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { allocate(layout, HEAP_ZERO_MEMORY) }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        let _ = deallocate(ptr, layout);
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>,
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        realloc(ptr, old_layout, new_layout, old_layout.size(), 0)
    }

    unsafe fn grow_zeroed(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        realloc(ptr, old_layout, new_layout, old_layout.size(), HEAP_ZERO_MEMORY)
    }

    unsafe fn shrink(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        realloc(ptr, old_layout, new_layout, new_layout.size(), 0)
    }
}
