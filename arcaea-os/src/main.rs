#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
extern crate alloc;

use core::panic::PanicInfo;

mod allocator;
mod trap;
mod log; // 【新增】引入異步日誌模組

core::arch::global_asm!(
    ".section .text.entry",
    ".globl _start",
    "_start:",
    "   la sp, _stack_top",
    "   call rust_main",
    "1:  wfi",
    "   j 1b"
);

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

// 【新增】暴露單字符打印，供 log 模組在安全上下文中調用
pub fn sbi_print_char(c: u8) {
    sbi_call(0x01, 0x00, c as usize);
}

pub fn sbi_print(s: &str) {
    for byte in s.bytes() {
        sbi_print_char(byte);
    }
}

#[macro_export]
macro_rules! sbi_println {
    ($s:expr) => {
        $crate::sbi_print($s);
        $crate::sbi_print("\r\n");
    };
}

include!("generated.rs");

#[alloc_error_handler]
fn alloc_error(_layout: core::alloc::Layout) -> ! {
    sbi_println!("[ALLOC] FATAL: Out of memory!");
    loop { unsafe { core::arch::asm!("wfi"); } }
}

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    sbi_println!("========================================");
    sbi_println!("   A.R.C.A.E.A. SYSTEM BOOT SEQUENCE   ");
    sbi_println!("========================================");
    
    trap::init_trap();
    unsafe { allocator::ALLOCATOR.init(); }
    
    sbi_println!("[BOOT] ARC-RustOS Kernel v0.1.0-Genesis");
    sbi_println!("[ARCH] RISC-V 64-bit (rv64gc)");
    sbi_println!("");
    
    sbi_println!("[FSM] Initializing Stable Components...");
    lca_driver::init();
    sbi_println!("[SYS ] arcaeaOS Generation Anchored.");
    sbi_println!("");
    
    // 【核心測試】觸發 ecall 異常 (注意：絕對不能用 ebreak，會觸發 LLVM Panic)
    sbi_println!("[TEST] Triggering ecall exception...");
    unsafe { core::arch::asm!("ecall"); }
    
    // 【核心修復】在主程序安全上下文中，刷新 Trap Handler 記錄的日誌！
    sbi_println!("[TEST] Flushing async trap logs...");
    crate::log::flush_logs();
    
    sbi_println!("[TEST] Returned from exception successfully!");
    sbi_println!("[HAL ] Entering low-power hibernation...");
    hal_riscv::enter_hibernation()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sbi_println!("[FSM] FATAL ERROR: PROTOCOL VIOLATION");
    loop { unsafe { core::arch::asm!("wfi"); } }
}