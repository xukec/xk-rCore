pub mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};

pub fn init() {
    heap_allocator::init_heap();
}