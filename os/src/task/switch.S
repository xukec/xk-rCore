.altmacro # 加上才能正常使用.rept命令
# 保存循环体
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)    #\n替换n参数
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm    
    .section .text
    .globl __switch
__switch:
    # 阶段1
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )
    # 阶段2
    # save kernel stack of current task
    # 数据传输指令，存双字。将sp中的数据储存到a0 上移 8字节
    # 保存通用寄存器 保存到[a0+8,a0+16) 公式[a0+8n,a0+8(n+1))
    sd sp, 8(a0)
    # 保存当前执行的 ra 和 s0~s11
    # ra 保存到[a0,a0+8)
    sd ra, 0(a0)
    # 保存s0-s11
    .set n, 0   #设置n变量为0
    .rept 12    #0~11 循环12次
        SAVE_SN %n  #传递n
        .set n, n+1 #n自加1 
    .endr
    # 阶段3
    # 恢复下次执行的 ra 和 s0~s11
    ld ra, 0(a1)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # 恢复下一个任务的内核栈
    ld sp, 8(a1)
    # 阶段4
    ret