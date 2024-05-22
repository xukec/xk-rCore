mod context;

use core::arch::global_asm;

use riscv::register::{
    stvec, stval,
    scause::{self, Exception, Trap},
    mtvec::TrapMode,
};

use crate::batch::run_next_app;
use crate::syscall::syscall;
pub use context::TrapContext;

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" { fn __alltraps(); }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    //cx 是 a0 规范
    let scause = scause::read(); //描述Trap的原因
    let stval = stval::read(); //给出Trap附加信息
    //scause 寄存器所保存的 Trap 的原因进行分发处理
    match scause.cause() {
        // U 特权级的 Environment Call（系统调用）
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4; //sepc保存的app ecall 地址，+4字节。sret返回时让它在 ecall 下一条指令开始执行
            // 返回值              syscall ID  参数a0     a1        a2     //内核栈a0变化
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        //分别处理应用程序出现访存错误和非法指令错误的情形
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StoreGuestPageFault) => {
            println!("[kernel] PageFault in application, kernel killed it.");
            run_next_app();//切换并运行下一个应用程序
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            run_next_app();
        }
        //遇到目前还不支持的 Trap 类型
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}