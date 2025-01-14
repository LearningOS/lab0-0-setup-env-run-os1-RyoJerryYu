mod inode;
mod stdio;
use crate::mm::UserBuffer;

pub use inode::{open_file, OpenFlags};
pub use stdio::{Stdin, Stdout};

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}
