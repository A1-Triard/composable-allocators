use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::cmp::min;
use core::ptr::NonNull;

/// # Safety
///
/// The ['layout`](Params::layout) method should return constant,
/// i.e. same value on every call.
pub unsafe trait Params {
    fn layout(&self) -> alloc::Layout;
}

pub struct CtParams<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
>(());

const fn is_power_of_two(x: usize) -> bool {
    x != 0 && (x & (x - 1)) == 0
}

impl<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
> CtParams<LAYOUT_SIZE, LAYOUT_ALIGN> {
    pub const fn new() -> Self {
        assert!(LAYOUT_SIZE <= isize::MAX as usize);
        assert!(LAYOUT_ALIGN <= isize::MAX as usize);
        assert!(is_power_of_two(LAYOUT_ALIGN));
        assert!(((LAYOUT_SIZE + LAYOUT_ALIGN - 1) / LAYOUT_ALIGN) * LAYOUT_ALIGN <= isize::MAX as usize);
        CtParams(())
    }
}

impl<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
> const Default for CtParams<LAYOUT_SIZE, LAYOUT_ALIGN> {
    fn default() -> Self { CtParams::new() }
}

unsafe impl<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
> const Params for CtParams<LAYOUT_SIZE, LAYOUT_ALIGN> {
    fn layout(&self) -> alloc::Layout {
        unsafe { alloc::Layout::from_size_align_unchecked(LAYOUT_SIZE, LAYOUT_ALIGN) }
    }
}

pub struct RtParams {
    pub layout: alloc::Layout,
}

unsafe impl Params for RtParams {
    fn layout(&self) -> alloc::Layout { self.layout }
}

pub struct UpTo<P: Params, A: Allocator> {
    params: P,
    base: A,
}

impl<P: Params, A: Allocator> UpTo<P, A> {
    pub fn new(params: P, base: A) -> Self {
        UpTo { params, base }
    }
}

unsafe impl<P: Params, A: Allocator> Composable for UpTo<P, A> {
    unsafe fn has_allocated(&self, _ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        layout.size() <= self.params.layout().size() &&
        layout.align() <= self.params.layout().align()
    }

    fn manages_on_its_own(&self, layout: alloc::Layout) -> bool {
        layout.size() <= self.params.layout().size() &&
        layout.align() <= self.params.layout().align()
    }
}

unsafe impl<P: Params, A: Allocator> Allocator for UpTo<P, A> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if self.manages_on_its_own(layout) {
            if let Ok(block) = self.base.allocate(layout) {
                let len = min(block.len(), self.params.layout().size());
                Ok(unsafe { NonNull::slice_from_raw_parts(NonNull::new_unchecked(block.as_mut_ptr()), len) })
            } else {
                Err(AllocError)
            }
        } else {
            Err(AllocError)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        self.base.deallocate(ptr, layout);
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self.manages_on_its_own(new_layout) {
            if let Ok(block) = self.base.grow(ptr, old_layout, new_layout) {
                let len = min(block.len(), self.params.layout().size());
                Ok(NonNull::slice_from_raw_parts(NonNull::new_unchecked(block.as_mut_ptr()), len))
            } else {
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
        if self.manages_on_its_own(new_layout) {
            if let Ok(block) = self.base.shrink(ptr, old_layout, new_layout) {
                let len = min(block.len(), self.params.layout().size());
                Ok(NonNull::slice_from_raw_parts(NonNull::new_unchecked(block.as_mut_ptr()), len))
            } else {
                Err(AllocError)
            }
        } else {
            Err(AllocError)
        }
    }
}
