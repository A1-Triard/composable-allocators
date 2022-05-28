use crate::base::*;
use core::alloc::{self, AllocError, Allocator};
use core::cell::Cell;
use core::mem::{MaybeUninit, transmute};
use core::ptr::NonNull;

/// # Safety
///
/// The [`buf_len`](Params::buf_len) method should
/// return constant value (i.e. same value on every call).
///
/// Returning value should satisfy `buf_len() <= isize::MAX as usize`.
pub unsafe trait Params {
    fn buf_len(&self) -> usize;
}

pub struct CtParams<const BUF_LEN: usize>(());

impl<const BUF_LEN: usize> CtParams<BUF_LEN> {
    pub const fn new() -> Self {
        assert!(BUF_LEN <= isize::MAX as usize);
        CtParams(())
    }
}

impl<const BUF_LEN: usize> const Default for CtParams<BUF_LEN> {
    fn default() -> Self { Self::new() }
}

unsafe impl<const BUF_LEN: usize> const Params for CtParams<BUF_LEN> {
    fn buf_len(&self) -> usize { BUF_LEN }
}

pub struct RtParams {
    buf_len: usize,
}

impl RtParams {
    /// # Safety
    ///
    /// Argument should satisfy `buf_len <= isize::MAX as usize`.
    pub unsafe fn new_unchecked(buf_len: usize) -> Self {
        RtParams { buf_len }
    }

    pub fn new(buf_len: usize) -> Self {
        assert!(buf_len <= isize::MAX as usize);
        unsafe { Self::new_unchecked(buf_len) }
    }
}

unsafe impl Params for RtParams {
    fn buf_len(&self) -> usize { self.buf_len }
}

pub struct Stacked<P: Params> {
    buf_ptr: NonNull<u8>,
    params: P,
    allocated: Cell<usize>,
    allocations_count: Cell<usize>,
}

impl<P: Params> Drop for Stacked<P> {
    fn drop(&mut self) {
        assert!(self.allocations_count.get() == 0, "memory leaks in Stacked allocator");
    }
}

impl<P: Params> Stacked<P> {
    /// # Safety
    ///
    /// `buf_ptr` should be a valid pointer to a slice with `params.buf_len()` bytes length.
    ///
    /// Arguments should satisfy
    /// `(isize::MAX as usize) - params.buf_len() >= buf_ptr as usize`
    pub unsafe fn with_params<T>(
        params: P,
        buf_ptr: NonNull<MaybeUninit<u8>>,
        f: impl for<'a> FnOnce(&'a Stacked<P>) -> T
    ) -> T {
        let stacked = Stacked {
            buf_ptr: transmute(buf_ptr),
            params,
            allocated: Cell::new(0),
            allocations_count: Cell::new(0),
        };
        f(&stacked)
    }
}

pub fn with_size<const BUF_LEN: usize, T>(
    f: impl for<'a> FnOnce(&'a Stacked<CtParams<BUF_LEN>>) -> T
) -> T {
    let mut buf: [MaybeUninit<u8>; BUF_LEN] = unsafe { MaybeUninit::uninit().assume_init() };
    let buf_ptr = unsafe { NonNull::new_unchecked(buf.as_mut_ptr()) };
    assert!((isize::MAX as usize) - BUF_LEN >= buf_ptr.as_ptr() as usize);
    unsafe { Stacked::with_params(CtParams::<BUF_LEN>::new(), buf_ptr, f) }
}

pub fn with_buf<T>(
    buf: &mut [MaybeUninit<u8>],
    f: impl for<'a> FnOnce(&'a Stacked<RtParams>) -> T
) -> T {
    let buf_len = buf.len();
    assert!(buf_len <= isize::MAX as usize && (isize::MAX as usize) - buf_len >= buf.as_ptr() as usize);
    let buf_ptr = unsafe { NonNull::new_unchecked(buf.as_mut_ptr()) };
    unsafe { Stacked::with_params(RtParams::new_unchecked(buf_len), buf_ptr, f) }
}

unsafe impl<P: Params> Fallbackable for Stacked<P> {
    unsafe fn has_allocated(&self, ptr: NonNull<u8>, _layout: alloc::Layout) -> bool {
        if let Some(offset) = (ptr.as_ptr() as usize).checked_sub(self.buf_ptr.as_ptr() as usize) {
            offset < self.params.buf_len() && self.buf_ptr.as_ptr().add(offset) == ptr.as_ptr()
        } else {
            false
        }
    }

    fn allows_fallback(&self, _layout: alloc::Layout) -> bool {
        true
    }
}

unsafe impl<P: Params> Allocator for Stacked<P> {
    fn allocate(&self, layout: alloc::Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = unsafe { self.buf_ptr.as_ptr().add(self.allocated.get()) };
        let padding = (layout.align() - (ptr as usize) % layout.align()) % layout.align();
        let size = padding.checked_add(layout.size()).ok_or(AllocError)?;
        if size > self.params.buf_len() - self.allocated.get() { return Err(AllocError); }
        self.allocations_count.set(self.allocations_count.get().checked_add(1).ok_or(AllocError)?);
        let res = NonNull::slice_from_raw_parts(
            unsafe { NonNull::new_unchecked(ptr.add(padding)) },
            layout.size()
        );
        self.allocated.set(self.allocated.get() + size);
        Ok(res)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        if
            ptr.as_ptr().add(layout.size()) ==
            self.buf_ptr.as_ptr().add(self.allocated.get())
        {
            self.allocated.set(self.allocated.get() - layout.size());
        }
        self.allocations_count.set(self.allocations_count.get() - 1);
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        if
            ptr.as_ptr().add(old_layout.size()) ==
            self.buf_ptr.as_ptr().add(self.allocated.get())
        {
            self.allocated.set(self.allocated.get() - old_layout.size());
            if let Ok(block) = self.allocate(new_layout) {
                Ok(block)
            } else {
                self.allocated.set(self.allocated.get() + old_layout.size());
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
        if
            ptr.as_ptr().add(old_layout.size()) ==
            self.buf_ptr.as_ptr().add(self.allocated.get())
        {
            self.allocated.set(self.allocated.get() - old_layout.size());
            if let Ok(block) = self.allocate(new_layout) {
                Ok(block)
            } else {
                self.allocated.set(self.allocated.get() + old_layout.size());
                Err(AllocError)
            }
        } else {
            if new_layout.align() > old_layout.align() {
                Err(AllocError)
            } else {
                Ok(NonNull::slice_from_raw_parts(ptr, old_layout.size()))
            }
        }
    }
}
