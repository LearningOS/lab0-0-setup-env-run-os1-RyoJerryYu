mod address;
mod frame_allocator;
mod heap_allocator;
mod memory_set;
mod page_table;

pub use address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
pub use frame_allocator::{frame_alloc, frame_dealloc, FrameTracker};
pub use memory_set::MemorySet;
pub use memory_set::KERNEL_SPACE;
pub use memory_set::{kernel_token, MapPermission};
pub use page_table::{
    translated_byte_buffer, translated_ref, translated_refmut, translated_str, PageTable,
    UserBuffer,
};

use crate::println;

pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    // set the satp to kernel space
    // because the kernel space is mapped to the same physical address
    // pc can work well to jump to the kernel space
    KERNEL_SPACE.exclusive_access().activate();
    println!("++++ setup memory!     ++++");
}
