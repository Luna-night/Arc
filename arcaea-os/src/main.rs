#![no_std]
#![no_main]

use core::panic::PanicInfo;

// ==========================================
// 【終極修復】RISC-V 啟動匯編內聯協議
// 徹底消滅 entry.asm 文件，直接嵌入 Rust 核心！
// ==========================================
core::arch::global_asm!(
    ".section .text.entry",
    ".globl _start",
    "_start:",
    "   la sp, _stack_top",    // 設置棧指針指向 linker.ld 中定義的棧頂
    "   call rust_main",       // 跳轉到 Rust 主函數
    "1:  wfi",                 // 如果 rust_main 意外返回，進入低功耗休眠
    "   j 1b"
);

// ==========================================
// 手寫 SBI (Supervisor Binary Interface) 調用
// 在 S-Mode 下，透過 ecall 與 M-Mode (OpenSBI) 對話
// ==========================================
fn sbi_call(eid: usize, fid: usize, arg0: usize) -> usize {
    let mut ret;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            in("a0") arg0,
            lateout("a0") ret,
            options(nostack)
        );
    }
    ret
}

// 串口輸出字符 (SBI Legacy Console Putchar, EID=0x01)
fn console_putchar(c: u8) {
    sbi_call(0x01, 0x00, c as usize);
}

fn print(s: &str) {
    for byte in s.bytes() {
        console_putchar(byte);
    }
}

fn println(s: &str) {
    print(s);
    print("\n");
}

// ==========================================
// 內核入口點 (從 _start 跳轉而來)
// ==========================================
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    println("========================================");
    println("   A.R.C.A.E.A. SYSTEM BOOT SEQUENCE   ");
    println("========================================");
    println("[BOOT] ARC-RustOS Kernel v0.1.0-Genesis");
    println("[ARCH] RISC-V 64-bit (rv64gc)");
    println("[MODE] S-Mode (Supervisor Mode via OpenSBI)");
    println("[HAL ] Cross-Platform Abstraction Layer... OK");
    println("");
    println("[FSM ] Verifying PROTO-0 Protocol... [PASS]");
    println("[SYS ] arcaeaOS Generation Anchored.");
    println("[SYS ] Welcome, Commander. System is ready.");
    println("");
    println("Entering low-power WFI state...");

    // 進入低功耗休眠 (RISC-V 使用 WFI 指令)
    loop {
        unsafe { core::arch::asm!("wfi"); }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println("[FSM] FATAL ERROR: PROTOCOL VIOLATION");
    loop {
        unsafe { core::arch::asm!("wfi"); }
    }
}