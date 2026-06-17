// arcaea-os/src/task.rs
use crate::context::TrapFrame;

/// 任务状态机 (对应 FSM 协议)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskState {
    Ready,    // 就绪，等待 CPU
    Running,  // 正在运行
    Blocked,  // 阻塞 (例如等待 I/O 或锁)
    Dead,     // 已终止
}

/// 【核心】任务控制块 (Task Control Block)
pub struct Task {
    pub id: usize,
    pub state: TaskState,
    pub context: TrapFrame,
    // 为每个任务分配 4KB 的内核栈 (在裸机环境下，简单使用内联数组)
    // 注意：RISC-V 要求栈指针 16 字节对齐
    pub kernel_stack: [u8; 4096], 
}

impl Task {
    /// 创建一个新的任务
    pub fn new(id: usize, entry_point: usize) -> Self {
        let mut task = Self {
            id,
            state: TaskState::Ready,
            context: TrapFrame::new(),
            kernel_stack: [0; 4096],
        };

        // 初始化上下文
        // 1. 设置栈指针 (sp) 指向栈顶 (数组末尾)，并确保 16 字节对齐
        let stack_top = task.kernel_stack.as_ptr() as usize + 4096;
        task.context.sp = stack_top & !0xF; // 清除低 4 位，强制 16 字节对齐

        // 2. 设置程序计数器 (sepc) 为任务的入口函数地址
        task.context.sepc = entry_point;

        // 3. 设置 sstatus 寄存器
        // 关键：设置 SPP (Supervisor Previous Privilege) 为 S-Mode (1)
        // 这样当 sret 返回时，CPU 知道是从 S-Mode 返回到 S-Mode
        // 同时开启 SIE (Supervisor Interrupt Enable) 位 (bit 1)
        task.context.sstatus = 1 << 8 | 1 << 1; // SPP=1, SIE=1

        task
    }
}