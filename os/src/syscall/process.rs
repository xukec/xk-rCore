use crate::task::suspend_current_and_run_next;
use crate::task::exit_current_and_run_next;

pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next(); //退出当前的应用并切换到下个应用
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next(); //暂停当前的应用并切换到下个应用。
    0
}