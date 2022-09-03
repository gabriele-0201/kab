# Memory Protection

### Virtual and Physical Memory

## Segmentation

 -> External Fragmentation problem

## Paging

Virtual Memory divided in **pages**
Physical Memory divided in **frames**

 -> Internal Fragmentation
    + this type of fragmentation is predictable, half size of a page every program that need to be paginated

### Page Tables

+ Each program instance has its own page table
+ The register CR3 contain the pointer to the page table
    + Loading the corrent page table for each program instance is a job of the page table
+ All the transaltion done from virtual to physical are done by the architecture and extremely fast
+ **Multi level page table**

## Paging on x86_64 (64 bit)

There is currenty 4 multilevel page -> 512 entries each one -> so 9 bit is needed to index an antry

Page are 4KiB -> 12 bit

so 12 + 9 * 4 = 48 -> from 48th bit to 64th those follow the sign-extension (all equal to the 47th)
This si needed to create uniqness -> usefeull also to allow in a future extend le multileveling

Attenction that each page table contain Physical address otherwise those should be converted and this would cause and infinete recursion.

## Paging (32 bit)

Come funziona logicamente il multitasking?

Tutto parte dal timer interrupt => quanto un interrupt generico accade vengono salvati nello stack tutte le info importanti dei registri,
dall'istruction pointer precedente, tutti segment register eccc.
quando noi torniamo dall'interrupt handler torniamo un nuovo puntatore di stack, per ora e' sempre lo stesso MA
la cosa figa che puo' essere fatta e' quella di preparare un NUOVO stack con tutte le informazioni necessarie al fine di:
+ tornare lo stack pointer del nuovo stack
+ finito l'handling dell'interrupt si seguiranno le solite prassi per tornare allo scope precedente nello stack solo che avendo cambiato completamente stack possiamo cambiare proprio quello che si stava facendo
+ eseguendo cosi un cazzo di task switch (cosi si puo' definire il contex switch?)


