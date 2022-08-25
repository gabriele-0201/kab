#!/bin/sh

cargo build
~/opt/cross/bin/i686-elf-ld -T src/linker_viktor.ld --gc-sections src/start.o target/x86_64-gab_os/debug/libkernel.a -o gab_kernel.elf
#qemu-system-i386 -kernel gab_kernel.elf
#qemu-system-i386 -kernel gab_kernel.elf -d int -M q35,smm=off -no-reboot -no-shutdown 
qemu-system-i386 -gdb tcp:localhost:1234 -S -kernel gab_kernel.elf

