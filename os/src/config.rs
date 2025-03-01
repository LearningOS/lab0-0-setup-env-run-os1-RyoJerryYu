pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const CLOCK_FREQ: usize = 12500000;
pub const PAGE_SIZE: usize = 0x1000; // 4 KiB
pub const PAGE_SIZE_BITS: usize = 0xc; // 2^12 = 4 KiB
pub const MEMORY_END: usize = 0x8800_0000;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE; // trap.S trap handler context, is const in virtual memory

pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
];
