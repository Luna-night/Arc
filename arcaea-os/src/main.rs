#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. 硬體級串口初始化 (COM1: 0x3f8, 115200 8N1)
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0x3f8u16 + 1, in("al") 0x00u8);
        core::arch::asm!("out dx, al", in("dx") 0x3f8u16 + 3, in("al") 0x80u8);
        core::arch::asm!("out dx, al", in("dx") 0x3f8u16 + 0, in("al") 0x03u8);
        core::arch::asm!("out dx, al", in("dx") 0x3f8u16 + 1, in("al") 0x00u8);
        core::arch::asm!("out dx, al", in("dx") 0x3f8u16 + 3, in("al") 0x03u8);
        core::arch::asm!("out dx, al", in("dx") 0x3f8u16 + 2, in("al") 0xC7u8);
        core::arch::asm!("out dx, al", in("dx") 0x3f8u16 + 4, in("al") 0x0Bu8);
    }

    // 2. 定義要輸出的字節切片
    let all_msgs: &[&[u8]] = &[
        b"========================================\n",
        b"   A.R.C.A.E.A. SYSTEM BOOT SEQUENCE   \n",
        b"========================================\n",
        b"[SYS] arcaeaOS Generation Anchored.\n",
        b"[SYS] Welcome, Commander.\n",
    ];

    // 3. 底層字節發送循環 (包含硬體握手輪詢)
    for m in all_msgs.iter() {
        for &byte in m.iter() {
            unsafe {
                let mut status: u8;
                loop {
                    core::arch::asm!("in al, dx", out("al") status, in("dx") 0x3f8u16 + 5, options(nomem, nostack, preserves_flags));
                    if (status & 0x20) != 0 { break; }
                }
                core::arch::asm!("out dx, al", in("dx") 0x3f8u16, in("al") byte, options(nomem, nostack, preserves_flags));
            }
        }
    }

    // 4. 進入低功耗休眠
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { unsafe { core::arch::asm!("hlt"); } }
}