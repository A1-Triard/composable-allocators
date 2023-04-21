use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::mem::{align_of, size_of};
use core::ptr::{self, NonNull};
use winapi::shared::minwindef::{BOOL, DWORD};
use winapi::shared::ntdef::MEMORY_ALLOCATION_ALIGNMENT;
use winapi::um::heapapi::{GetProcessHeap, HeapAlloc, HeapFree, HeapReAlloc};
use winapi::um::winnt::HEAP_ZERO_MEMORY;

#[derive(Debug, Copy, Clone)]
pub struct WinApi;

impl Default for WinApi {
    fn default() -> Self { WinApi }
}

fn non_zero(r: BOOL) -> Result<BOOL, AllocError> {
    if r == 0 {
        Err(AllocError)
    } else {
        Ok(r)
    }
}

fn is_native_align(align: usize) -> bool {
    align <= MEMORY_ALLOCATION_ALIGNMENT
}

unsafe fn allocate(layout: alloc::Layout, flags: DWORD) -> Result<NonNull<[u8]>, AllocError> {
    if MEMORY_ALLOCATION_ALIGNMENT < size_of::<usize>() || align_of::<usize>() > size_of::<usize>() {
        return Err(AllocError);
    }
    let heap = NonNull::new(GetProcessHeap()).ok_or(AllocError)?;
    let (ptr, size) = if is_native_align(layout.align()) {
        (
            NonNull::new(HeapAlloc(heap.as_ptr(), flags, layout.size()) as *mut u8).ok_or(AllocError)?,
            layout.size()
        )
    } else {
        let mut size = layout.size().checked_add(layout.align()).ok_or(AllocError)?;
        let mut ptr = NonNull::new(HeapAlloc(heap.as_ptr(), flags, size) as *mut u8).ok_or(AllocError)?;
        ptr = NonNull::new_unchecked(ptr.as_ptr().add(size_of::<usize>()));
        size -= size_of::<usize>();
        let offset = (layout.align() - (ptr.as_ptr() as usize) % layout.align()) % layout.align();
        ptr = NonNull::new_unchecked(ptr.as_ptr().add(offset));
        size -= offset;
        ptr::write(ptr.as_ptr().offset(-(size_of::<usize>() as isize)) as *mut usize, offset);
        (ptr, size)
    };
    Ok(NonNull::slice_from_raw_parts(ptr, size))
}

unsafe fn deallocate(ptr: NonNull<u8>, layout: alloc::Layout) -> Result<(), AllocError> {
    let ptr = if !is_native_align(layout.align()) {
        let ptr = ptr.as_ptr().offset(-(size_of::<usize>() as isize));
        let offset = ptr::read(ptr as *mut usize);
        ptr.offset(-(offset as isize))
    } else {
        ptr.as_ptr()
    };
    let heap = NonNull::new(GetProcessHeap()).ok_or(AllocError)?;
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
        let heap = NonNull::new(GetProcessHeap()).ok_or(AllocError)?;
        let ptr = NonNull::new(HeapReAlloc(heap.as_ptr(), flags, ptr.as_ptr() as _, new_layout.size()) as *mut u8)
            .ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
    } else {
        let new = allocate(new_layout, flags)?;
        ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_mut_ptr(), min_size);
        let _ = deallocate(ptr, old_layout);
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

#[cfg(test)]
mod test {
    use crate::winapi::WinApi;
    use core::alloc::{self, Allocator};

    #[test]
    fn allocate_zero_size() {
        let layout = alloc::Layout::from_size_align(0, 1).unwrap();
        let p = WinApi.allocate(layout).unwrap();
        unsafe { WinApi.deallocate(p.as_non_null_ptr(), layout); }
    }
}
