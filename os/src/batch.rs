use core::arch::asm;
use crate::trap::TrapContext;
use lazy_static::*;
use crate::sync::UPSafeCell;
use crate::sbi::shutdown;

const MAX_APP_NUM: usize = 16;
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;
const USER_STACK_SIZE: usize = 4096 * 2;
const KERNEL_STACK_SIZE: usize = 4096 * 2;

static KERNEL_STACK: KernelStack = KernelStack { data: [0; KERNEL_STACK_SIZE] };
static USER_STACK: UserStack = UserStack { data: [0; USER_STACK_SIZE] };

#[repr(align(4096))]//#[repr] 属性有一个新参数 align，用于设置结构体的对齐方式
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE] //8192个u8
}
#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE]
}

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        //从 get_sp 获得的地址中减去 TrapContext 的大小
        let cx_prt = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        //储存
        unsafe {
            *cx_prt = cx;
        }
        //返回可变原始指针
        unsafe {
            cx_prt.as_mut().unwrap()
        }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

struct AppManager {
    num_app: usize, //app数量
    current_app: usize, //当前执行的是第几个app
    app_start: [usize; MAX_APP_NUM + 1], //按照顺序放置每个应用程序的起始地址，最后一个元素放置最后一个应用程序的结束位置
}

impl AppManager {
    fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i+1]
            );
        }
    }

    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            println!("All applications completed!");
            shutdown(false);
        }
        println!("[kernel] Loading app_{}", app_id);
        //清空一块内存
        //将从APP_BASE_ADDRESS开始的、长度为APP_SIZE_LIMIT的内存区域中的所有字节都设置为0
        core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0);
        //获取 app_id 的待加载 app 所占的内存空间 
        //|app0|app1|app2|app3|app4|app4end|
        //如果app_id = 0 则获取到 |app0|
        let app_src = core::slice::from_raw_parts(
            self.app_start[app_id] as *const u8, 
            self.app_start[app_id + 1] - self.app_start[app_id],
        );
        //获取 app_src(|app0|) 所占的内存空间
        let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
        //拷贝到app_dst 执行代码区域
        app_dst.copy_from_slice(app_src);
        //刷新指令缓存
        asm!("fence.i");
    }

    fn get_current_app(&self) -> usize {
        self.current_app
    }

    fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = 
    unsafe { 
        UPSafeCell::new(
            {
                extern "C" { fn _num_app(); } //link_app.S内app数据信息的位置

                let num_app_ptr = _num_app as usize as *const usize; //先转换为usize类型，在转化为指向usize的常量指针
                let num_app = num_app_ptr.read_volatile(); //从指针位置读取值 也就是app数量
                let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM +1]; //初始化数组
                //|num_app|app0|app1|app2|app3|app4|app4end|
                //获取app0 ~ app4e
                let app_start_raw: &[usize] = core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
                //拷贝复制到app_start[0~5]
                app_start[..=num_app].copy_from_slice(app_start_raw);
                
                AppManager {
                    num_app,
                    current_app: 0,
                    app_start,
                }
            }
        ) 
    };
}

pub fn init() {
    print_app_info();
}

fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    unsafe {
        app_manager.load_app(current_app);
    }
    app_manager.move_to_next_app();
    drop(app_manager);

    extern "C" { fn __restore(cx_addr: usize); }
    unsafe {
        __restore(KERNEL_STACK.push_context(
            TrapContext::app_init_context(APP_BASE_ADDRESS, USER_STACK.get_sp())
        ) as *const _ as usize);
    }
    //不可能
    panic!("Unreachable in batch::run_current_app!");
}