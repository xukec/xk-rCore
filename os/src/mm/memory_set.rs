use crate::config::{MEMORY_END, PAGE_SIZE, USER_STACK_SIZE, TRAMPOLINE, TRAP_CONTEXT};

use super::{frame_alloc, PTEFlags, FrameTracker, PageTable, PhysPageNum, VPNRange, VirtAddr, VirtPageNum, StepByOne};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use bitflags::bitflags;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical, //恒等映射
    Framed, //每个虚拟页面都有一个新分配的物理页帧与之对应，虚地址与物理地址的映射关系是相对随机的。
}

bitflags! {
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

pub struct MapArea {
    vpn_range: VPNRange, //描述一段虚拟页号的连续区间，表示该逻辑段在地址区间中的位置和长度。
    data_frames: BTreeMap<VirtPageNum, FrameTracker>, //保存了该逻辑段内的每个虚拟页面和它被映射到的物理页帧 FrameTracker 的一个键值对容器 BTreeMap 中
    map_type: MapType, //逻辑段内的所有虚拟页面映射到物理页帧的方式
    map_perm: MapPermission, //控制该逻辑段的访问方式
}

impl MapArea {
    pub fn new(start_va: VirtAddr, end_va: VirtAddr, map_type: MapType, map_perm: MapPermission) -> Self {
        let start_vpn: VirtPageNum = start_va.floor(); //向下取整，
        let end_vpn: VirtPageNum = end_va.ceil(); //向上取整
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
        }
    }
    //将单个虚拟页号 vpn 映射到一个物理页号 ppn，并将映射关系添加到页表中。
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum;
        match self.map_type {
            MapType::Identical => {
                ppn = PhysPageNum(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }
        //将 MapPermission 中的权限标志位转换为 PTEFlags 类型，这样这些权限就可以正确应用于页表项。
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }

    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        match self.map_type {
            MapType::Framed => {
                self.data_frames.remove(&vpn);
            }
            _ => {}
        }
        page_table.unmap(vpn);
    }

    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.map_one(page_table, vpn);
        }
    }

    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }
    //将给定的数据（切片 data）逐页拷贝到当前逻辑段（MapArea）对应的物理页帧中。
    //切片 data 中的数据大小不超过当前逻辑段的总大小，且切片中的数据会被对齐到逻辑段的开头，然后逐页拷贝到实际的物理页帧。
    pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        //确保虚拟页号和物理页帧之间的映射是非恒等映射
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0; //追踪在数据切片 data 中当前拷贝的起始位置。
        let mut current_vpn = self.vpn_range.get_start(); //初始值为逻辑段的起始虚拟页号。
        let len = data.len(); //切片的长度
        //循环逐页拷贝数据
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)]; //len.min(...) 用于确保不超出 data 的总长度
            let dst = &mut page_table.translate(current_vpn).unwrap().ppn().get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
    }
}

pub struct MemorySet {
    page_table: PageTable, //地址空间多级页表
    areas: Vec<MapArea>, //逻辑段向量
}

