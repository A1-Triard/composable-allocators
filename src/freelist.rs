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

/// # Safety
///
/// All methods should return constants, i.e. same values on every call.
///
/// Returned values should satisfy
/// `tolerance().size() <= layout().size() && tolerance().align() <= layout().align()`,
/// and
/// `layout.size() >= MIN_LAYOUT_SIZE && layout.align() >= MIN_LAYOUT_ALIGN`.
#[const_trait]
pub unsafe trait Params {
    type Limit: LimitParam;
    fn layout(&self) -> alloc::Layout;
    fn tolerance(&self) -> alloc::Layout;
    fn limit(&self) -> &Self::Limit;
}

pub struct CtParams<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
    const TOLERANCE_SIZE: usize,
    const TOLERANCE_ALIGN: usize,
    Limit: LimitParam,
> {
    limit: Limit,
}

const fn is_power_of_two(x: usize) -> bool {
    x != 0 && (x & (x - 1)) == 0
}

impl<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
    const TOLERANCE_SIZE: usize,
    const TOLERANCE_ALIGN: usize,
    Limit: LimitParam,
> CtParams<LAYOUT_SIZE, LAYOUT_ALIGN, TOLERANCE_SIZE, TOLERANCE_ALIGN, Limit> {
    pub const fn new(limit: Limit) -> Self {
        assert!(LAYOUT_SIZE <= isize::MAX as usize);
        assert!(LAYOUT_ALIGN <= isize::MAX as usize);
        assert!(TOLERANCE_SIZE <= isize::MAX as usize);
        assert!(TOLERANCE_SIZE <= isize::MAX as usize);
        assert!(is_power_of_two(LAYOUT_ALIGN) && is_power_of_two(TOLERANCE_ALIGN));
        assert!(((LAYOUT_SIZE + LAYOUT_ALIGN - 1) / LAYOUT_ALIGN) * LAYOUT_ALIGN <= isize::MAX as usize);
        assert!(((TOLERANCE_SIZE + TOLERANCE_ALIGN - 1) / TOLERANCE_ALIGN) * TOLERANCE_ALIGN <= isize::MAX as usize);
        assert!(TOLERANCE_SIZE <= LAYOUT_SIZE && TOLERANCE_ALIGN <= LAYOUT_ALIGN);
        assert!(LAYOUT_SIZE >= MIN_LAYOUT_SIZE && LAYOUT_ALIGN >= MIN_LAYOUT_ALIGN);
        CtParams { limit }
    }
}

impl<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
    const TOLERANCE_SIZE: usize,
    const TOLERANCE_ALIGN: usize,
    Limit: LimitParam + ConstDefault,
> ConstDefault for CtParams<LAYOUT_SIZE, LAYOUT_ALIGN, TOLERANCE_SIZE, TOLERANCE_ALIGN, Limit> {
    const DEFAULT: Self = Self::new(Limit::DEFAULT);
}

unsafe impl<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
    const TOLERANCE_SIZE: usize,
    const TOLERANCE_ALIGN: usize,
    Limit: LimitParam,
> const Params for CtParams<LAYOUT_SIZE, LAYOUT_ALIGN, TOLERANCE_SIZE, TOLERANCE_ALIGN, Limit> {
    type Limit = Limit;

    fn layout(&self) -> alloc::Layout {
        unsafe { alloc::Layout::from_size_align_unchecked(LAYOUT_SIZE, LAYOUT_ALIGN) }
    }

    fn tolerance(&self) -> alloc::Layout {
        unsafe { alloc::Layout::from_size_align_unchecked(TOLERANCE_SIZE, TOLERANCE_ALIGN) }
    }

    fn limit(&self) -> &Limit { &self.limit }
}

pub struct RtParams<Limit: LimitParam> {
    layout: alloc::Layout,
    tolerance: alloc::Layout,
    limit: Limit,
}

impl<Limit: LimitParam> RtParams<Limit> {
    /// # Safety
    ///
    /// Arguments should satisfy
    /// `tolerance.size() <= layout.size() && tolerance.align() <= layout.align()`,
    /// and
    /// `layout.size() >= MIN_LAYOUT_SIZE && layout.align() >= MIN_LAYOUT_ALIGN`.
    pub const unsafe fn new_unchecked(layout: alloc::Layout, tolerance: alloc::Layout, limit: Limit) -> Self {
        RtParams { layout, tolerance, limit }
    }

