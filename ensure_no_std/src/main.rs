#![feature(allocator_api)]
#![feature(default_alloc_error_handler)]
#![feature(explicit_generic_args_with_impl_trait)]
#![feature(iter_collect_into)]
#![feature(start)]

#![deny(warnings)]

#![no_std]

extern crate alloc;

use core::alloc::Layout;
use core::panic::PanicInfo;
#[cfg(not(windows))]
use libc::exit;
use libc_alloc::LibcAlloc;
#[cfg(windows)]
use winapi::shared::minwindef::UINT;
#[cfg(windows)]
use winapi::um::processthreadsapi::ExitProcess;

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[global_allocator]
static ALLOCATOR: LibcAlloc = LibcAlloc;

#[cfg(windows)]
unsafe fn exit(code: UINT) -> ! {
    ExitProcess(code);
    loop { }
}

#[panic_handler]
pub extern fn panic(_info: &PanicInfo) -> ! {
    unsafe { exit(99) }
}

#[no_mangle]
pub fn rust_oom(_layout: Layout) -> ! {
    unsafe { exit(98) }
}

use alloc::vec::Vec;
use composable_allocators::Global;
use composable_allocators::or::Or;
use composable_allocators::stacked::{self};

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    stacked::with_size::<256, _>(|stacked| {
        let mut vec = Vec::new_in(Or(stacked, Global));
        [0u8, 1, 2, 3].iter().copied().collect_into(&mut vec);
    });
    0
}
