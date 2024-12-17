#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(panic_info_message)]

use syscall::sys_get_time;

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    exit(main());
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    syscall::sys_write(fd, buf)
}
pub fn exit(code: i32) -> ! {
    syscall::sys_exit(code as usize)
}
pub fn yield_() {
    syscall::sys_yield()
}
pub fn get_time() -> isize {
    sys_get_time()
}
