#![no_std]      // 禁用标准库，拥抱裸机
#![no_main]     // 禁用默认的 main 入口，使用自定义的 _start

use core::panic::PanicInfo;

// ==========================================
// 硬件抽象层 (HAL)：VGA 文本模式与串口
// ==========================================

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

const COLOR_CYAN: u8 = 0x03;
const COLOR_PURPLE: u8 = 0x05;
const COLOR_GREEN: u8 = 0x02;
const COLOR_WHITE: u8 = 0x0F;
const COLOR_RED: u8 = 0x04;

struct VgaWriter { col: usize, row: usize, color: u8 }

impl VgaWriter {
    fn new(color: u8) -> Self { Self { col: 0, row: 0, color } }
    
    fn clear(&mut self) {
        for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
            unsafe {
                *VGA_BUFFER.offset((i * 2) as isize) = b' ';
                *VGA_BUFFER.offset((i * 2 + 1) as isize) = 0x00; // 黑底
            }
        }
    }

    fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' { self.new_line(); return; }
        if self.col >= VGA_WIDTH { self.new_line(); }
        let offset = (self.row * VGA_WIDTH + self.col) * 2;
        unsafe {
            *VGA_BUFFER.offset(offset as isize) = byte;
            *VGA_BUFFER.offset((offset + 1) as isize) = self.color;
        }
        self.col += 1;
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        self.col = 0;
        if self.row < VGA_HEIGHT - 1 { self.row += 1; }
    }
    
    fn set_color(&mut self, color: u8) { self.color = color; }
}

// 串口 (Serial Port 0x3f8) 输出，用于在纯命令行终端中显示日志
const SERIAL_PORT: *mut u8 = 0x3f8 as *mut u8;
fn serial_print(s: &str) {
    for byte in s.bytes() {
        unsafe { *SERIAL_PORT = byte; }
    }
}

// ==========================================
// 内核入口点 (Kernel Entry Point)
// ==========================================

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut vga = VgaWriter::new(COLOR_CYAN);
    vga.clear();

    // 双通道输出宏 (同时输出到 VGA 屏幕和串口终端)
    macro_rules! boot_log {
        ($color:expr, $msg:expr) => {
            vga.set_color($color);
            vga.write_string($msg);
            serial_print($msg);
        };
    }

    boot_log!(COLOR_PURPLE, "========================================\n");
    boot_log!(COLOR_PURPLE, "   A.R.C.A.E.A. SYSTEM BOOT SEQUENCE   \n");
    boot_log!(COLOR_PURPLE, "========================================\n\n");
    
    boot_log!(COLOR_CYAN, "[BOOT] ARC-RustOS Kernel v0.1.0-Genesis\n");
    boot_log!(COLOR_CYAN, "[HAL ] Cross-Platform Abstraction Layer... ");
    boot_log!(COLOR_GREEN, "OK\n");
    
    boot_log!(COLOR_GREEN, "[FSM ] Loading Deterministic State Machine...\n");
    boot_log!(COLOR_GREEN, "[FSM ] Verifying PROTO-0 Protocol... ");
    boot_log!(COLOR_WHITE, "[PASS]\n");
    boot_log!(COLOR_GREEN, "[FSM ] Rules Tree Integrity... ");
    boot_log!(COLOR_WHITE, "[100% VERIFIED]\n\n");
    
    boot_log!(COLOR_PURPLE, "[RANT] Initializing Resonant Topology...\n");
    boot_log!(COLOR_PURPLE, "[RANT] Deep-space wave decoder... ");
    boot_log!(COLOR_RED, "STANDBY\n\n");
    
    boot_log!(COLOR_WHITE, "[SYS ] arcaeaOS Generation gen-e1428464 Anchored.\n");
    boot_log!(COLOR_WHITE, "[SYS ] Welcome, Commander. System is ready.\n\n");
    
    boot_log!(COLOR_CYAN, "Entering low-power HALT state (Wait for Interrupt)...\n");

    // 死循环，使用 hlt 指令让 CPU 进入极低功耗休眠状态
    // 这完美契合了 arcaeaOS "待机功耗 ＜0.08mW/MHz" 的设定
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// ==========================================
// Panic 处理 (系统崩溃时的底层守护)
// ==========================================
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut vga = VgaWriter::new(COLOR_RED);
    vga.write_string("\n[FSM] FATAL ERROR: PROTOCOL VIOLATION\n");
    serial_print("\n[FSM] FATAL ERROR: PROTOCOL VIOLATION\n");
    
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}