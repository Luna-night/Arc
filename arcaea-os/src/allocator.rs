// arcaea-os/src/allocator.rs
use core::alloc::{GlobalAlloc, Layout};

// 【物理内存池定义】
// QEMU virt 机器的 RAM 从 0x80000000 开始。
// 内核加载在 0x80200000。我们划出 0x88000000 开始的 4MB 作为我们的物理内存池。
const POOL_START: usize = 0x88000000;
const POOL_SIZE: usize = 4 * 1024 * 1024; // 4MB
const PAGE_SIZE: usize = 4096;
const NUM_PAGES: usize = POOL_SIZE / PAGE_SIZE; // 1024 页
const BITMAP_SIZE: usize = NUM_PAGES / 8;       // 128 字节位图

// 【位图状态】0 = 空闲 (Free), 1 = 已分配 (Used)
// 使用 static mut 在裸机环境下进行全局可变状态管理
static mut BITMAP: [u8; BITMAP_SIZE] = [0; BITMAP_SIZE];
static mut ALLOCATOR_INITIALIZED: bool = false;

pub struct BitmapAllocator;

impl BitmapAllocator {
    pub const fn new() -> Self {
        BitmapAllocator
    }

    // 必须在 rust_main 早期调用
    pub unsafe fn init(&self) {
        // 清零位图，标记所有页为空闲
        core::ptr::write_bytes(BITMAP.as_mut_ptr(), 0, BITMAP_SIZE);
        ALLOCATOR_INITIALIZED = true;
        crate::sbi_println!("[ALLOC] Bitmap Allocator initialized (4MB Pool @ 0x88000000)");
    }

    unsafe fn allocate_page(&self) -> Option<usize> {
        for i in 0..NUM_PAGES {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            
            // 检查该位是否为 0 (空闲)
            if (BITMAP[byte_idx] & (1 << bit_idx)) == 0 {
                // 标记为已分配
                BITMAP[byte_idx] |= 1 << bit_idx;
                return Some(POOL_START + i * PAGE_SIZE);
            }
        }
        None // 内存耗尽
    }

    unsafe fn deallocate_page(&self, addr: usize) {
        if addr < POOL_START || addr >= POOL_START + POOL_SIZE {
            return;
        }
        let page_idx = (addr - POOL_START) / PAGE_SIZE;
        let byte_idx = page_idx / 8;
        let bit_idx = page_idx % 8;
        
        // 标记为空闲
        BITMAP[byte_idx] &= !(1 << bit_idx);
    }
}

unsafe impl GlobalAlloc for BitmapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !ALLOCATOR_INITIALIZED {
            return core::ptr::null_mut();
        }

        // 极简策略：对于小于等于 4KB 的请求，分配一个物理页
        // 对于更大的请求，分配多个连续页 (此处简化为只处理 <= 4KB)
        if layout.size() <= PAGE_SIZE {
            if let Some(addr) = self.allocate_page() {
                return addr as *mut u8;
            }
        }
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        self.deallocate_page(ptr as usize);
    }
}

// ... (前面的代碼保持不變)
// 聲明全局分配器 (必須是 pub 才能被 main.rs 訪問)
#[global_allocator]
pub static ALLOCATOR: BitmapAllocator = BitmapAllocator::new();