pub mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
pub use frame_allocator::{FrameTracker, frame_alloc};
pub use page_table::{PageTableEntry};

pub fn init() {
    heap_allocator::init_heap();
}