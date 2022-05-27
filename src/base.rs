use core::alloc::{self, AllocError, Allocator};
use core::ptr::{self, NonNull};

/// # Safety
///
/// The [`manages_on_its_own`](Composable::manages_on_its_own) function should be pure, i.e. always return same value
/// for same `layout`.
///
/// An implementer of this trait should return `true` from
/// [`has_allocated`](Composable::has_allocated)
/// if and only if at least one of the following conditions is satisfied:
///
/// - the passed pointer is denoting to
///   [currently allocated block](https://doc.rust-lang.org/core/alloc/trait.Allocator.html#currently-allocated-memory),
///
/// - [`manages_on_its_own`](Composable::manages_on_its_own) returns true for [`Layout`](alloc::Layout) used to allocate
///   memory block, denoting by `ptr`.
pub unsafe trait Composable: Allocator {
    /// # Safety
    ///
    /// The `ptr` parameter should denote a memory block,
    /// [currently allocated](https://doc.rust-lang.org/core/alloc/trait.Allocator.html#currently-allocated-memory)
    /// by this or any other [`Allocator`].
    ///
    /// The `layout` parameter should
    /// [fit](https://doc.rust-lang.org/core/alloc/trait.Allocator.html#memory-fitting)
    /// the memory block denoting by `ptr`.
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool;

    fn manages_on_its_own(&self, layout: alloc::Layout) -> bool;
}

unsafe impl<'a, T: Composable> Composable for &'a T {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        (*self).has_allocated(ptr, layout)
    }

    fn manages_on_its_own(&self, layout: alloc::Layout) -> bool {
        (*self).manages_on_its_own(layout)
    }
}

pub struct Or<Primary: Allocator, Fallback: Allocator>(pub Primary, pub Fallback);

unsafe impl<Primary: Composable, Fallback: Composable> Composable for Or<Primary, Fallback> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        self.0.has_allocated(ptr, layout) || self.1.has_allocated(ptr, layout)
    }

    fn manages_on_its_own(&self, layout: alloc::Layout) -> bool {
        self.0.manages_on_its_own(layout) || self.1.manages_on_its_own(layout)
    }
}

unsafe impl<Primary: Composable, Fallback: Allocator> Allocator for Or<Primary, Fallback> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if let Ok(block) = self.0.allocate(layout) {
            Ok(block)
        } else if self.0.manages_on_its_own(layout) {
            Err(AllocError)
        } else {
            self.1.allocate(layout)
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
