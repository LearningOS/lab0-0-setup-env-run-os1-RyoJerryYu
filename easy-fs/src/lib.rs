#![no_std]
mod bitmap;
mod block_cache;
mod block_dev;
mod efs;
mod layout;
mod vfs;

extern crate alloc;

// 512 bytes per block
pub const BLOCK_SZ: usize = 512;
