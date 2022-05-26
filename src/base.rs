use core::alloc::{self, AllocError, Allocator};
use core::ptr::{self, NonNull};

pub unsafe trait Composable: Allocator {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool;
}

unsafe impl<'a, T: Composable> Composable for &'a T {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        (*self).has_allocated(ptr, layout)
    }
}

pub struct Or<Primary: Allocator, Fallback: Allocator>(pub Primary, pub Fallback);

unsafe impl<Primary: Composable, Fallback: Composable> Composable for Or<Primary, Fallback> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        self.0.has_allocated(ptr, layout) || self.1.has_allocated(ptr, layout)
    }
}

unsafe impl<Primary: Composable, Fallback: Allocator> Allocator for Or<Primary, Fallback> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.0.allocate(layout).or_else(|AllocError| self.1.allocate(layout))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        if self.0.has_allocated(ptr, layout) {
            self.0.deallocate(ptr, layout);
        } else {
            self.1.deallocate(ptr, layout);
        }
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self.0.has_allocated(ptr, old_layout) {
            if let Ok(block) = self.0.grow(ptr, old_layout, new_layout) {
                Ok(block)
            } else {
                if let Ok(block) = self.1.allocate(new_layout) {
                    ptr::copy_nonoverlapping(ptr.as_ptr(), block.as_mut_ptr(), old_layout.size());
                    self.0.deallocate(ptr, old_layout);
                    Ok(block)
                } else {
                    Err(AllocError)
                }
            }
        } else {
            self.1.grow(ptr, old_layout, new_layout)
        }
    }

    unsafe fn shrink(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        if self.0.has_allocated(ptr, old_layout) {
            if let Ok(block) = self.0.shrink(ptr, old_layout, new_layout) {
                Ok(block)
            } else {
                if let Ok(block) = self.1.allocate(new_layout) {
                    ptr::copy_nonoverlapping(ptr.as_ptr(), block.as_mut_ptr(), new_layout.size());
                    self.0.deallocate(ptr, old_layout);
                    Ok(block)
                } else {
                    Err(AllocError)
                }
            }
        } else {
            self.1.shrink(ptr, old_layout, new_layout)
        }
    }
}
