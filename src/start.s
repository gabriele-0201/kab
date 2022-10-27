
// external function, start of the kernel.c
.extern kernel_main

// global becouse the linker have to see this
//.global start

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
    // Prepare heap and stack space
    heap_bottom:
        .skip 4 * 1024 // 4KiB
    heap_top:
    .align 16 // WHY?
    stack_bottom:
        .skip 1048576 * 1 // 1MB <- why not 1024*1024?
    stack_top:

.section .text
.global start
    start: 
        //mov $stack_top, %esp
        lea esp, stack_top
        lea ecx, heap_bottom
        lea edx, heap_top

        push esp // push stack top position
        push edx // push heap top position
        push ecx // push heap bottom position
        push ebx // push Multiboot info
        push eax // push Multiboot flag

        // now the environment is ready, start the code
        call kernel_main

        hang:
            cli // disable CPU interrupts
            hlt // halt the CPU
            jmp hang // if does not work loop again

.global reloadSegments
    reloadSegments:
       // Reload CS register containing code selector:
       //ljmp   0x08, reload_cs // 0x08 is a stand-in for your code segment
       jmp   0x08, reload_cs // global_asm does not like ljmp
    reload_cs:
       // Reload data segment registers:
       mov   ax, 0x10 // 0x10 is a stand-in for your data segment
       mov   ds, ax
       mov   es, ax
       mov   fs, ax
       mov   gs, ax
       mov   ss, ax
       ret

.section .data
    gdtr:
        .word 0 // For limit storage
        .long 0 // For base storage
