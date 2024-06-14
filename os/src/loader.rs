use core::arch::asm;
use crate::trap::TrapContext;
use crate::config::*;

//声明静态数组，数组包含MAX_APP_NUM个KernelStack元素
static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack { data: [0; KERNEL_STACK_SIZE] }; MAX_APP_NUM];

static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack { data: [0; USER_STACK_SIZE] }; MAX_APP_NUM];

#[repr(align(4096))] //#[repr] 属性有一个新参数 align，用于设置结构体的对齐方式
#[derive(Clone, Copy)] //结构体或枚举正在请求Rust编译器自动为它实现 Clone 和 Copy 这两个trait。
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE] //8192个u8
}
#[repr(align(4096))]
#[derive(Clone, Copy)]
struct UserStack {
    data: [u8; USER_STACK_SIZE]
}

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    fn push_context(&self, trap_cx: TrapContext) -> usize {
        //从 get_sp 获得的地址中减去 TrapContext 的大小
        let trap_cx_prt = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        //储存
        unsafe {
            *trap_cx_prt = trap_cx;
        }
        //返回指针，表示trap_cx在栈中的地址
        trap_cx_prt as usize
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

//得到对应app的起始地址
fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

//得到app数量
pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as usize as *const usize).read_volatile() }
}

//加载
pub fn load_app() {
    extern "C" { fn _num_app(); } //link_app.S内app数据信息的位置

    let num_app_ptr = _num_app as usize as *const usize; //先转换为usize类型，在转化为指向usize的常量指针
    let num_app = get_num_app(); //从指针位置读取值 也就是app数量
    //|num_app|app0|app1|app2|app3|app4|app4end|
    //获取app0 ~ app4end
    let app_start = unsafe {
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
    };

    for i in 0..num_app {
        let base_i = get_base_i(i);
        //清零base_i ~ base_i + 0x20000
        (base_i..base_i + APP_SIZE_LIMIT).for_each(|addr| unsafe {
            (addr as *mut u8).write_volatile(0)
        });
        //在app_start中获取 app i 的待加载 app 所占的内存空间 
        //|app0|app1|app2|app3|app4|app4end|
        //如果i = 0 则获取到 |app0|
        let src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        //获取 从appi的内存起始地址开始 所占的内存空间
        let dst = unsafe {
            core::slice::from_raw_parts_mut(base_i as *mut u8, src.len())
        };
        //拷贝到dst执行代码区域
        dst.copy_from_slice(src);
        //刷新指令缓存
        unsafe {
            asm!("fence.i");
        }
    }
}

//返回cx：trap上下文的栈顶指针，通过__restore赋值给a0，进而恢复上下文，跳转到appi的入口点entry
pub fn init_app_cx(app_id: usize) -> usize {
    //println!("[kernel] spec:{:X}", get_base_i(app_id));
    KERNEL_STACK[app_id].push_context(TrapContext::app_init_context(
        get_base_i(app_id), 
        USER_STACK[app_id].get_sp()
    ))
}