//!当 mtime > mtimecmp 就会触发一次时钟中断

use riscv::register::time;
use crate::sbi::set_mtimecmp;
use crate::config::CLOCK_FREQ;

const TICKS_PER_SEC: usize = 100;
const MICRO_PER_SEC: usize = 1000;

// 获取当前 mtime 计数器的值。作用 :用来统计处理器自上电以来经过了多少个内置时钟的时钟周期
pub fn get_mtime() -> usize {
    time::read()
}

//CLOCK_FREQ / TICKS_PER_SEC 是10ms之内计数器的增量
//CLOCK_FREQ是平台的时钟频率（hz），也是在1s内计数器增加的增量。1s=1000ms。
//想增加10ms 就要将CLOCK_FREQ/100即可
//10ms 之后一个 S 特权级时钟中断就会被触发。
pub fn set_next_trigger() {
    set_mtimecmp(get_mtime() + CLOCK_FREQ / TICKS_PER_SEC);
}

//以ms为单位返回当前计数器的值。
//CLOCK_FREQ / MICRO_PER_SEC 即1ms
pub fn get_time_ms() -> usize {
    get_mtime() / (CLOCK_FREQ / MICRO_PER_SEC)
}