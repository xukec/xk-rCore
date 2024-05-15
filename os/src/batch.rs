use lazy_static::*;
use crate::sync::UPSafeCell;
use crate::sbi::shutdown;

const MAX_APP_NUM: usize = 16;

struct AppManager {
    num_app: usize, //app数量
    current_app: usize, //当前执行的是第几个app
    app_start: [usize; MAX_APP_NUM + 1], //按照顺序放置每个应用程序的起始地址，最后一个元素放置最后一个应用程序的结束位置
}

impl AppManager {
    pub fn print_app_info(&self) {
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
        
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
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