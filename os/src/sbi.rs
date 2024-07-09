//sbi提供的输出一个字符api
pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}

//sbi提供的关机
pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}

//sbi提供的用来设置 mtimecmp 的值的api
pub fn set_mtimecmp(mtimecmp: usize) {
    sbi_rt::set_timer(mtimecmp as _);
}