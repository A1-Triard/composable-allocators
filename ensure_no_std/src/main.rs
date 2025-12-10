#![feature(allocator_api)]
#![feature(iter_collect_into)]

#![deny(warnings)]

#![no_std]
#![no_main]

extern crate alloc;

#[cfg(windows)]
#[link(name="msvcrt")]
extern "C" { }

mod no_std {
    use composable_allocators::{AsGlobal, System};
    use core::panic::PanicInfo;
    use exit_no_std::exit;

    #[global_allocator]
    static ALLOCATOR: AsGlobal<System> = AsGlobal(System);

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        exit(99)
    }
}

use alloc::vec::Vec;
use composable_allocators::Global;
use composable_allocators::fallbacked::Fallbacked;
use composable_allocators::stacked::{self};
use core::ffi::{c_char, c_int};

#[unsafe(no_mangle)]
extern "C" fn main(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
    stacked::with_size::<256, _>(|stacked| {
        let mut vec = Vec::new_in(Fallbacked(stacked, Global));
        [0u8, 1, 2, 3].iter().copied().collect_into(&mut vec);
    });
    0
}
