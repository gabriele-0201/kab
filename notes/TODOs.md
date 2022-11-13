# What I have to do now? 

+ [] Kernel memory managment
    + [x] Split stack\_kernel and heap\_kernel :
        |first 1MiB|kernel_code|kernel_stack|kernel_heap| something... to 4MiB | start_paging_memory ... |
    + [] Decide what's happen if the kernel\_heap go over 4MiB
+ [x] GlobalAllocator
    + [x] Modular heap allocator
+ [] Virtual Memory Management
    + All the stuff related to memory switching, update and switch tables
+ [] Process Management
    + [] ELF parser
    + [] Program Loader
    + [] Switch to user mode
    + [] System API
    + [] PCB (TCB ahahahhaha NO)
    + [] context switch
    + [] Scheduler

