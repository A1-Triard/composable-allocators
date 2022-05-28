use core::alloc::{self, Allocator};
use core::ptr::NonNull;

/// # Safety
///
/// The [`allows_fallback`](Fallbackable::allows_fallback) function should be pure, i.e. always return same value
/// for same `layout`.
///
/// An implementer of this trait should return `true` from
/// [`has_allocated`](Fallbackable::has_allocated)
/// if and only if at least one of the following conditions is satisfied:
///
/// - the passed pointer is denoting to
///   [currently allocated block](https://doc.rust-lang.org/core/alloc/trait.Allocator.html#currently-allocated-memory),
///
/// - [`allows_fallback`](Fallbackable::allows_fallback) returns false for [`Layout`](alloc::Layout) used to allocate
///   memory block, denoting by `ptr`.
pub unsafe trait Fallbackable: Allocator {
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

    fn allows_fallback(&self, layout: alloc::Layout) -> bool;
}

unsafe impl<'a, T: Fallbackable> Fallbackable for &'a T {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        (*self).has_allocated(ptr, layout)
    }

    fn allows_fallback(&self, layout: alloc::Layout) -> bool {
        (*self).allows_fallback(layout)
    }
}
