#![feature(allocator_api)]
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

#[doc(hidden)]
pub use core::alloc::Layout as std_alloc_Layout;
#[doc(hidden)]
pub use core::mem::MaybeUninit as std_mem_MaybeUninit;
#[doc(hidden)]
pub use core::ptr::addr_of_mut as std_ptr_addr_of_mut;

#[macro_export]
macro_rules! global_freelist_allocator_128_KiB_align_8 {
    () => {
        const MEM_SIZE: usize = 131072;

        static mut MEM: [$crate::std_mem_MaybeUninit<u8>; MEM_SIZE] =
            [$crate::std_mem_MaybeUninit::uninit(); MEM_SIZE]
        ;

        static STACKED: $crate::stacked::Stacked =
            $crate::stacked::Stacked::from_static_array(unsafe { &mut *$crate::std_ptr_addr_of_mut!(MEM) })
        ;

        type Freelist8B = $crate::AsGlobal<$crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist16B>>;

        #[global_allocator]
        static FREELIST_8_B: Freelist8B = $crate::AsGlobal($crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(8, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(8, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(1, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_16_B
        ));

        type Freelist16B = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist32B>;

        static FREELIST_16_B: Freelist16B = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(16, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(16, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(9, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_32_B
        );

        type Freelist32B = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist64B>;

        static FREELIST_32_B: Freelist32B = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(32, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(32, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(17, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_64_B
        );

        type Freelist64B = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist128B>;

        static FREELIST_64_B: Freelist64B = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(64, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(64, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(33, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_128_B
        );

        type Freelist128B = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist256B>;

        static FREELIST_128_B: Freelist128B = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(128, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(128, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(65, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_256_B
        );

        type Freelist256B = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist512B>;

        static FREELIST_256_B: Freelist256B = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(256, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(256, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(129, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_512_B
        );

        type Freelist512B = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist1KiB>;

        static FREELIST_512_B: Freelist512B = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(512, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(512, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(257, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_1_KIB
        );

        type Freelist1KiB = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist2KiB>;

        static FREELIST_1_KIB: Freelist1KiB = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(1024, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(1024, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(513, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_2_KIB
        );

        type Freelist2KiB = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist4KiB>;

        static FREELIST_2_KIB: Freelist2KiB = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(2048, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(2048, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(1025, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_4_KIB
        );

        type Freelist4KiB = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist8KiB>;

        static FREELIST_4_KIB: Freelist4KiB = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(4096, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(4096, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(2049, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_8_KIB
        );

        type Freelist8KiB = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist16KiB>;

        static FREELIST_8_KIB: Freelist8KiB = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(8192, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(8192, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(4097, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_16_KIB
        );

        type Freelist16KiB = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist32KiB>;

        static FREELIST_16_KIB: Freelist16KiB = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(16384, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(16384, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(8193, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_32_KIB
        );
    
        type Freelist32KiB = $crate::fallbacked::Fallbacked<$crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >, &'static Freelist64KiB>;

        static FREELIST_32_KIB: Freelist32KiB = $crate::fallbacked::Fallbacked(
            $crate::limited_up_to::LimitedUpTo::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(32768, 8) },
                $crate::freelist::Freelist::new(
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(32768, 8) },
                    unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(16385, 1) },
                    $crate::freelist::NoLimit,
                    &STACKED
                )
            ),
            &FREELIST_64_KIB
        );

        type Freelist64KiB = $crate::limited_up_to::LimitedUpTo<
            $crate::freelist::Freelist<$crate::freelist::NoLimit, &'static $crate::stacked::Stacked>
        >;

        static FREELIST_64_KIB: Freelist64KiB = $crate::limited_up_to::LimitedUpTo::new(
            unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(65536, 8) },
            $crate::freelist::Freelist::new(
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(65536, 8) },
                unsafe { $crate::std_alloc_Layout::from_size_align_unchecked(32769, 1) },
                $crate::freelist::NoLimit,
                &STACKED
            )
        );
    };
}
