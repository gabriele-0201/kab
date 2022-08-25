
// here should be all the interrupt handler that will manage register and call
// the interrupt handerl function + other thigs

// TODO find a solution to not duplicate this here and in the interrupts.rs
.extern handle_interrupt

.set IRQ_BASE, 0x20

.section .text

    .macro HandleException num
    .global handleException\num
    handleException\num:
        mov byte ptr [interruptnumber], \num
        jmp interrupt_first_handler
    .endm
    
    
    .macro HandleInterruptRequest num
    .global handleInterruptRequest\num
    handleInterruptRequest\num:
        mov byte ptr [interruptnumber], \num + IRQ_BASE
        jmp interrupt_first_handler
    .endm

    HandleException 0x00
    HandleException 0x01
    HandleException 0x02
    HandleException 0x03
    HandleException 0x04
    HandleException 0x05
    HandleException 0x06
    HandleException 0x07
    HandleException 0x08
    HandleException 0x09
    HandleException 0x0A
    HandleException 0x0B
    HandleException 0x0C
    HandleException 0x0D
    HandleException 0x0E
    HandleException 0x0F
    HandleException 0x10
    HandleException 0x11
    HandleException 0x12
    HandleException 0x13
    
    HandleInterruptRequest 0x00
    HandleInterruptRequest 0x01
    HandleInterruptRequest 0x02
    HandleInterruptRequest 0x03
    HandleInterruptRequest 0x04
    HandleInterruptRequest 0x05
    HandleInterruptRequest 0x06
    HandleInterruptRequest 0x07
    HandleInterruptRequest 0x08
    HandleInterruptRequest 0x09
    HandleInterruptRequest 0x0A
    HandleInterruptRequest 0x0B
    HandleInterruptRequest 0x0C
    HandleInterruptRequest 0x0D
    HandleInterruptRequest 0x0E
    HandleInterruptRequest 0x0F
    HandleInterruptRequest 0x31

    interrupt_first_handler:
       pushad // -> 32 bit general purpose registers; pusha -> 16 bit 
       push ds
       push es
       push fs
       push gs
    
       push esp
       push interruptnumber // automatically deferenciate
       call handle_interrupt 
    
       // theorically this will jump over the old esp value, the interruptnumber and pointing to the last element
       //add esp, 5 // in viktor code here is 6
       mov esp, eax // set the new stack ptr with the returned value
    
       pop gs
       pop fs
       pop es
       pop ds
       popad 
    
.global interruptIgnore
    interruptIgnore:
       iret

.section .data
    interruptnumber: .byte 0
