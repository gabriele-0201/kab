.intel_syntax noprefix

// external function, start of the kernel.c
.extern kernel_main

// global becouse the linker have to see this
.global start
.global set_gdt

// the bootloader GRUB need some standard basic info
// the standard used is 'Multiboot'
// the following constants will define the Multiboot Header
.set MB_MAGIC, 0x1BADB002 // magic constant used by grub to define the kernel location
.set MB_FLAGS, (1 << 0) | (1 << 1) // 1: load modules on page bounderies, 2: provide memory map
.set MB_CHECKSUM, (0 - (MB_MAGIC + MB_FLAGS)) // check sum that include both

// define the section of the exevutable that will contain the Mutiboot Header
.section .multiboot 
    .align 4 // data has to be aligned on multiple of 4 bytes
    .long MB_MAGIC
    .long MB_FLAGS
    .long MB_CHECKSUM

// data initialized to zeros when the kernel is loaded
.section .bss
    // C code need a stack
    .align 16 // WHY?
    stack_bottom:
        .skip 4096 // 4K
    stack_top:

.section .text
    start: 
        mov esp, stack_top
        //mov esp, stack_top

        // now the environment is ready, start the code
        call kernel_main

        hang:
            cli // disable CPU interrupts
            hlt // halt the CPU
            jmp hang // if does not work loop again

    // setGdt(limit, base)
    set_gdt:
        MOV   AX, [esp + 4]
        MOV   [gdtr], AX
        MOV   EAX, [ESP + 8]
        MOV   [gdtr + 2], EAX
        LGDT  [gdtr]
        RET
    //set_gdt:
    //    mov 4(%esp), %ax
    //    mov %ax, $gdtr
    //    mov 8(%esp), %eax
    //    mov %eax, $gdtr+2
    //    lgdt $gdtr
    //    ret

.section .data
    gdtr:
        .word 0 // For limit storage
        .long 0 // For base storage
