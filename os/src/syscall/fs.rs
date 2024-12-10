use log::trace;

use crate::print;

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            trace!("sys_write: fd={}, buf={:p}, len={}", fd, buf, len);
            let buffer = unsafe { core::slice::from_raw_parts(buf, len) };
            let s = core::str::from_utf8(buffer).unwrap();
            print!("{}", s);
            len as isize
        }
        _ => {
            panic!("sys_write: not support fd: {}", fd);
        },
    }
}

