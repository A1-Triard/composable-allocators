#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(maybe_uninit_uninit_array)]
#![feature(never_type)]
#![cfg_attr(feature="logging", feature(panic_abort))]
#![feature(slice_ptr_get)]
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

extern crate alloc;

mod base;
pub use base::*;

pub mod fallbacked;

pub mod limited_up_to;

mod global;
pub use global::*;

mod as_global;
pub use as_global::*;

#[cfg(feature="logging")]
mod logging;
#[cfg(feature="logging")]
pub use logging::*;

#[cfg(all(not(target_os="dos"), windows))]
mod winapi;

#[cfg(all(not(target_os="dos"), not(windows)))]
mod posix;

#[cfg(not(target_os="dos"))]
mod system;

#[cfg(not(target_os="dos"))]
pub use system::*;

mod impossible;
pub use impossible::*;

mod non_working;
pub use non_working::*;

pub mod stacked;

pub mod freelist;
