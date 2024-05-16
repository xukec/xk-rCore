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