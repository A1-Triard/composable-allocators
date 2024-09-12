use crate::base::*;
use const_default::ConstDefault;
use const_default_derive::ConstDefault;
use core::alloc::{self, AllocError, Allocator};
use core::cell::Cell;
use core::mem::{align_of, size_of};
use core::ptr::{self, NonNull};

pub const MIN_LAYOUT_SIZE: usize = size_of::<Node>();

pub const MIN_LAYOUT_ALIGN: usize = align_of::<Node>();

/// # Safety
///
/// This trait cannot be implemented outside of this module.
pub unsafe trait LimitParam {
    #[doc(hidden)]
    type ListLen: ConstDefault + Copy;

    #[doc(hidden)]
    #[allow(clippy::missing_safety_doc)]
    unsafe fn limit_reached(&self, list_len: Self::ListLen) -> bool;

    #[doc(hidden)]
    #[allow(clippy::missing_safety_doc)]
    unsafe fn dec_list_len(&self, list_len: Self::ListLen) -> Self::ListLen;

    #[doc(hidden)]
    #[allow(clippy::missing_safety_doc)]
    unsafe fn inc_list_len(&self, list_len: Self::ListLen) -> Self::ListLen;
}

#[derive(ConstDefault)]
pub struct NoLimit;

unsafe impl LimitParam for NoLimit {
    type ListLen = ();

    unsafe fn limit_reached(&self, (): Self::ListLen) -> bool { false }

    unsafe fn dec_list_len(&self, (): Self::ListLen) -> Self::ListLen { }

    unsafe fn inc_list_len(&self, (): Self::ListLen) -> Self::ListLen { }
}

#[doc(hidden)]
#[derive(ConstDefault, Clone, Copy)]
pub struct FixedLimitListLen(usize);

#[derive(ConstDefault)]
pub struct FixedCtLimit<const LIMIT: usize>;

unsafe impl<const LIMIT: usize> LimitParam for FixedCtLimit<LIMIT> {
    type ListLen = FixedLimitListLen;

    unsafe fn limit_reached(&self, list_len: Self::ListLen) -> bool {
        list_len.0 == LIMIT
    }

    unsafe fn dec_list_len(&self, list_len: Self::ListLen) -> Self::ListLen {
        FixedLimitListLen(list_len.0 - 1)
    }

    unsafe fn inc_list_len(&self, list_len: Self::ListLen) -> Self::ListLen {
        FixedLimitListLen(list_len.0 + 1)
    }
}

pub struct FixedRtLimit {
    limit: usize,
}

unsafe impl LimitParam for FixedRtLimit {
    type ListLen = FixedLimitListLen;

    unsafe fn limit_reached(&self, list_len: Self::ListLen) -> bool {
        list_len.0 == self.limit
    }

    unsafe fn dec_list_len(&self, list_len: Self::ListLen) -> Self::ListLen {
        FixedLimitListLen(list_len.0 - 1)
    }

    unsafe fn inc_list_len(&self, list_len: Self::ListLen) -> Self::ListLen {
        FixedLimitListLen(list_len.0 + 1)
    }
}

#[derive(Clone, Copy)]
struct Node {
    next: Option<NonNull<u8>>,
}

pub struct Freelist<Limit: LimitParam, A: Allocator> {
    base: A,
    list: Cell<Node>,
    list_len: Cell<<Limit as LimitParam>::ListLen>,
    layout: alloc::Layout,
    tolerance: alloc::Layout,
    limit: Limit,
}

unsafe impl<Limit: LimitParam, A: NonUnwinding> NonUnwinding for Freelist<Limit, A> { }

impl<Limit: LimitParam, A: Allocator> Freelist<Limit, A> {
    pub const fn new(layout: alloc::Layout, tolerance: alloc::Layout, limit: Limit, base: A) -> Self {
        assert!(tolerance.size() <= layout.size() && tolerance.align() <= layout.align());
        assert!(layout.size() >= MIN_LAYOUT_SIZE && layout.align() >= MIN_LAYOUT_ALIGN);
        unsafe { Self::new_unchecked(layout, tolerance, limit, base) }
    }

    /// # Safety
    ///
    /// Arguments should satisfy
    /// `tolerance.size() <= layout.size() && tolerance.align() <= layout.align()`,
    /// and
    /// `layout.size() >= MIN_LAYOUT_SIZE && layout.align() >= MIN_LAYOUT_ALIGN`.
    pub const unsafe fn new_unchecked(layout: alloc::Layout, tolerance: alloc::Layout, limit: Limit, base: A) -> Self {
        Freelist {
            base,
            list: Cell::new(Node { next: None }),
            list_len: Cell::new(ConstDefault::DEFAULT),
            layout,
            tolerance,
            limit,
        }
    }

    fn manages(&self, layout: alloc::Layout) -> bool {
        (self.tolerance.size() ..= self.layout.size()).contains(&layout.size()) &&
        (self.tolerance.align() ..= self.layout.size()).contains(&layout.align())
    }
}

unsafe impl<Limit: LimitParam, A: Fallbackable> Fallbackable for Freelist<Limit, A> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        let layout = if self.manages(layout) { self.layout } else { layout };
        self.base.has_allocated(ptr, layout)
    }

    fn allows_fallback(&self, layout: alloc::Layout) -> bool {
        let layout = if self.manages(layout) { self.layout } else { layout };
        self.base.allows_fallback(layout)
    }
}

unsafe impl<Limit: LimitParam, A: Allocator> Allocator for Freelist<Limit, A> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if !self.manages(layout) {
            return self.base.allocate(layout);
        }
        if let Some(list) = self.list.get().next {
            let next = unsafe { ptr::read(list.as_ptr() as *const Node) }.next;
            self.list.set(Node { next });
            self.list_len.set(unsafe { self.limit.dec_list_len(self.list_len.get()) });
            Ok(NonNull::slice_from_raw_parts(list, self.layout.size()))
        } else {
            self.base.allocate(self.layout)
        }
    }

    fn allocate_zeroed(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if !self.manages(layout) {
            return self.base.allocate_zeroed(layout);
        }
        if let Some(list) = self.list.get().next {
            let next = unsafe { ptr::read(list.as_ptr() as *const Node) }.next;
            self.list.set(Node { next });
            self.list_len.set(unsafe { self.limit.dec_list_len(self.list_len.get()) });
            let ptr = NonNull::slice_from_raw_parts(list, self.layout.size());
            unsafe { ptr.as_mut_ptr().write_bytes(0, ptr.len()); }
            Ok(ptr)
        } else {
            self.base.allocate_zeroed(self.layout)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        if self.limit.limit_reached(self.list_len.get()) || !self.manages(layout) {
            return self.base.deallocate(ptr, layout);
        }
        ptr::write(ptr.as_ptr() as *mut Node, self.list.get());
        self.list.set(Node { next: Some(ptr) });
        self.list_len.set(unsafe { self.limit.inc_list_len(self.list_len.get()) });
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_layout = if self.manages(old_layout) { self.layout } else { old_layout };
        self.base.grow(ptr, old_layout, new_layout)
    }

    unsafe fn grow_zeroed(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_layout = if self.manages(old_layout) { self.layout } else { old_layout };
        self.base.grow_zeroed(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_layout = if self.manages(old_layout) {
            if self.manages(new_layout) {
                return Ok(NonNull::slice_from_raw_parts(ptr, self.layout.size()));
            }
            self.layout
        } else {
            old_layout
        };
        self.base.shrink(ptr, old_layout, new_layout)
    }
}
