use crate::{
    mm::translated_byte_buffer,
    print,
    sbi::console_getchar,
    task::{current_user_token, suspend_current_and_run_next},
};

use super::File;

pub struct Stdin;
pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        false
    }

    fn read(&self, mut buf: crate::mm::UserBuffer) -> usize {
        assert_eq!(buf.len(), 1);
        let mut c: usize;
        loop {
            c = console_getchar();
            if c == 0 {
                suspend_current_and_run_next();
                continue;
            } else {
                break;
            }
        }
        let ch = c as u8;
        unsafe {
            buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        1
    }

    fn write(&self, buf: crate::mm::UserBuffer) -> usize {
        panic!("Stdin is not writable!");
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        true
    }

    fn read(&self, buf: crate::mm::UserBuffer) -> usize {
        panic!("Stdout is not readable!");
    }

    fn write(&self, buf: crate::mm::UserBuffer) -> usize {
        for buffer in buf.buffers.iter() {
            print!("{}", core::str::from_utf8(buffer).unwrap());
        }
        buf.len()
    }
}
