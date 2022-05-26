#![feature(allocator_api)]
#![feature(const_trait_impl)]
#![feature(never_type)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(slice_ptr_get)]

#![deny(warnings)]

#![no_std]

#[cfg(feature="alloc")]
extern crate alloc as alloc_crate;

mod base;
pub use base::*;

#[cfg(feature="alloc")]
mod global;
#[cfg(feature="alloc")]
pub use global::*;

mod impossible;
pub use impossible::*;

mod non_working;
pub use non_working::*;

mod stacked;
pub use stacked::*;

mod freelist;
pub use freelist::*;
