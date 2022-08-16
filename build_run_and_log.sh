#!/bin/sh

cargo build
~/opt/cross/bin/i686-elf-ld -T src/linker.ld --gc-sections target/x86_64-gab_os/debug/libkernel.a -o gab_kernel.elf
qemu-system-i386 -kernel gab_kernel.elf -d int -M q35,smm=off -no-reboot -no-shutdown 

