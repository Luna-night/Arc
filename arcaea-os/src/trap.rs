// arcaea-os/src/trap.rs
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

#[no_mangle]
pub extern "C" fn rust_trap_handler(_trap_frame: *mut usize) {
    let scause: usize;
    let mut sepc: usize;
    unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) scause);
        core::arch::asm!("csrr {}, sepc", out(reg) sepc);
    }
    
    let is_interrupt = (scause & 0x8000000000000000) != 0;
    let cause_code = scause & 0x7FFFFFFFFFFFFFFF;
    
    // 忽略異步中斷（如定時器中斷）
    if is_interrupt { return; }
    
    // 【核心修復】極速寫入異步日誌緩衝區，絕不阻塞，絕不觸發嵌套異常！
    unsafe {
        crate::log::trap_log_write("\r\n[TRAP] Exception detected! scause=");
        crate::log::trap_log_write_hex(cause_code);
        crate::log::trap_log_write("\r\n");
        
        if cause_code == 9 {
            crate::log::trap_log_write("[TRAP] Type: Environment Call (ecall)\r\n");
        } else if cause_code == 3 {
            crate::log::trap_log_write("[TRAP] Type: Breakpoint (ebreak)\r\n");
        } else if cause_code == 13 {
            crate::log::trap_log_write("[TRAP] Type: Load Access Fault\r\n");
        } else {
            crate::log::trap_log_write("[TRAP] Type: Unknown Exception\r\n");
        }
        
        // 跳過導致異常的指令 (sepc + 4)
        core::arch::asm!("csrw sepc, {}", in(reg) sepc + 4);
    }
}

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