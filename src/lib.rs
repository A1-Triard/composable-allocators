//! **Crate features**
//!
//! * `"alloc"`
//! Enabled by default. Disable to unlink `alloc` crate.

#![feature(allocator_api)]
#![feature(const_trait_impl)]
#![feature(never_type)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(raw_ref_op)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
#![feature(strict_provenance)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::collapsible_else_if)]

#![no_std]

#[cfg(feature="global")]
extern crate alloc;

mod base;
pub use base::*;

pub mod fallbacked;

pub mod limited_up_to;

#[cfg(feature="global")]
mod global;
#[cfg(feature="global")]
pub use global::*;

mod as_global;
pub use as_global::*;

#[cfg(all(windows, any(feature="winapi", feature="system")))]
mod winapi;
#[cfg(all(windows, feature="winapi"))]
pub use crate::winapi::*;

#[cfg(all(not(windows), any(feature="posix", feature="system")))]
mod posix;
#[cfg(all(not(windows), feature="posix"))]
pub use posix::*;

#[cfg(feature="system")]
mod system;
#[cfg(feature="system")]
pub use system::*;

mod impossible;
pub use impossible::*;

mod non_working;
pub use non_working::*;

pub mod stacked;

pub mod freelist;
