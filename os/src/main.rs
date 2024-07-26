#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate bitflags;

use core::arch::global_asm;
use log::*;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod logging;
//mod batch;
mod sync;
mod trap;
mod syscall;
mod config;
mod loader;
mod task;
mod timer;
mod mm;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[no_mangle]
pub fn rust_main() -> ! {
    //链接器提供
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
    }

    clear_bss();

    logging::init();

    println!("[kernel] Hello, world!");

    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize, etext as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    error!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);

    //panic!("Shutdown machine!");
    //sbi::shutdown(false);
    mm::init();
    mm::heap_allocator::heap_test();
    trap::init();

    //batch::init();
    loader::load_app();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    //batch::run_next_app();
    panic!("Unreachable in rust_main!");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0) }
    });
}