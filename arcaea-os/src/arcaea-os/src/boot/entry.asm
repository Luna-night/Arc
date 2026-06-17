# arcaeaOS RISC-V 启动入口
.section .text.entry
.globl _start

_start:
    # 1. 检查是否从 M-Mode 启动（首次启动）
    csrr t0, mhartid        # 读取硬件线程 ID
    bnez t0, park_loop      # 如果不是主线程，进入休眠
    
    # 2. 设置栈指针（指向栈顶）
    la sp, _stack_top
    
    # 3. 清零 BSS 段
    la t0, sbss
    la t1, ebss
    bgeu t0, t1, 2f         # 如果 sbss >= ebss，跳过清零
1:
    sd zero, (t0)           # 清零 8 字节
    addi t0, t0, 8
    bltu t0, t1, 1b         # 如果 t0 < t1，继续清零
2:

    # 4. 跳转到 Rust 主函数
    call rust_main

    # 5. 死循环（不应到达）
park_loop:
    wfi                     # Wait for Interrupt
    j park_loop