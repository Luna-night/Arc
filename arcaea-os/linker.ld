/* arcaeaOS 記憶體錨定協議 (Memory Anchoring Protocol) */
ENTRY(_start);

SECTIONS {
    /* 【核心修復】強制將核心載入到虛擬位址 2MB (0x200000) */
    /* 這將徹底避開 bootloader 0.9 在 0x1000 處的內部保留區，粉碎 PageAlreadyMapped 幽靈 */
    . = 0x200000;

    .text : {
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        *(.bss .bss.*)
    }

    /* 配置 4KB 的堆疊空間 */
    . = ALIGN(8);
    . = . + 0x1000; 
    _stack_top = .;
}
