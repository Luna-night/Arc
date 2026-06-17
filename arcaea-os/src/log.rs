// arcaea-os/src/log.rs
pub const LOG_BUF_SIZE: usize = 512;
static mut LOG_BUF: [u8; LOG_BUF_SIZE] = [0; LOG_BUF_SIZE];
static mut LOG_HEAD: usize = 0; // 写入位置 (Trap Handler 操作)
static mut LOG_TAIL: usize = 0; // 读取位置 (主循环操作)

// 【核心标志】通知主循环需要重新设置定时器
pub static mut TIMER_NEED_REARM: bool = false;

// 【核心】在 Trap Handler (中断上下文) 中调用。
// 绝对安全：只写内存，不阻塞，不调用 ecall！
pub unsafe fn trap_log(s: &str) {
    for c in s.bytes() {
        let next = (LOG_HEAD + 1) % LOG_BUF_SIZE;
        if next != LOG_TAIL {
            LOG_BUF[LOG_HEAD] = c;
            LOG_HEAD = next;
        }
        // 如果缓冲区满了，丢弃新字符，防止覆盖未读数据
    }
}

// 【核心】在主循环 (安全上下文) 中调用。
// 安全地通过 SBI 打印缓冲区内容。
pub fn flush_logs() {
    while unsafe { LOG_HEAD != LOG_TAIL } {
        let c = unsafe { LOG_BUF[LOG_TAIL] };
        unsafe { LOG_TAIL = (LOG_TAIL + 1) % LOG_BUF_SIZE; }
        crate::sbi_print_char(c);
    }
}