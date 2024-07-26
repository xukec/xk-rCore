use bitflags::*;

use super::address::PhysPageNum;

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    //获取物理页号
    pub fn ppn(&self) -> PhysPageNum {
        ((self.bits >> 10) & ((1usize << 44) -1)).into() //去掉后10位，截取44位 获得物理页号 在转换为PhysPageNum类型
    }
    //获取后8位 得到PTEFlags类型
    pub fn flags(&self) -> PTEFlags {
        //from_bits 是一个用于从位数值生成 PTEFlags 实例的方法。它接受一个 u8 值，并根据这个值返回一个 Option<PTEFlags>。如果传入的位值包含无效的标志，则返回 None。
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    //通过物理页号和页表标志位新建页表项
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits as usize, //.bits可以访问整个PTEFlags的值
        }
    }
    //生成一个全零的页表项，注：（页表项的 V 标志位为 0 ，因此它是不合法的）
    pub fn empty() -> Self {
        PageTableEntry{
            bits: 0,
        }
    }
    //快速判断一个页表项的 V/R/W/X 标志位是否为 1
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty() 
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty() 
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty() 
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty() 
    }
}