impl MemorySet {
    //新建一个空的地址空间
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }
    //在当前地址空间插入一个新的逻辑段
    fn push(&mut self, mut map_area: MapArea, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&self.page_table, data);
        }
        self.areas.push(map_area);
    }
    //在当前地址空间插入一个 Framed 方式映射到物理内存的逻辑段。
    //注意该方法的调用者要保证同一地址空间内的任意两个逻辑段不能存在交集
    pub fn insert_framed_area(&mut self, start_va: VirtAddr, end_va: VirtAddr, permission: MapPermission) {
        self.push(MapArea::new(start_va, end_va, MapType::Framed, permission), None);

    }
    //生成内核的地址空间
    pub fn new_kernel() -> Self {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline();
        // map kernel sections
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(".bss [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);
        
        println!("mapping .text section");
        memory_set.push(MapArea::new(
            (stext as usize).into(),
            (etext as usize).into(), 
            MapType::Identical, 
            MapPermission::R | MapPermission::X
        ), None);
        println!("mapping .rodata section");
        memory_set.push(MapArea::new(
            (srodata as usize).into(),
            (erodata as usize).into(),
            MapType::Identical,
            MapPermission::R,
        ), None);
        println!("mapping .data section");
        memory_set.push(MapArea::new(
            (sdata as usize).into(),
            (edata as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ), None);
        println!("mapping .bss section");
        memory_set.push(MapArea::new(
            (sbss_with_stack as usize).into(),
            (ebss as usize).into(),
            MapType::Identical,
            MapPermission::R | MapPermission::W,
        ), None);
        println!("mapping physical memory");
        memory_set.push(MapArea::new(
            (ekernel as usize).into(), 
            MEMORY_END.into(), 
            MapType::Identical, 
            MapPermission::R | MapPermission::W
        ), None);
        memory_set
    }
    //分析应用的 ELF 文件格式的内容，解析出各数据段并生成对应的地址空间。
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        //包含elf、trampoline、TrapContext和user stack中的部分，
        //同样返回user_sp和入口点
        let mut memory_set = Self::new_bare();
        // 将 trampoline跳板 映射到地址空间中。Trampoline 是一种特殊的代码段，通常用于处理用户态和内核态之间的切换。
        memory_set.map_trampoline();
        // 映射elf的程序头，标记为U。使用外部 crate xmas_elf 来解析传入的应用 ELF 数据并可以轻松取出各个部分。
        // rust-readobj -all target/debug/os 看看 ELF 文件中究竟包含什么内容
        // 魔数 Magic: (7F 45 4C 46)独特的常数，存放在 ELF header 的一个固定位置。当加载器将 ELF 文件加载到内存之前，通常会查看 该位置的值是否正确，来快速确认被加载的文件是不是一个 ELF 。
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        //直接得到 program header 的数目
        /*
        除了 ELF header 之外，还有另外两种不同的 header，分
        别称为 program header 和 section header， 
        它们都有多个。ELF header 中给出了其他两种header 的大小、在文件中的位置以及数目。
        Entry: 0x5070
        ProgramHeaderOffset: 0x40 //从文件的 0x40 字节偏移处开始
        SectionHeaderOffset: 0x32D8D0
        Flags [ (0x0)
        ]
        HeaderSize: 64
        ProgramHeaderEntrySize: 56 //每个 56 字节；
        ProgramHeaderCount: 12 //12 个不同的 program header //从文件的 0x40 字节偏移处开始 每个 56 字节；
        SectionHeaderEntrySize: 64
        SectionHeaderCount: 42
        StringTableSectionIndex: 41
        }
        ......
        */
        let ph_count = elf_header.pt2.ph_count();
        let mut max_end_vpn = VirtPageNum(0);
        // 遍历所有的 program header 并将合适的区域加入到应用地址空间中。
        for i in 0..ph_count {
            let ph = elf.program_header(i).unwrap();
            // 确认 program header 的类型是 LOAD ，这表明它有被内核加载的必要
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                //通过 ph.virtual_addr() 和 ph.mem_size() 来计算这一区域在应用地址空间中的位置
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                //通过 ph.flags() 来确认这一区域访问方式的限制并将其转换为 MapPermission 类型（注意它默认包含 U 标志位）
                let mut map_perm = MapPermission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MapPermission::R;
                }
                if ph_flags.is_write() { 
                    map_perm |= MapPermission::W; 
                }
                if ph_flags.is_execute() { 
                    map_perm |= MapPermission::X; 
                }
                //创建逻辑段 map_area 
                let map_area = MapArea::new(start_va, end_va, MapType::Framed, map_perm);
                max_end_vpn = map_area.vpn_range.get_end();
                // push 到应用地址空间
                //在 push 的时候我们需要完成数据拷贝，当前 program header 数据被存放的位置可以通过 ph.offset() 和 ph.file_size() 来找到。
                //注意当存在一部分零初始化的时候， ph.file_size() 将会小于 ph.mem_size() ，因为这些零出于缩减可执行文件大小的原因不应该实际出现在 ELF 数据中。
                memory_set.push(map_area, Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]));
            }
        }
        // 映射用户栈 max_end_vpn 记录目前涉及到的最大的虚拟页号
        let max_end_va: VirtAddr = max_end_vpn.into();
        //用户栈底虚拟地址
        let mut user_stack_bottom: usize = max_end_va.into();
        // 在max_end_vpn上面再放置一个保护页面和用户栈即可。
        //保护页面guard page
        user_stack_bottom += PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(MapArea::new(
            user_stack_bottom.into(),
            user_stack_top.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W | MapPermission::U,
        ), None);
        // map TrapContext
        //在应用地址空间中映射次高页面来存放 Trap 上下文。
        memory_set.push(MapArea::new(
            TRAP_CONTEXT.into(),
            TRAMPOLINE.into(),
            MapType::Framed,
            MapPermission::R | MapPermission::W,
        ), None);
        //不仅返回应用地址空间 memory_set ，也同时返回用户栈虚拟地址 user_stack_top 以及从解析 ELF 得到的该应用入口点地址，它们将被我们用来创建应用的任务控制块。
        (memory_set, user_stack_top, elf.header.pt2.entry_point() as usize)
    }
}