#![feature(allocator_api)]
#![feature(default_alloc_error_handler)]
#![feature(iter_collect_into)]
#![feature(start)]

#![deny(warnings)]

#![no_std]

extern crate alloc;

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

mod no_std {
    use composable_allocators::{AsGlobal, System};
    use core::panic::PanicInfo;
    use exit_no_std::exit;

    #[global_allocator]
    static ALLOCATOR: AsGlobal<System> = AsGlobal(System);

    #[panic_handler]
    extern fn panic(_info: &PanicInfo) -> ! {
        exit(99)
    }

    #[cfg(all(windows))]
    #[no_mangle]
    fn rust_oom(_layout: core::alloc::Layout) -> ! {
        exit(98)
    }
}

use alloc::vec::Vec;
use composable_allocators::Global;
use composable_allocators::fallbacked::Fallbacked;
use composable_allocators::stacked::{self};

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    stacked::with_size::<256, _>(|stacked| {
        let mut vec = Vec::new_in(Fallbacked(stacked, Global));
        [0u8, 1, 2, 3].iter().copied().collect_into(&mut vec);
    });
    0
}
