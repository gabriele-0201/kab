
// external function, start of the kernel.c
.extern kernel_main

// global becouse the linker have to see this
.global start
.global set_gdt

// TEST
.extern handle_interrupt
.global handleException0x00
.global handleException0x06
.global handleInterruptRequest0x00
.global handleInterruptRequest0x01
.global interruptIgnore


.global testJmpAfterGdt
.global reloadSegments


.set IRQ_BASE, 0x20
// END TEST

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
        .skip 1048576 * 10 // 10MB
//        .skip 4096 // 1MB
    stack_top:

.section .text

    test:
        jmp resume

    start: 
        lea esp, stack_top
        //mov $stack_top, %esp

        // now the environment is ready, start the code
        call kernel_main

        hang:
            cli // disable CPU interrupts
            hlt // halt the CPU
            jmp hang // if does not work loop again

    // setGdt(limit, base)
    set_gdt:
        call testJmpAfterGdt
        MOV   AX, [esp + 4]
        MOV   [gdtr], AX
        MOV   EAX, [ESP + 8]
        MOV   [gdtr + 2], EAX
        LGDT  [gdtr]

        reloadSegments:
           // Reload CS register containing code selector:
           LJMP   0x08, reload_CS // 0x08 is a stand-in for your code segment
        reload_CS:
           // Reload data segment registers:
           MOV   AX, 0x10 // 0x10 is a stand-in for your data segment
           MOV   DS, AX
           MOV   ES, AX
           MOV   FS, AX
           MOV   GS, AX
           MOV   SS, AX

        RET

    // TEST 

    testJmpAfterGdt:
    // TEST JUMP
            jmp test
        resume:
        ret
    // END TEST JUMP
    
    interrupt_first_handler:
       pushad // -> 32 bit general purpose registers; pusha -> 16 bit 
    
       push esp
       push [interruptnumber]
    
       call handle_interrupt 
       // the return value of the called function will go on the stack
       // the value will be the new stack pointer ?
       // how manage the Istruction Register?
       // maybe we will set it someway
    
       //add esp, 6 // ?? why 6 and not 5? 
       mov esp, eax // set the new stack ptr
    
       popad 
    
    interruptIgnore:
        iret // why this is translated to iretw??

    handleException0x00:
        mov byte ptr [interruptnumber], 0x00
        jmp interrupt_first_handler

    handleException0x06:
        mov byte ptr [interruptnumber], 0x06
        jmp interrupt_first_handler
    
    handleInterruptRequest0x00:
        mov byte ptr [interruptnumber], 0x20
        jmp interrupt_first_handler

    handleInterruptRequest0x01:
        mov byte ptr [interruptnumber], 0x21
        jmp interrupt_first_handler
    // END TEST

.section .data
    gdtr:
        .word 0 // For limit storage
        .long 0 // For base storage
    // TEST
    interruptnumber: .byte 0
    // END TEST
