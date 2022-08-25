# Interrupt

Managed by an IDT formed by entry with some specific values,
those entry rapresent the handle interrupt of the ith entry

0x20 is the interrupt related to the clock

## PIC

There is the slave pic and the master pic

The pic have a IMR (Interrupt Mask Register) used to set some bit (8 bit total)
to make some mask on some interrupt -> this will be like an ignore interrupt

There is also two register, In-Service Register and Interrupt Request Resgister, 
the first one contain the current servicin interrupt instead the second contain all the 
interrupts that will be managed

Spurious IRQs -> maybe go deeper if needed

# Interrupt Tutorial

-> What I've done:
    + Set up the idt point to handleInterruptRequestNUM 
    + those function call a common function called first_interrupt_handler
    + this function will set up a stack e correctly call a rust function: handle_interrupt
    + this fn using a static pointer will be called a function inside the struct IDT
        + this fucntion should call the correct handler based on the interrrupt number
...
 What to do next

After completing this tutorial, there is still much left for you to do to fully harness the power of interrupts.

You can:

    + Initialize the PIC
    + Make use of PIC IRQs
    + Understand the NMI
    + Configure the local APIC
    + Write a driver for the IOAPIC
    + Make use of Message Signaled Interrupts

# KEYBOARD:

