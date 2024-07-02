mod context;
mod switch;
//抑制 Clippy 的警告:同名的嵌套模块
//#[allow(clippy::module_inception)]
mod task;

use context::TaskContext;
use switch::__switch;
use crate::loader::{get_num_app, init_app_cx};
use crate::{config::MAX_APP_NUM, sync::UPSafeCell};
use lazy_static::*;
use task::{TaskControlBlock, TaskStatus};

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
} 

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

impl TaskManager {
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access(); //获取可变引用
        let current = inner.current_task;
        inner.tasks[current].tasks_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].tasks_status = TaskStatus::Exited;
    }

    //寻找一个运行状态为 Ready 的应用并返回其 ID
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access(); //可变借用
        let current = inner.current_task;
        /*
        举例：
        current = 3;
        num_app = 5;
        (current + 1..current + self.num_app + 1) 即 [4,9) 即 4, 5, 6, 7, 8
        对 num_app 取模得到下标 [4 % 5, 5 % 5, 6 % 5, 7 % 5, 8 % 5] = [4, 0, 1, 2, 3]
        
        find 方法返回一个 Option<usize> 类型的值：
        如果找到了满足条件的 id，则返回 Some(id)。如果没有找到任何满足条件的 id，则返回 None。
        */
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| {
                inner.tasks[*id].tasks_status == TaskStatus::Ready
            })
    }

    //运行下一个任务
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access(); //获取独占访问权
            let current = inner.current_task;
            //更新下一个任务的状态和索引
            inner.tasks[next].tasks_status = TaskStatus::Running;
            inner.current_task = next;
            //获取当前和下一个任务的上下文指针
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            //提前释放独占访问权，防止后续代码中再借用 inner
            //需要手动 drop 掉我们获取到的 TaskManagerInner 的来自 UPSafeCell 的借用标记
            drop(inner); 
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            //跳转到用户态
        } else {
            panic!("All applications completed!");
        }
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app(); //获取app数量
        //任务数组，包含app数量个任务控制块，进行初始化
        let mut tasks = [
            TaskControlBlock {
                task_cx: TaskContext::zero_init(), //任务上下文初始化为0
                tasks_status: TaskStatus::UnInit, //任务状态为未初始化
            }; 
            MAX_APP_NUM
        ];
        //循环赋值
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::goto_restore(init_app_cx(i)); //任务上下文赋值
            task.tasks_status = TaskStatus::Ready; //将任务设为准备状态
        }
        //创建 TaskManager 实例
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended(); //使当前任务暂停
    run_next_task(); //尝试切换到下一个应用
}

pub fn exit_current_and_run_next() {
    mark_current_exited(); //使当前任务退出
    run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended(); //修改当前应用的运行状态
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}