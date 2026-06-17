// arcaea-os/src/context.rs

/// 【核心】Trap 帧 / 任务上下文
/// 必须使用 #[repr(C)] 确保内存布局与汇编代码中的压栈顺序严格一致！
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TrapFrame {
    // --- 通用寄存器 (与 trap.rs 汇编中的 sd 顺序完全对应) ---
    pub ra: usize,  // x1: Return Address
    pub sp: usize,  // x2: Stack Pointer
    pub gp: usize,  // x3: Global Pointer
    pub tp: usize,  // x4: Thread Pointer
    pub t0: usize,  // x5: Temporary
    pub t1: usize,  // x6: Temporary
    pub t2: usize,  // x7: Temporary
    pub s0: usize,  // x8: Saved (Frame Pointer)
    pub s1: usize,  // x9: Saved
    pub a0: usize,  // x10: Argument / Return Value
    pub a1: usize,  // x11: Argument / Return Value
    pub a2: usize,  // x12: Argument
    pub a3: usize,  // x13: Argument
    pub a4: usize,  // x14: Argument
    pub a5: usize,  // x15: Argument
    pub a6: usize,  // x16: Argument
    pub a7: usize,  // x17: Argument
    pub s2: usize,  // x18: Saved
    pub s3: usize,  // x19: Saved
    pub s4: usize,  // x20: Saved
    pub s5: usize,  // x21: Saved
    pub s6: usize,  // x22: Saved
    pub s7: usize,  // x23: Saved
    pub s8: usize,  // x24: Saved
    pub s9: usize,  // x25: Saved
    pub s10: usize, // x26: Saved
    pub s11: usize, // x27: Saved
    pub t3: usize,  // x28: Temporary
    pub t4: usize,  // x29: Temporary
    pub t5: usize,  // x30: Temporary
    pub t6: usize,  // x31: Temporary
    
    // --- 特权级寄存器 (在汇编中未保存，需在 Rust 层通过 csrr 读取) ---
    pub sepc: usize,    // Supervisor Exception Program Counter
    pub sstatus: usize, // Supervisor Status Register
}

impl TrapFrame {
    /// 创建一个初始的、全零的上下文 (用于新任务)
    pub const fn new() -> Self {
        Self {
            ra: 0, sp: 0, gp: 0, tp: 0,
            t0: 0, t1: 0, t2: 0, s0: 0, s1: 0,
            a0: 0, a1: 0, a2: 0, a3: 0, a4: 0, a5: 0, a6: 0, a7: 0,
            s2: 0, s3: 0, s4: 0, s5: 0, s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
            t3: 0, t4: 0, t5: 0, t6: 0,
            sepc: 0,
            sstatus: 0,
        }
    }
}