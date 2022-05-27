use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::cell::Cell;
use core::mem::{align_of, size_of};
use core::ptr::{self, NonNull};

pub const MIN_LAYOUT_SIZE: usize = size_of::<Node>();

pub const MIN_LAYOUT_ALIGN: usize = align_of::<Node>();

/// # Safety
///
/// All methods should return constants, i.e. same values on every call.
///
/// Returned values should satisfy
/// `tolerance().size() <= layout().size() && tolerance().align() <= layout().align()`,
/// and
/// `layout.size() >= MIN_LAYOUT_SIZE && layout.align() >= MIN_LAYOUT_ALIGN`.
pub unsafe trait Params {
    fn layout(&self) -> alloc::Layout;
    fn tolerance(&self) -> alloc::Layout;
    fn top(&self) -> usize;
}

pub struct CtParams<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
    const TOLERANCE_SIZE: usize,
    const TOLERANCE_ALIGN: usize,
    const TOP: usize,
>(());

const fn is_power_of_two(x: usize) -> bool {
    x != 0 && (x & (x - 1)) == 0
}

impl<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
    const TOLERANCE_SIZE: usize,
    const TOLERANCE_ALIGN: usize,
    const TOP: usize,
> CtParams<LAYOUT_SIZE, LAYOUT_ALIGN, TOLERANCE_SIZE, TOLERANCE_ALIGN, TOP> {
    pub const fn new() -> Self {
        assert!(LAYOUT_SIZE <= isize::MAX as usize);
        assert!(LAYOUT_ALIGN <= isize::MAX as usize);
        assert!(TOLERANCE_SIZE <= isize::MAX as usize);
        assert!(TOLERANCE_SIZE <= isize::MAX as usize);
        assert!(((LAYOUT_SIZE + LAYOUT_ALIGN - 1) / LAYOUT_ALIGN) * LAYOUT_ALIGN <= isize::MAX as usize);
        assert!(((TOLERANCE_SIZE + TOLERANCE_ALIGN - 1) / TOLERANCE_ALIGN) * TOLERANCE_ALIGN <= isize::MAX as usize);
        assert!(is_power_of_two(LAYOUT_ALIGN) && is_power_of_two(TOLERANCE_ALIGN));
        assert!(TOLERANCE_SIZE <= LAYOUT_SIZE && TOLERANCE_ALIGN <= LAYOUT_ALIGN);
        assert!(LAYOUT_SIZE >= MIN_LAYOUT_SIZE && LAYOUT_ALIGN >= MIN_LAYOUT_ALIGN);
        CtParams(())
    }
}

impl<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
    const TOLERANCE_SIZE: usize,
    const TOLERANCE_ALIGN: usize,
    const TOP: usize
> const Default for CtParams<LAYOUT_SIZE, LAYOUT_ALIGN, TOLERANCE_SIZE, TOLERANCE_ALIGN, TOP> {
    fn default() -> Self { Self::new() }
}

unsafe impl<
    const LAYOUT_SIZE: usize,
    const LAYOUT_ALIGN: usize,
    const TOLERANCE_SIZE: usize,
    const TOLERANCE_ALIGN: usize,
    const TOP: usize,
> const Params for CtParams<LAYOUT_SIZE, LAYOUT_ALIGN, TOLERANCE_SIZE, TOLERANCE_ALIGN, TOP> {
    fn layout(&self) -> alloc::Layout {
        unsafe { alloc::Layout::from_size_align_unchecked(LAYOUT_SIZE, LAYOUT_ALIGN) }
    }

    fn tolerance(&self) -> alloc::Layout {
        unsafe { alloc::Layout::from_size_align_unchecked(TOLERANCE_SIZE, TOLERANCE_ALIGN) }
    }

    fn top(&self) -> usize { TOP }
}

pub struct RtParams {
    layout: alloc::Layout,
    tolerance: alloc::Layout,
    top: usize,
}

impl RtParams {
    /// # Safety
    ///
    /// Arguments should satisfy
    /// `tolerance.size() <= layout.size() && tolerance.align() <= layout.align()`,
    /// and
    /// `layout.size() >= MIN_LAYOUT_SIZE && layout.align() >= MIN_LAYOUT_ALIGN`.
    pub unsafe fn new_unchecked(layout: alloc::Layout, tolerance: alloc::Layout, top: usize) -> Self {
        RtParams { layout, tolerance, top }
    }

    pub fn new(layout: alloc::Layout, tolerance: alloc::Layout, top: usize) -> Self {
        assert!(tolerance.size() <= layout.size() && tolerance.align() <= layout.align());
        assert!(layout.size() >= MIN_LAYOUT_SIZE && layout.align() >= MIN_LAYOUT_ALIGN);
        unsafe { RtParams::new_unchecked(layout, tolerance, top) }
    }
}

unsafe impl Params for RtParams {
    fn layout(&self) -> alloc::Layout { self.layout }

    fn tolerance(&self) -> alloc::Layout { self.tolerance }

    fn top(&self) -> usize { self.top }
}

#[derive(Clone, Copy)]
struct Node {
    next: Option<NonNull<u8>>,
}

pub struct Freelist<P: Params, A: Allocator> {
    base: A,
    list: Cell<Node>,
    list_len: Cell<usize>,
    params: P,
}

impl<P: Params, A: Allocator> Freelist<P, A> {
    pub fn new(params: P, base: A) -> Self {
        Freelist { base, list: Cell::new(Node { next: None }), list_len: Cell::new(0), params }
    }

    fn manage(&self, layout: alloc::Layout) -> bool {
        (self.params.tolerance().size() ..= self.params.layout().size()).contains(&layout.size()) &&
        (self.params.tolerance().align() ..= self.params.layout().size()).contains(&layout.align())
    }
}

unsafe impl<P: Params, A: Composable> Composable for Freelist<P, A> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, layout: alloc::Layout) -> bool {
        self.base.has_allocated(ptr, layout)
    }
}

unsafe impl<P: Params, A: Allocator> Allocator for Freelist<P, A> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        if !self.manage(layout) {
            return self.base.allocate(layout);
        }
        if let Some(list) = self.list.get().next {
            let next = unsafe { ptr::read(list.as_ptr() as *const Node) }.next;
            self.list.set(Node { next });
            self.list_len.set(self.list_len.get() - 1);
            Ok(NonNull::slice_from_raw_parts(list, self.params.layout().size()))
        } else {
            self.base.allocate(self.params.layout())
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        if self.list_len.get() == self.params.top() || !self.manage(layout) {
            return self.base.deallocate(ptr, layout);
        }
        ptr::write(ptr.as_ptr() as *mut Node, self.list.get());
        self.list.set(Node { next: Some(ptr) });
        self.list_len.set(self.list_len.get() + 1);
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_layout = if self.manage(old_layout) { self.params.layout() } else { old_layout };
        self.base.grow(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        let old_layout = if self.manage(old_layout) {
            if self.manage(new_layout) {
                return Ok(NonNull::slice_from_raw_parts(ptr, self.params.layout().size()));
            }
            self.params.layout()
        } else {
            old_layout
        };
        self.base.shrink(ptr, old_layout, new_layout)
    }
}
