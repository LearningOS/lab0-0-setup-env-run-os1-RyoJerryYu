#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(panic_info_message)]

use buddy_system_allocator::LockedHeap;
use syscall::{sys_exec, sys_fork, sys_get_time, sys_getpid, sys_waitpid};

#[macro_use]
pub mod console;
mod lang_items;
mod syscall;

const USER_HEAP_SIZE: usize = 16384;

static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
    exit(main());
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

pub fn read(fd: usize, buf: &mut [u8]) -> isize {
    syscall::sys_read(fd, buf)
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
pub fn getpid() -> isize {
    sys_getpid()
}
pub fn fork() -> isize {
    sys_fork()
}
pub fn exec(path: &str) -> isize {
    sys_exec(path)
}
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => yield_(),
            pid => return pid,
        }
    }
}
pub fn waitpid(pid: isize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid, exit_code as *mut _) {
            -2 => yield_(),
            pid => return pid,
        }
    }
}
