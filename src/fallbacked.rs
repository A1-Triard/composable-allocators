use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::ptr::{self, NonNull};

pub struct Fallbacked<A: Allocator, Fallback: Allocator>(pub A, pub Fallback);

unsafe impl<A: Fallbackable, Fallback: Fallbackable> Fallbackable for Fallbacked<A, Fallback> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        self.0.has_allocated(ptr, layout) || self.1.has_allocated(ptr, layout)
    }

    fn allows_fallback(&self, layout: alloc::Layout) -> bool {
        self.0.allows_fallback(layout) && self.1.allows_fallback(layout)
    }
}

unsafe impl<A: Fallbackable, Fallback: Allocator> Allocator for Fallbacked<A, Fallback> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if let Ok(block) = self.0.allocate(layout) {
            Ok(block)
        } else if self.0.allows_fallback(layout) {
            self.1.allocate(layout)
        } else {
            Err(AllocError)
        }
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
            } else if self.0.allows_fallback(new_layout) {
                if let Ok(block) = self.1.allocate(new_layout) {
                    ptr::copy_nonoverlapping(ptr.as_ptr(), block.as_mut_ptr(), old_layout.size());
                    self.0.deallocate(ptr, old_layout);
                    Ok(block)
                } else {
                    Err(AllocError)
                }
            } else {
                Err(AllocError)
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
            } else if self.0.allows_fallback(new_layout) {
                if let Ok(block) = self.1.allocate(new_layout) {
                    ptr::copy_nonoverlapping(ptr.as_ptr(), block.as_mut_ptr(), new_layout.size());
                    self.0.deallocate(ptr, old_layout);
                    Ok(block)
                } else {
                    Err(AllocError)
                }
            } else {
                Err(AllocError)
            }
        } else {
            self.1.shrink(ptr, old_layout, new_layout)
        }
    }
}
