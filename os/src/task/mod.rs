mod context;
mod switch;
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