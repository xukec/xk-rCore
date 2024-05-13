//错误处理
use core::panic::PanicInfo;
//use crate::println; 引入当前的create的(一般不使用) == #[macro_use]

#[panic_handler] //定义一个函数作为程序遇到 panic 时调用的处理函数
fn panic_handler(panic_info: &PanicInfo) -> ! {
    if let Some(location) = panic_info.location() {
        println!(
            "Panicked at {}:{}:{}",
            location.file(),
            location.line(),
            panic_info.message().unwrap()
        );
    } else {
        println!("Panicked: {}", panic_info.message().unwrap());
    }
    loop {}
}