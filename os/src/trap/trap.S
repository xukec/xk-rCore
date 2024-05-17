.altmacro #加上才能正常使用.rept命令
#保存循环体
.macro SACE_GP n
    sd x\n, \n*8(sp)    #\n替换n参数
.endm

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
    #可以自由地使用t0/t1/t2，因为它们是在内核栈上的
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
    mv a0, sp   #将sp的值移动到a0中
    call trap_handler