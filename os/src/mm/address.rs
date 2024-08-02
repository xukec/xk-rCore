use crate::config::{PAGE_SIZE_BITS, PAGE_SIZE}; //Page Offset(12位，4kib)


//Physical Address (56bits) [Physical Page Number (PPN 44bits), Page Offset (12bits)]
const PA_WIDTH_SV39: usize = 56;
//Virtual Address (39bits) [Virtual Page Number (VPN 27bits), Page Offset (12bits)]
const VA_WIDTH_SV39: usize = 39;
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

/// T: {PhysAddr, VirtAddr, PhysPageNum, VirtPageNum}
/// T -> usize: T.0
/// usize -> T: usize.into()

///通过 usize 类型的值来创建 {PhysAddr, VirtAddr, PhysPageNum, VirtPageNum} 实例

impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self {
        Self(value & ( (1 << PA_WIDTH_SV39) - 1)) //截断到56位，只保留低 56 位的值 如：64位0x1234_5678_9abc_def0 变为0x34_5678_9abc_def0
    }
}

impl From<usize> for PhysPageNum {
    fn from(value: usize) -> Self {
        Self(value & ( (1 << PPN_WIDTH_SV39) - 1)) //截断到44位，只保留低 44 位的值 如：64位0x1234_5678_9abc_def0 变为0x678_9abc_def0
    }
}

impl From<usize> for VirtAddr {
    fn from(value: usize) -> Self {
        Self(value & ( (1 << VA_WIDTH_SV39) - 1)) //截断到39位，只保留低 39 位的值
    }
}

impl From<usize> for VirtPageNum {
    fn from(value: usize) -> Self {
        Self(value & ( (1 << VPN_WIDTH_SV39) - 1)) //截断到27位，只保留低 27 位的值 
    }
}

///转换为 usize 类型

impl From<PhysAddr> for usize {
    fn from(value: PhysAddr) -> Self {
        value.0 //从 PhysAddr 结构体中获取其字段 0 的值，并将其作为 usize 返回
    }
}

impl From<PhysPageNum> for usize {
    fn from(value: PhysPageNum) -> Self {
        value.0
    }
}

//为什么虚拟地址用判断？ 
/*
虚拟地址可能包含负值，这是由于虚拟地址空间的高位作为符号位来区分用户空间和内核空间。
SV39 模式下的虚拟地址是 39 位宽的，这意味着第 38 位是符号位。如果符号位为 1，需要对其进行符号扩展，以确保地址在高位上正确填充符号位的值。

物理地址和物理页号在内存管理中没有符号位的概念，它们总是非负数，并且直接映射到实际的物理内存位置。因此，在转换物理地址和物理页号时，不需要进行符号扩展。

虚拟页号通常是从虚拟地址中提取出来的，且不会直接涉及符号扩展。虚拟页号只是地址的一部分，通常用于页表索引，不直接用于地址计算，因此也不需要进行符号扩展。
*/
impl From<VirtAddr> for usize {
    fn from(value: VirtAddr) -> Self {
        //判断符号位是否为1       1和38个0
        if value.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
            //如果是1 为负数 进行符号拓展
            value.0 | (!((1 << VA_WIDTH_SV39) - 1))    //或上全1和39个0，即将虚拟地址的高位填充为符号位的值，以保持正确的负数表示。
        } else {
            //如果为0 原值返回
            value.0
        }
    }
}

impl From<VirtPageNum> for usize {
    fn from(value: VirtPageNum) -> Self {
        value.0
    }
}

impl PhysAddr {
    //从物理地址中提取页内偏移量
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1) //1111 1111 1111 如 0x1234_5678_9abc_def0 -> 0xef0
    }
    //计算当前物理地址所属的物理页面号，表示该地址在物理内存中的哪个页面。(下取整) 2.4 -> 2
    pub fn floor(&self) -> PhysPageNum { PhysPageNum(self.0 / PAGE_SIZE) }
    //(向上取整) 2.4 -> 3
    pub fn ceil(&self) -> PhysPageNum { PhysPageNum((self.0 + (PAGE_SIZE - 1)) / PAGE_SIZE) }
    //对齐
    pub fn aligned(&self) -> bool { self.page_offset() == 0 }
}

impl PhysPageNum {
    //获取一个指向物理页框内存的可变字节数组
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        //从物理地址创建一个指向内存的可变切片
        unsafe {
            core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096)
        }
    }
}

//通过物理地址获得物理页面号
impl From<PhysAddr> for PhysPageNum {
    fn from(value: PhysAddr) -> Self {
        assert_eq!(value.page_offset(), 0);//返回物理地址的页面内偏移量。只有当偏移量为0时，地址才是页面对齐的。
        value.floor()
    }
}

//通过物理页面号获得物理地址，乘以页面大小（即将页号转换为起始地址）
impl From<PhysPageNum> for PhysAddr {
    fn from(value: PhysPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BITS)
    }
}

impl VirtAddr {
    //从虚拟地址中提取页内偏移量
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1) //1111 1111 1111 如 0x1234_5678_9abc_def0 -> 0xef0
    }
    //计算当前虚拟地址所属的页面号，表示该地址在虚拟内存中的哪个页面。(下取整)
    pub fn floor(&self) -> VirtPageNum { VirtPageNum(self.0 / PAGE_SIZE) }
    //(上取整)
    pub fn ceil(&self) -> VirtPageNum { VirtPageNum((self.0 + (PAGE_SIZE - 1)) / PAGE_SIZE) }
    //对齐
    pub fn aligned(&self) -> bool { self.page_offset() == 0 }
}

//通过虚拟地址获得虚拟页面号
impl From<VirtAddr> for VirtPageNum {
    fn from(value: VirtAddr) -> Self {
        assert_eq!(value.page_offset(), 0);//返回物虚拟地址的页面内偏移量。只有当偏移量为0时，地址才是页面对齐的。
        value.floor()
    }
}

//通过虚拟页面号获得虚拟地址，乘以页面大小（即将页号转换为起始地址）
impl From<VirtPageNum> for VirtAddr {
    fn from(value: VirtPageNum) -> Self {
        Self(value.0 << PAGE_SIZE_BITS)
    }
}