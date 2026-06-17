// arcaea-os/src/allocator.rs
use core::alloc::{GlobalAlloc, Layout};

// 【核心修复 1】将内存池移至 64MB (0x84000000) 处，大小 32MB
// QEMU virt 机器 RAM 从 0x80000000 开始，128MB 边界是 0x88000000。
// 0x84000000 + 32MB = 0x86000000，绝对安全，杜绝越界！
const POOL_START: usize = 0x84000000;
const POOL_SIZE: usize = 32 * 1024 * 1024; // 32MB
const PAGE_SIZE: usize = 4096;
const NUM_PAGES: usize = POOL_SIZE / PAGE_SIZE; // 8192 页
const BITMAP_SIZE: usize = NUM_PAGES / 8;       // 1024 字节位图

static mut BITMAP: [u8; BITMAP_SIZE] = [0; BITMAP_SIZE];
static mut ALLOCATOR_INITIALIZED: bool = false;

pub struct BitmapAllocator;

impl BitmapAllocator {
    pub const fn new() -> Self {
        BitmapAllocator
    }

    pub unsafe fn init(&self) {
        // 【核心修复 2】使用 addr_of_mut! 避免创建 mutable reference，消除 Rust 2024 警告
        core::ptr::write_bytes(core::ptr::addr_of_mut!(BITMAP) as *mut u8, 0, BITMAP_SIZE);
        ALLOCATOR_INITIALIZED = true;
        crate::sbi_println!("[ALLOC] Bitmap Allocator initialized (32MB Pool @ 0x84000000)");
    }

    unsafe fn allocate_pages(&self, num_pages: usize) -> Option<usize> {
        if num_pages == 0 || num_pages > NUM_PAGES {
            return None;
        }
        let bitmap_ptr = core::ptr::addr_of_mut!(BITMAP) as *mut u8;
        
        // 寻找连续的 num_pages 个空闲页
        for i in 0..=(NUM_PAGES - num_pages) {
            let mut free = true;
            for j in 0..num_pages {
                let byte_idx = (i + j) / 8;
                let bit_idx = (i + j) % 8;
                let byte_ptr = bitmap_ptr.add(byte_idx);
                if (core::ptr::read_volatile(byte_ptr) & (1 << bit_idx)) != 0 {
                    free = false;
                    break;
                }
            }
            
            if free {
                // 标记为已分配
                for j in 0..num_pages {
                    let byte_idx = (i + j) / 8;
                    let bit_idx = (i + j) % 8;
                    let byte_ptr = bitmap_ptr.add(byte_idx);
                    let current = core::ptr::read_volatile(byte_ptr);
                    core::ptr::write_volatile(byte_ptr, current | (1 << bit_idx));
                }
                return Some(POOL_START + i * PAGE_SIZE);
            }
        }
        None // 内存耗尽
    }

    unsafe fn deallocate_pages(&self, addr: usize, num_pages: usize) {
        if addr < POOL_START || addr >= POOL_START + POOL_SIZE {
            return; // 越界保护
        }
        let start_page = (addr - POOL_START) / PAGE_SIZE;
        let bitmap_ptr = core::ptr::addr_of_mut!(BITMAP) as *mut u8;
        
        // 清零位图，释放内存
        for j in 0..num_pages {
            let byte_idx = (start_page + j) / 8;
            let bit_idx = (start_page + j) % 8;
            let byte_ptr = bitmap_ptr.add(byte_idx);
            let current = core::ptr::read_volatile(byte_ptr);
            core::ptr::write_volatile(byte_ptr, current & !(1 << bit_idx));
        }
    }
}

unsafe impl GlobalAlloc for BitmapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !ALLOCATOR_INITIALIZED {
            return core::ptr::null_mut();
        }
        
        let size = layout.size();
        let align = layout.align();
        
        // 简化版对齐检查：如果请求的对齐大于 PAGE_SIZE，暂不支持
        if align > PAGE_SIZE {
            return core::ptr::null_mut();
        }

        let num_pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        if let Some(addr) = self.allocate_pages(num_pages) {
            addr as *mut u8
        } else {
            core::ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }
        let size = layout.size();
        let num_pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        self.deallocate_pages(ptr as usize, num_pages);
    }
}

// 声明全局分配器 (必须是 pub 以便外部可见)
#[global_allocator]
pub static ALLOCATOR: BitmapAllocator = BitmapAllocator::new();