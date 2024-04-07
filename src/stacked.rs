use crate::base::*;
use const_default::ConstDefault;
use core::alloc::{self, AllocError, Allocator};
use core::mem::{MaybeUninit};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// # Safety
///
/// The [`buf_len`](Params::buf_len) method should
/// return constant value (i.e. same value on every call).
///
/// Returning value should satisfy `buf_len() <= isize::MAX as usize`.
#[const_trait]
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

impl<const BUF_LEN: usize> ConstDefault for CtParams<BUF_LEN> {
    const DEFAULT: Self = Self::new();
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
    pub const unsafe fn new_unchecked(buf_len: usize) -> Self {
        RtParams { buf_len }
    }

    pub const fn new(buf_len: usize) -> Self {
        assert!(buf_len <= isize::MAX as usize);
        unsafe { Self::new_unchecked(buf_len) }
    }
}

unsafe impl Params for RtParams {
    fn buf_len(&self) -> usize { self.buf_len }
}

pub struct Stacked<P: Params> {
    buf_ptr: AtomicPtr<u8>,
    params: P,
    allocated: AtomicUsize,
    allocations_count: AtomicUsize,
}

impl<P: Params> Drop for Stacked<P> {
    fn drop(&mut self) {
        assert!(self.allocations_count.load(Ordering::Relaxed) == 0, "memory leaks in Stacked allocator");
    }
}

unsafe impl<P: Params> NonUnwinding for Stacked<P> { }

impl Stacked<RtParams> {
    pub const fn from_static_slice(
        buf: &'static mut [MaybeUninit<u8>],
    ) -> Self {
        Stacked {
            buf_ptr: AtomicPtr::new(buf.as_mut_ptr() as *mut u8),
            params: RtParams { buf_len: buf.len() },
            allocated: AtomicUsize::new(0),
            allocations_count: AtomicUsize::new(0),
        }
    }
}

impl<const BUF_LEN: usize> Stacked<CtParams<BUF_LEN>> {
    pub const fn from_static_array(
        buf: &'static mut [MaybeUninit<u8>; BUF_LEN],
    ) -> Self {
        Stacked {
            buf_ptr: AtomicPtr::new(buf.as_mut_ptr() as *mut u8),
            params: CtParams(()),
            allocated: AtomicUsize::new(0),
            allocations_count: AtomicUsize::new(0),
        }
    }
}

impl<P: Params> Stacked<P> {
    /// # Safety
    ///
    /// `buf_ptr` should be a valid unique pointer to a slice with `params.buf_len()` bytes length.
    ///
    /// Arguments should satisfy
    /// `(isize::MAX as usize) - params.buf_len() >= buf_ptr as usize`
    pub unsafe fn with_params<T>(
        params: P,
        buf_ptr: NonNull<MaybeUninit<u8>>,
        f: impl for<'a> FnOnce(&'a Stacked<P>) -> T
    ) -> T {
        let stacked = Stacked {
            buf_ptr: AtomicPtr::new(buf_ptr.as_ptr() as *mut u8),
            params,
            allocated: AtomicUsize::new(0),
            allocations_count: AtomicUsize::new(0),
        };
        f(&stacked)
    }

    unsafe fn grow_raw(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout,
        zeroed: bool,
    ) -> Result<NonNull<[u8]>, AllocError> {
        if new_layout.align() > old_layout.align() { return Err(AllocError); }
        let start_offset = ptr.as_ptr().offset_from(self.buf_ptr.load(Ordering::Relaxed)) as usize;
        if new_layout.size() > self.params.buf_len() - start_offset { return Err(AllocError); }
        let old_end_offset = start_offset + old_layout.size();
        let new_end_offset = start_offset + new_layout.size();
        self.allocated.compare_exchange(old_end_offset, new_end_offset, Ordering::Relaxed, Ordering::Relaxed)
            .map_err(|_| AllocError)?;
        if zeroed {
            ptr.as_ptr().add(old_layout.size()).write_bytes(0, new_layout.size() - old_layout.size());
        }
        Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
    }
}

pub fn with_size<const BUF_LEN: usize, T>(
    f: impl for<'a> FnOnce(&'a Stacked<CtParams<BUF_LEN>>) -> T
) -> T {
    let mut buf: [MaybeUninit<u8>; BUF_LEN] = MaybeUninit::uninit_array();
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
        if let Some(offset) = (ptr.as_ptr() as usize).checked_sub(self.buf_ptr.load(Ordering::Relaxed) as usize) {
            offset < self.params.buf_len() && self.buf_ptr.load(Ordering::Relaxed).add(offset) == ptr.as_ptr()
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
        let mut padding = MaybeUninit::uninit();
        let allocated = self.allocated.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |allocated| {
            let ptr = unsafe { self.buf_ptr.load(Ordering::Relaxed).add(allocated) };
            let padding = padding.write((layout.align() - (ptr as usize) % layout.align()) % layout.align());
            let size = padding.checked_add(layout.size())?;
            if size > self.params.buf_len() - allocated { return None; }
            Some(allocated + size)
        }).map_err(|_| AllocError)?;
        let ptr = unsafe { self.buf_ptr.load(Ordering::Relaxed).add(allocated) };
        let padding = unsafe { padding.assume_init() };
        self.allocations_count.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |allocations_count|
            allocations_count.checked_add(1)
        ).map_err(|_| AllocError)?;
        let res = NonNull::slice_from_raw_parts(
            unsafe { NonNull::new_unchecked(ptr.add(padding)) },
            layout.size()
        );
        Ok(res)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: alloc::Layout) {
        let start_offset = ptr.as_ptr().offset_from(self.buf_ptr.load(Ordering::Relaxed)) as usize;
        let end_offset = start_offset + layout.size();
        let _ = self.allocated.compare_exchange(end_offset, start_offset, Ordering::Relaxed, Ordering::Relaxed);
        self.allocations_count.fetch_sub(1, Ordering::Relaxed);
    }

    unsafe fn grow(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.grow_raw(ptr, old_layout, new_layout, false)
    }

    unsafe fn grow_zeroed(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.grow_raw(ptr, old_layout, new_layout, true)
    }

    unsafe fn shrink(
        &self, 
        ptr: NonNull<u8>, 
        old_layout: alloc::Layout, 
        new_layout: alloc::Layout
    ) -> Result<NonNull<[u8]>, AllocError> {
        if new_layout.align() > old_layout.align() { return Err(AllocError); }
        let start_offset = ptr.as_ptr().offset_from(self.buf_ptr.load(Ordering::Relaxed)) as usize;
        let old_end_offset = start_offset + old_layout.size();
        let new_end_offset = start_offset + new_layout.size();
        let size = match self.allocated.compare_exchange(old_end_offset, new_end_offset, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => new_layout.size(),
            Err(_) => old_layout.size(),
        };
        Ok(NonNull::slice_from_raw_parts(ptr, size))
    }
}
