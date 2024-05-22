use riscv::register::sstatus::{self, Sstatus, SPP};

///Trap Context
#[repr(C)] //这是最重要的一种 repr。它的目的很简单，就是和 C 保持一致。
pub struct TrapContext {
    /// general regs[0..31]
    pub x: [usize; 32],
    /// CSR sstatus
    pub sstatus: Sstatus,
    /// CSR sepc
    pub sepc: usize,
}

impl TrapContext {
    //设置x2 
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    //初始化context
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry, //0x80400000 sret后跳到这个地开始执行
        };
        cx.set_sp(sp); //设置x2为用户栈栈顶
        cx
    }
}