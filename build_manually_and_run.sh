#!/bin/sh

cargo build

~/opt/cross/bin/i686-elf-as -msyntax=intel -mnaked-reg -g src/start.s -o src/start.o
~/opt/cross/bin/i686-elf-as -msyntax=intel -mnaked-reg -g src/interrupts/interrupt_handlers.s -o src/interrupts/interrupt_handlers.o
~/opt/cross/bin/i686-elf-as -msyntax=intel -mnaked-reg -g src/memory_manager/memory_manager.s  -o src/memory_manager/memory_manager.o

~/opt/cross/bin/i686-elf-ld -T src/linker.ld --gc-sections src/start.o src/interrupts/interrupt_handlers.o src/memory_manager/memory_manager.o target/x86_64-gab_os/debug/libkernel.a -o gab_kernel.elf

#~/opt/cross/bin/i686-elf-ld -T src/linker_viktor.ld --gc-sections src/start.o target/x86_64-gab_os/debug/libkernel.a -o gab_kernel.elf

qemu-system-i386 -m 500 -kernel gab_kernel.elf
#qemu-system-i386 -kernel gab_kernel.elf -d int -M q35,smm=off -no-reboot -no-shutdown 
#qemu-system-i386 -gdb tcp:localhost:1234 -S -kernel gab_kernel.elf

