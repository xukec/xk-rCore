use alloc::vec::Vec;
use lazy_static::*;

use crate::sync::UPSafeCell;
use crate::config::MEMORY_END;
use super::{PhysPageNum, PhysAddr};

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct StackFrameAllocator {
    current: usize, //空闲内存的起始物理页号
    end: usize,     //空闲内存的结束物理页号
    recycled: Vec<usize>,   //回收的物理页号
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        //首先查找recycled内是否有之前回收的物理页号，有的话弹出栈顶返回
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            //recycled为空且内存耗尽分配失败
            if self.current == self.end {
                None
            } else {
                self.current += 1; //+1表示当前页已经被分配
                Some((self.current -1).into())
            }
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        //check ppn之前被分配出去了就一定小于当前物理页号，且不可能被回收
        if ppn >= self.current || self.recycled.iter().find(|&v| {*v == ppn}).is_some() {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        //recycle
        self.recycled.push(ppn);
    }
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

//ref 定义一个公有的、具有静态生命周期的引用 FRAME_ALLOCATOR。
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> = unsafe {
        UPSafeCell::new(FrameAllocatorImpl::new())
    };
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR
    .exclusive_access()
    .init(PhysAddr::from(ekernel as usize).ceil(), PhysAddr::from(MEMORY_END).floor());
}

//为什么要封装？ 封装后可以为其实现 Drop Trait ，就不必手动回收物理页帧了。在编译期就解决了很多潜在的问题。
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    fn new(ppn: PhysPageNum) ->Self {
        let bytes_attay = ppn.get_bytes_array();
        //物理页号所在的空间清零
        for i in bytes_attay {
            *i = 0;
        }
        Self {
            ppn
        }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR.exclusive_access().alloc().map(|ppn| FrameTracker::new(ppn))
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}