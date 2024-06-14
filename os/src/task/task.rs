use super::TaskContext;

#[derive(Clone, Copy, PartialEq)]
pub enum TaskStatus {
    UnInit, // 未初始化
    Ready, // 准备运行
    Running, // 正在运行
    Exited, // 已退出
}

#[derive(Clone, Copy)]
pub struct TaskControlBlock {
    pub tasks_status: TaskStatus,
    pub task_cx: TaskContext,
}