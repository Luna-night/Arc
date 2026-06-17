/* arcaeaOS RISC-V 記憶體錨定協議 */
OUTPUT_ARCH(riscv)
ENTRY(_start)

/* RISC-V 標準內核入口地址（QEMU virt 機器，OpenSBI 加載點） */
/* 0x80200000 = 0x80000000 (RAM 基址) + 0x200000 (2MB 偏移) */
BASE_ADDRESS = 0x80200000;

SECTIONS {
    . = BASE_ADDRESS;
    
    skernel = .;
    
    .text : {
        *(.text.entry)      /* 內聯匯編入口點優先 */
        *(.text .text.*)
    }
    
    .rodata : {
        *(.rodata .rodata.*)
    }
    
    .data : {
        *(.data .data.*)
    }
    
    .bss : {
        sbss = .;
        *(.bss .bss.*)
        *(COMMON)
        ebss = .;
    }
    
    /* 配置 64KB 的堆疊空間 */
    . = ALIGN(4K);
    _stack_start = .;
    . = . + 0x10000; 
    _stack_top = .;
    
    ekernel = .;
}
