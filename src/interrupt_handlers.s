//.intel_syntax noprefix

// here should be all the interrupt handler that will manage register and call
// the interrupt handerl function + other thigs

// TODO find a solution to not duplicate this here and in the interrupts.rs
.extern handle_interrupt

.set IRQ_BASE, 0x20

.section text

    .macro HandleException num
    .global handleException\num
    handleException\num:
        mov byte ptr [interruptnumber], \num
        jmp interrupt_first_handler
    .endm
    
    
    .macro HandleInterruptRequest num
    .global handleInterruptRequest\num
    handleInterruptRequest\num:
        push eax

        mov al, \num
        add al, IRQ_BASE

        //mov byte ptr [interruptnumber], \num + IRQ_BASE
        mov byte ptr [interruptnumber], al

        pop eax

        jmp interrupt_first_handler
    .endm

    HandleException 0x00
    
    HandleInterruptRequest 0x00
    HandleInterruptRequest 0x01
    
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
    
    .global interruptIgnore
    interruptIgnore:
       iret


.section .data
    interruptnumber: .byte 0
