.altmacro #加上才能正常使用.rept命令
#保存循环体
.macro SACE_GP n
    sd x\n, \n*8(sp)    #\n替换n参数
.endm

.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

    .section .text
    .globl __alltraps
    .globl __restore
    .align 2    #.align integer 2的integer次方个字节对齐 这里是将 __alltraps 的地址4字节对齐。（RISC-V 特权级规范的要求）
__alltraps:
    #csrrw rd, csr, rs 将CSR当前的值读到通用寄存器rd中，然后将通用寄存器rs的值写入CSR。
    #这里是交换 sp 和 sscratch 未交换前 sp -> user stack, sscratch -> kernel stack
    csrrw sp, sscratch, sp  # 执行完指令 sp -> kernel stack， sscratch -> user stack
    
    #addi rd, rs1, imm 功能是把一个寄存器的值和一个12位的有符号的立即数相加，并把结果存入另一个寄存器。
    #rd是目标寄存器，rs1是源寄存器，imm是12位的立即数。 这里是将 sp 自加 -34*8
    addi sp, sp, -34*8 #在内核栈上保存 Trap 上下文，预先分配34*8字节的栈帧。地址区间[sp,sp+8*34)

    #x0 被硬编码为 0 ，不会有变化；tp(x4) 寄存器，除非我们手动出于一些特殊用途使用它，否则一般也不会被用到
    #数据传输指令，存双字。将x1中的数据储存到sp 上移 8字节
    sd x1, 1*8(sp)  #保存通用寄存器 保存到[sp+8,sp+16) 公式[sp+8n,sp+8(n+1))
    # skip sp(x2), we will save it later .不保存 sp(x2)要基于它来找到每个寄存器应该被保存到的正确的位置
    sd x3, 3*8(sp)

    #保存x5-x31
    .set n, 5   #设置n变量为5
    .rept 27    #5~31 循环27次
        SACE_GP %n  #传递n
        .set n, n+1 #n自加1 
    .endr

    #将 CSR sstatus 和 sepc 的值分别读到寄存器 t0 和 t1 中然后保存到内核栈对应的位置上
    #可以自由地使用t0/t1/t2，不用担心被覆盖，因为它们在上面已经被保存在内核栈上了
    #csrr rd, csr 功能将 CSR 的值读到寄存器rd中
    csrr t0, sstatus #之前特权级
    csrr t1, sepc #记录trap发生前最后一条指令地址
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)

    #从sscratch中读用户栈地址到寄存器t2中，保存它到内核栈
    csrr t2, sscratch
    sd t2, 2*8(sp)

    #设置trap_handler的输入参数(cx: &mut TrapContext)
    #使寄存器 a0 指向 内核栈的栈指针也就是我们刚刚保存的 Trap 上下文的地址
    
    #原因：接下来要调用 trap_handler 进行 Trap 处理，它的第一个参数 cx 由调用规范要从 a0 中获取。
    #而 Trap 处理函数 trap_handler 需要 Trap 上下文的原因在于：它需要知道其中某些寄存器的值，
    #比如在系统调用的时候应用程序传过来的 syscall ID 和对应参数。我们不能直接使用这些寄存器现在的值，
    #因为它们可能已经被修改了，因此要去内核栈上找已经被保存下来的值。
    mv a0, sp   #将sp的值复制到a0中
    call trap_handler

__restore:
    #开始运行 app 通过__restore
    #在处理玩trap返回到用户态
    mv sp, a0   #(猜测：可能保存了返回值要传递回来)
    #现在sp->内核栈(分配后)，sscratch->用户栈
    #先恢复 CSR 再恢复通用寄存器，这样我们使用的三个临时寄存器才能被正确恢复

    #恢复CSR
    #数据传输指令，取双字。将32*8(sp)中的数据取到t0寄存器
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)
    #向控制和状态寄存器中写入数据 csrr csr, rd 功能将 rd 的值写到寄存器CSR中
    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2

    #恢复通用寄存器
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr

    #在内核栈回收Trap上下文
    addi sp, sp, 34*8

    #交换 sp 和 sscratch sp->user stack, sscratch->kernel stack
    csrrw sp, sscratch, sp

    #返回到上一个模式 回到 U 特权级继续运行app
    sret