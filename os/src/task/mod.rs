mod context;
mod switch;
//抑制 Clippy 的警告:同名的嵌套模块
//#[allow(clippy::module_inception)]
mod task;

use context::TaskContext;
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

    fn run_next_task(&self) {
        
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