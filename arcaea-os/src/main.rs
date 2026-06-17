// arcaea-os/src/main.rs
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
extern crate alloc;

use core::panic::PanicInfo;

// 引入底层模块
mod allocator;
mod trap;

// 引入 ACTP 编译器生成的组件代码
include!("generated.rs");

// ==========================================
// RISC-V 启动汇编入口
// ==========================================
core::arch::global_asm!(
    ".section .text.entry",
    ".globl _start",
    "_start:",
    "   la sp, _stack_top",
    "   call rust_main",
    "1:  wfi",
    "   j 1b"
);

// ==========================================
// SBI 底层通讯与打印宏 (供全项目调用)
// ==========================================
fn sbi_call(eid: usize, fid: usize, arg0: usize) -> usize {
    let mut ret;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") eid, in("a6") fid, in("a0") arg0, lateout("a0") ret,
            options(nostack)
        );
    }
    ret
}

pub fn sbi_print_char(c: u8) {
    sbi_call(0x01, 0x00, c as usize);
}

pub fn sbi_print(s: &str) {
    for byte in s.bytes() {
        sbi_print_char(byte);
    }
}

// 【关键】使用 #[macro_export] 让 allocator.rs 和 trap.rs 也能调用
#[macro_export]
macro_rules! sbi_println {
    ($s:expr) => {
        $crate::sbi_print($s);
        $crate::sbi_print("\r\n");
    };
}

// ==========================================
// 内存分配错误处理
// ==========================================
#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    sbi_println!("[ALLOC] FATAL: Out of memory!");
    loop { unsafe { core::arch::asm!("wfi"); } }
}

// ==========================================
// 内核主入口
// ==========================================
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    sbi_println!("========================================");
    sbi_println!("   A.R.C.A.E.A. SYSTEM BOOT SEQUENCE   ");
    sbi_println!("========================================");
    
    // 1. 初始化底层硬件
    trap::init_trap();
    unsafe { allocator::ALLOCATOR.init(); }
    
    sbi_println!("[BOOT] ARC-RustOS Kernel v0.1.0-Genesis");
    sbi_println!("[ARCH] RISC-V 64-bit (rv64gc)");
    sbi_println!("");
    
    // 2. 运行 ACTP 声明式组件 (内存读写测试)
    sbi_println!("[FSM] Initializing Stable Components...");
    lca_driver::init();
    sbi_println!("[SYS ] arcaeaOS Generation Anchored.");
    sbi_println!("");
    
    // 3. 触发异常测试 (验证 Trap Handler 安全返回)
    sbi_println!("[TEST] Triggering ecall exception...");
    unsafe { core::arch::asm!("ecall"); }
    
    sbi_println!("[TEST] Returned from exception successfully!");
    sbi_println!("[HAL ] Entering low-power hibernation...");
    
    // 4. 进入休眠
    hal_riscv::enter_hibernation()
}

// ==========================================
// Panic 处理
// ==========================================
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sbi_println!("[FSM] FATAL ERROR: PROTOCOL VIOLATION");
    loop { unsafe { core::arch::asm!("wfi"); } }
}