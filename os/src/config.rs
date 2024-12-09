
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const CLOCK_FREQ: usize = 12500000;
pub const PAGE_SIZE: usize = 0x1000; // 4 KiB
pub const PAGE_SIZE_BITS: usize = 0xc; // 2^12 = 4 KiB
pub const MEMORY_END: usize = 0x8800_0000;
