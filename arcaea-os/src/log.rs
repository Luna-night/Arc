// arcaea-os/src/log.rs
pub const LOG_BUF_SIZE: usize = 512;
static mut LOG_BUF: [u8; LOG_BUF_SIZE] = [0; LOG_BUF_SIZE];
static mut LOG_HEAD: usize = 0; // 寫入位置
static mut LOG_TAIL: usize = 0; // 讀取位置

// 【核心】在 Trap 上下文中調用，必須極速且安全（無鎖，單核環境下依靠極簡邏輯）
pub unsafe fn trap_log_write(s: &str) {
    for &c in s.as_bytes() {
        let next_head = (LOG_HEAD + 1) % LOG_BUF_SIZE;
        if next_head != LOG_TAIL {
            LOG_BUF[LOG_HEAD] = c;
            LOG_HEAD = next_head;
        } else {
            break; // 緩衝區滿，丟棄新字符以防覆蓋未讀數據
        }
    }
}

// 輔助函數：將 usize 轉為 hex 字串寫入 buffer
pub unsafe fn trap_log_write_hex(val: usize) {
    let hex = b"0123456789abcdef";
    let mut buf = [0u8; 18]; // "0x" + 16 chars
    buf[0] = b'0';
    buf[1] = b'x';
    for i in 0..16 {
        let nibble = (val >> ((15 - i) * 4)) & 0xF;
        buf[2 + i] = hex[nibble as usize];
    }
    trap_log_write(core::str::from_utf8_unchecked(&buf));
}

pub fn flush_logs() {
    unsafe {
        while LOG_HEAD != LOG_TAIL {
            let c = LOG_BUF[LOG_TAIL];
            LOG_TAIL = (LOG_TAIL + 1) % LOG_BUF_SIZE;
            // 調用主程序暴露的安全打印函數
            crate::sbi_print_char(c);
        }
    }
}