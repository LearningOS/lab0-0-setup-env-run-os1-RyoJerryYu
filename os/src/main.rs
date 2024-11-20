#![no_std]
#![no_main]
mod lang_items;
mod sbi;
mod logging;
#[macro_use]
mod console;

use core::arch::global_asm;

use log::{info, trace};

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    println!("\x1b[31mHello, RISC-V!\x1b[0m");

    trace!("Hello, RISC-V!");
    info!("Hello, RISC-V!");
    panic!("oops!");
    // sbi::shutdown(false);
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe {
        (a as *mut u8).write_volatile(0);
    });
}
