// arcaea-os/src/trap.rs

// ==========================================
// RISC-V S-mode Trap 處理入口 (彙編)
// ==========================================
core::arch::global_asm!(
    ".section .text.trap",
    ".globl trap_handler_entry",
    "trap_handler_entry:",
    "addi sp, sp, -256",
    "sd ra, 0(sp)", "sd sp, 8(sp)", "sd gp, 16(sp)", "sd tp, 24(sp)",
    "sd t0, 32(sp)", "sd t1, 40(sp)", "sd t2, 48(sp)", "sd s0, 56(sp)",
    "sd s1, 64(sp)", "sd a0, 72(sp)", "sd a1, 80(sp)", "sd a2, 88(sp)",
    "sd a3, 96(sp)", "sd a4, 104(sp)", "sd a5, 112(sp)", "sd a6, 120(sp)",
    "sd a7, 128(sp)", "sd s2, 136(sp)", "sd s3, 144(sp)", "sd s4, 152(sp)",
    "sd s5, 160(sp)", "sd s6, 168(sp)", "sd s7, 176(sp)", "sd s8, 184(sp)",
    "sd s9, 192(sp)", "sd s10, 200(sp)", "sd s11, 208(sp)", "sd t3, 216(sp)",
    "sd t4, 224(sp)", "sd t5, 232(sp)", "sd t6, 240(sp)",
    "mv a0, sp",
    "call rust_trap_handler",
    "ld ra, 0(sp)", "ld gp, 16(sp)", "ld tp, 24(sp)", "ld t0, 32(sp)",
    "ld t1, 40(sp)", "ld t2, 48(sp)", "ld s0, 56(sp)", "ld s1, 64(sp)",
    "ld a0, 72(sp)", "ld a1, 80(sp)", "ld a2, 88(sp)", "ld a3, 96(sp)",
    "ld a4, 104(sp)", "ld a5, 112(sp)", "ld a6, 120(sp)", "ld a7, 128(sp)",
    "ld s2, 136(sp)", "ld s3, 144(sp)", "ld s4, 152(sp)", "ld s5, 160(sp)",
    "ld s6, 168(sp)", "ld s7, 176(sp)", "ld s8, 184(sp)", "ld s9, 192(sp)",
    "ld s10, 200(sp)", "ld s11, 208(sp)", "ld t3, 216(sp)", "ld t4, 224(sp)",
    "ld t5, 232(sp)", "ld t6, 240(sp)",
    "addi sp, sp, 256",
    "sret"
);

// ==========================================
// Rust 層面的 Trap 處理函數 (絕對純淨版)
// ==========================================
#[no_mangle]
pub extern "C" fn rust_trap_handler(_trap_frame: *mut usize) {
    let scause: usize;
    let mut sepc: usize;
    unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) scause);
        core::arch::asm!("csrr {}, sepc", out(reg) sepc);
    }

    // 區分中斷 (Interrupt) 和異常 (Exception)
    let is_interrupt = (scause & 0x8000000000000000) != 0;
    let _cause_code = scause & 0x7FFFFFFFFFFFFFFF;

    // 【核心修復】忽略所有異步中斷（如定時器中斷），絕對不修改 sepc，也不引用任何外部變量！
    if is_interrupt {
        return;
    }

    // 對於異常，跳過導致異常的指令 (sepc + 4)
    unsafe {
        core::arch::asm!("csrw sepc, {}", in(reg) sepc + 4);
    }
}

// ==========================================
// 初始化 Trap 向量表
// ==========================================
pub fn init_trap() {
    unsafe {
        core::arch::asm!(
            "la t0, trap_handler_entry",
            "li t1, -4",
            "and t0, t0, t1",
            "csrw stvec, t0",
            options(nostack)
        );
        crate::sbi_println!("[TRAP] Trap handler installed.");
    }
}