#!/bin/sh

cargo build
#~/opt/cross/bin/i686-elf-as -msyntax=intel -mnaked-reg -g src/start.s -o src/start.o
~/opt/cross/bin/i686-elf-ld -T src/linker.ld --gc-sections target/x86_64-gab_os/debug/libkernel.a -o gab_kernel.elf
#~/opt/cross/bin/i686-elf-ld -T src/linker.ld --gc-sections src/start.o target/x86_64-gab_os/debug/libkernel.a -o gab_kernel.elf
qemu-system-i386 -kernel gab_kernel.elf
