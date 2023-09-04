//! **Crate features**
//!
//! * `"alloc"`
//! Enabled by default. Disable to unlink `alloc` crate.

#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(effects)]
#![feature(maybe_uninit_uninit_array)]
#![feature(never_type)]
#![cfg_attr(feature="logging", feature(panic_abort))]
#![feature(raw_ref_op)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
#![feature(strict_provenance)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::collapsible_else_if)]

#![cfg_attr(not(feature="logging"), no_std)]

#[cfg(feature="logging")]
extern crate core;

#[cfg(feature="logging")]
extern crate panic_abort;

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

#[cfg(feature="logging")]
mod logging;
#[cfg(feature="logging")]
pub use logging::*;

#[cfg(all(not(target_os="dos"), windows, any(feature="winapi", feature="system")))]
mod winapi;
#[cfg(all(not(target_os="dos"), windows, feature="winapi"))]
pub use crate::winapi::*;

#[cfg(all(not(target_os="dos"), not(windows), any(feature="posix", feature="system")))]
mod posix;
#[cfg(all(not(target_os="dos"), not(windows), feature="posix"))]
pub use posix::*;

#[cfg(all(not(target_os="dos"), feature="system"))]
mod system;
#[cfg(all(not(target_os="dos"), feature="system"))]
pub use system::*;

mod impossible;
pub use impossible::*;

mod non_working;
pub use non_working::*;

pub mod stacked;

pub mod freelist;