    pub const fn new(layout: alloc::Layout, tolerance: alloc::Layout, limit: Limit) -> Self {
        assert!(tolerance.size() <= layout.size() && tolerance.align() <= layout.align());
        assert!(layout.size() >= MIN_LAYOUT_SIZE && layout.align() >= MIN_LAYOUT_ALIGN);
        unsafe { RtParams::new_unchecked(layout, tolerance, limit) }
    }
}

unsafe impl<Limit: LimitParam> Params for RtParams<Limit> {
    type Limit = Limit;

    fn layout(&self) -> alloc::Layout { self.layout }

    fn tolerance(&self) -> alloc::Layout { self.tolerance }

    fn limit(&self) -> &Limit { &self.limit }
}

#[derive(Clone, Copy)]
struct Node {
    next: Option<NonNull<u8>>,
}

pub struct Freelist<P: Params, A: Allocator> {
    base: A,
    list: Cell<Node>,
    list_len: Cell<<P::Limit as LimitParam>::ListLen>,
    params: P,
}

unsafe impl<P: Params, A: NonUnwinding> NonUnwinding for Freelist<P, A> { }

impl<P: Params, A: Allocator> Freelist<P, A> {
    pub const fn new(params: P, base: A) -> Self {
        Freelist { base, list: Cell::new(Node { next: None }), list_len: Cell::new(ConstDefault::DEFAULT), params }
    }

    fn manages(&self, layout: alloc::Layout) -> bool {
        (self.params.tolerance().size() ..= self.params.layout().size()).contains(&layout.size()) &&
        (self.params.tolerance().align() ..= self.params.layout().size()).contains(&layout.align())
    }
}

unsafe impl<P: Params, A: Fallbackable> Fallbackable for Freelist<P, A> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        let layout = if self.manages(layout) { self.params.layout() } else { layout };
        self.base.has_allocated(ptr, layout)
    }

    fn allows_fallback(&self, layout: alloc::Layout) -> bool {
        let layout = if self.manages(layout) { self.params.layout() } else { layout };
        self.base.allows_fallback(layout)
    }
}

unsafe impl<P: Params, A: Allocator> Allocator for Freelist<P, A> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if !self.manages(layout) {
            return self.base.allocate(layout);
        }
        if let Some(list) = self.list.get().next {
            let next = unsafe { ptr::read(list.as_ptr() as *const Node) }.next;
            self.list.set(Node { next });
            self.list_len.set(unsafe { self.params.limit().dec_list_len(self.list_len.get()) });
            Ok(NonNull::slice_from_raw_parts(list, self.params.layout().size()))
        } else {
            self.base.allocate(self.params.layout())
        }
    }

    fn allocate_zeroed(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if !self.manages(layout) {
            return self.base.allocate_zeroed(layout);
        }
        if let Some(list) = self.list.get().next {
            let next = unsafe { ptr::read(list.as_ptr() as *const Node) }.next;
            self.list.set(Node { next });
            self.list_len.set(unsafe { self.params.limit().dec_list_len(self.list_len.get()) });
            let ptr = NonNull::slice_from_raw_parts(list, self.params.layout().size());
            unsafe { ptr.as_mut_ptr().write_bytes(0, ptr.len()); }
            Ok(ptr)
        } else {
            self.base.allocate_zeroed(self.params.layout())
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        if self.params.limit().limit_reached(self.list_len.get()) || !self.manages(layout) {
            return self.base.deallocate(ptr, layout);
        }
        ptr::write(ptr.as_ptr() as *mut Node, self.list.get());
        self.list.set(Node { next: Some(ptr) });
        self.list_len.set(unsafe { self.params.limit().inc_list_len(self.list_len.get()) });
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_layout = if self.manages(old_layout) { self.params.layout() } else { old_layout };
        self.base.grow(ptr, old_layout, new_layout)
    }

    unsafe fn grow_zeroed(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_layout = if self.manages(old_layout) { self.params.layout() } else { old_layout };
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
                return Ok(NonNull::slice_from_raw_parts(ptr, self.params.layout().size()));
            }
            self.params.layout()
        } else {
            old_layout
        };
        self.base.shrink(ptr, old_layout, new_layout)
    }
}
