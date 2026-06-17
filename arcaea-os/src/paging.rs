// arcaea-os/src/paging.rs

// Sv39 PTE (Page Table Entry) 標誌位
pub const PTE_V: usize = 1 << 0; // Valid
pub const PTE_R: usize = 1 << 1; // Read
pub const PTE_W: usize = 1 << 2; // Write
pub const PTE_X: usize = 1 << 3; // Execute
pub const PTE_A: usize = 1 << 6; // Accessed (必須手動設置)
pub const PTE_D: usize = 1 << 7; // Dirty (必須手動設置)

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry(pub usize);

// 頁表結構 (512 個 PTE，必須 4KB 對齊)
#[derive(Debug, Clone, Copy)]
#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        Self { entries: [PageTableEntry(0); 512] }
    }
}

// 只需要一個根頁表 (Root Page Table)
#[repr(C, align(4096))]
static mut ROOT_PT: PageTable = PageTable::new();

// ==========================================
// 【核心修復】使用 1GB 大頁 (Gigapage) 進行恆等映射
// 徹底拋棄繁瑣的 4KB 映射與動態分配！
// ==========================================
pub unsafe fn init_mmu() {
    // 1. 映射 0x00000000 - 0x3FFFFFFF (包含 UART 0x10000000)
    // 物理地址 0x0，PPN 全為 0。設置 R/W/X 表示這是 1GB 大頁。
    ROOT_PT.entries[0] = PageTableEntry(
        (0usize << 10) | PTE_V | PTE_R | PTE_W | PTE_X | PTE_A | PTE_D
    );

    // 2. 映射 0x80000000 - 0xBFFFFFFF (包含 RAM 0x80000000 - 0x88000000)
    // 物理地址 0x80000000。
    // Sv39 PTE 的 PPN 佔據 [53:10]。
    // 0x80000000 >> 12 = 0x80000。
    // 0x80000 << 10 = 0x20000000。
    let pa_ram = 0x80000000usize;
    ROOT_PT.entries[2] = PageTableEntry(
        ((pa_ram >> 12) << 10) | PTE_V | PTE_R | PTE_W | PTE_X | PTE_A | PTE_D
    );

    // 3. 寫入 satp 寄存器，開啟 Sv39 (MODE = 8)
    // 使用 addr_of! 避免 Rust 2024 的 static mut 引用警告
    let root_pa = core::ptr::addr_of!(ROOT_PT) as usize;
    let satp = (8usize << 60) | (root_pa >> 12); 
    
    core::arch::asm!(
        "csrw satp, {0}",
        "sfence.vma", // 刷新 TLB
        in(reg) satp
    );
}