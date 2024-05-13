#![no_std]
#![feature(panic_info_message)]
#![feature(linkage)] //支持下面的链接操作

use syscall::{sys_exit, sys_write};

#[macro_use] //外部的crate，想要使用console这个crate提供的宏时
pub mod console;
mod syscall;
mod lang_items;

#[no_mangle]
#[link_section = ".text.entry"] //将_start编译后的汇编代码放到名为.text.entry的代码段中。方便后续链接的时候调整它的位置使得它能够作为用户库的入口。
pub extern "C" fn _start() -> ! {
    clear_bss(); //手动清空需要零初始化的 .bss 段
    exit(main()); 
    panic!("unreachable after sys_exit!");
}

/*
在最后链接的时候，虽然在 lib.rs 和 bin 目录下的某个应用程序都有 main 符号，
但由于 lib.rs 中的 main 符号是弱链接，链接器会使用 bin 目录下的应用主逻辑作为 main 。
这里主要是进行某种程度上的保护，如果在 bin 目录下找不到任何 main ，那么编译也能够通过，但会在运行时报错。
*/
#[linkage = "weak"] //将其函数符号 main 标志为弱链接
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

fn clear_bss() {
    extern "C" {
        fn start_bss();
        fn end_bss();
    }
    (start_bss as usize..end_bss as usize).for_each(|addr| unsafe {
        (addr as *mut u8).write_volatile(0);
    })
}

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}