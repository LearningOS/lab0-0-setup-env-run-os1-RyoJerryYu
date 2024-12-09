mod address;
mod heap_allocator;
mod page_table;

pub fn init() {
    heap_allocator::init_heap();
}
