
.section .text

// change_page_directory(page_direcotry: u32)
.global change_page_directory
    change_page_directory:
        //push eax
        //mov eax, [esp + 4]
        //mov eax, eax
        mov cr3, eax
        //pop eax
        ret
    
// enable_paging
.global enable_paging
    enable_paging:
        push eax
        mov eax, cr0
        or eax, 0x80000001
        mov cr0, eax
        pop eax
        ret

// flush_tlb_entry(virtual_addr)
.global flush_tlb_entry
    flush_tlb_entry:
        cli
		invlpg [eax]
		sti
        ret

