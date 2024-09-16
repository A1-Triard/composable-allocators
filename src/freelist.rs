use crate::base::*;
use const_default::ConstDefault;
use const_default_derive::ConstDefault;
use core::alloc::{self, AllocError, Allocator};
use core::mem::{align_of, size_of};
use core::ptr::{self, NonNull, null_mut};
use core::sync::atomic::AtomicPtr;
use sync_no_std::mutex::Mutex;

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

pub struct FixedLimit {
    limit: usize,
}

unsafe impl LimitParam for FixedLimit {
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

struct Node {
    next: AtomicPtr<u8>,
}

struct List<Limit: LimitParam> {
    head: Node,
    len: <Limit as LimitParam>::ListLen,
}

pub struct Freelist<Limit: LimitParam, A: Allocator + Clone> {
    list: Mutex<List<Limit>, A>,
    layout: alloc::Layout,
    tolerance: alloc::Layout,
    limit: Limit,
}

unsafe impl<Limit: LimitParam, A: NonUnwinding + Clone> NonUnwinding for Freelist<Limit, A> { }

impl<Limit: LimitParam, A: Allocator + Clone> Freelist<Limit, A> {
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
            list: Mutex::new_in(List {
                head: Node { next: AtomicPtr::new(null_mut()) },
                len: ConstDefault::DEFAULT,
            }, base),
            layout,
            tolerance,
            limit,
        }
    }

    fn manages(&self, layout: alloc::Layout) -> bool {
        (self.tolerance.size() ..= self.layout.size()).contains(&layout.size()) &&
        (self.tolerance.align() ..= self.layout.size()).contains(&layout.align())
    }

    fn base(&self) -> &A { self.list.allocator() }
}

unsafe impl<Limit: LimitParam, A: Fallbackable + Clone> Fallbackable for Freelist<Limit, A> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        let layout = if self.manages(layout) { self.layout } else { layout };
        self.base().has_allocated(ptr, layout)
    }

    fn allows_fallback(&self, layout: alloc::Layout) -> bool {
        let layout = if self.manages(layout) { self.layout } else { layout };
        self.base().allows_fallback(layout)
    }
}

unsafe impl<Limit: LimitParam, A: Allocator + Clone> Allocator for Freelist<Limit, A> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if !self.manages(layout) {
            return self.base().allocate(layout);
        }
        let mut list = self.list.lock().unwrap();
        if let Some(next_ptr) = NonNull::new(*list.head.next.get_mut()) {
            let next = unsafe { ptr::read(next_ptr.as_ptr() as *const Node) }.next;
            list.head = Node { next };
            list.len = unsafe { self.limit.dec_list_len(list.len) };
            Ok(NonNull::slice_from_raw_parts(next_ptr, self.layout.size()))
        } else {
            self.base().allocate(self.layout)
        }
    }

    fn allocate_zeroed(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if !self.manages(layout) {
            return self.base().allocate_zeroed(layout);
        }
        let mut list = self.list.lock().unwrap();
        if let Some(next_ptr) = NonNull::new(*list.head.next.get_mut()) {
            let next = unsafe { ptr::read(next_ptr.as_ptr() as *const Node) }.next;
            list.head = Node { next };
            list.len = unsafe { self.limit.dec_list_len(list.len) };
            let ptr = NonNull::slice_from_raw_parts(next_ptr, self.layout.size());
            unsafe { ptr.as_mut_ptr().write_bytes(0, ptr.len()); }
            Ok(ptr)
        } else {
            self.base().allocate_zeroed(self.layout)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        let mut list = self.list.lock().unwrap();
        if self.limit.limit_reached(list.len) || !self.manages(layout) {
            return self.base().deallocate(ptr, layout);
        }
        ptr::write(ptr.as_ptr() as *mut Node, Node { next: AtomicPtr::new(*list.head.next.get_mut()) });
        *list.head.next.get_mut() = ptr.as_ptr();
        list.len = unsafe { self.limit.inc_list_len(list.len) };
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_layout = if self.manages(old_layout) { self.layout } else { old_layout };
        self.base().grow(ptr, old_layout, new_layout)
    }

    unsafe fn grow_zeroed(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_layout = if self.manages(old_layout) { self.layout } else { old_layout };
        self.base().grow_zeroed(ptr, old_layout, new_layout)
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
        self.base().shrink(ptr, old_layout, new_layout)
    }
}
