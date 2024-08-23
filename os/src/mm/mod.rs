pub mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;
mod memory_set;

pub use address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum, VPNRange, StepByOne};
pub use frame_allocator::{FrameTracker, frame_alloc};
pub use page_table::{PageTableEntry, PageTable, PTEFlags};

pub fn init() {
    heap_allocator::init_heap();
}