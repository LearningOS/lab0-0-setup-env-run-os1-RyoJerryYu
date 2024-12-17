pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const CLOCK_FREQ: usize = 12500000;
pub const PAGE_SIZE: usize = 0x1000; // 4 KiB
pub const PAGE_SIZE_BITS: usize = 0xc; // 2^12 = 4 KiB
pub const MEMORY_END: usize = 0x8800_0000;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE; // trap.S trap handler context, is const in virtual memory

/// Return (bottom, top) of a kernel stack in kernel space.
/// kernel stack is stored in the higher address of the kernel space.
/// from trampoline, each app have same kernel stack size,
/// and each of them have a page size space to avoid overlap.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
