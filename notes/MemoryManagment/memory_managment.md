# MemoryManagment

## Identity Mapping

This will cover the first page entry, so the first pageTable
From 0 to 4MiB

## HHF

This will Map from 3GiB to 1MiB and cover max 1GiB
Note all the GiB will be covered but only based on the kernel dimension,
the PageTable that will be needed are:

kernel\_dim = kernel\_heap\_end - kernel\_start (1MiB)
num\_page\_required = floor((kernel\_dim / page\_size) / entry\_per\_page)

where page\_size = 4MiB and entry\_per\_page = 1024

Those page table will be pointed by every page\_directory 
SO those page will be allocated and almost ever deallocated I think

On every new virtual address creation the entry for the HHF will be
copied by the previous one

At the beginning will be ready only the Identity and HHF
the problem now is: how the kernel start working using the HHF?
