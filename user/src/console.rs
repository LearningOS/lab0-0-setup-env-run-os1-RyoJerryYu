use core::fmt::{self, Write};

use crate::{read, write};

struct Stdout;

const STDOUT: usize = 1;
const STDIN: usize = 0;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)*)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)*)?));
    };
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)*)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)*)?));
    };
}

// Read a character from the console
pub fn getchar() -> u8 {
    let mut buf = [0u8; 1];
    read(STDIN, &mut buf);
    buf[0]
}
