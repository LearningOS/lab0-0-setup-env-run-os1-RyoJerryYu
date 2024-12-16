mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
pub use memory_set::remap_test;
pub use memory_set::MapPermission;
pub use memory_set::MemorySet;
pub use memory_set::KERNEL_SPACE;
pub use page_table::translated_byte_buffer;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    // set the satp to kernel space
    // because the kernel space is mapped to the same physical address
    // pc can work well to jump to the kernel space
    KERNEL_SPACE.exclusive_access().activate();
}
