[BITS 32]
[global preplongmode]
[global mmap_len]

preplongmode:
	; set CR3 to PML4 address, and clear all entries
	mov edi, [paging.pml4]
	mov cr3, edi 
	xor eax, eax
	mov ecx, 0x1000 
	rep stosd

	; Set PML4[0] -> PDP
	mov edi, [paging.pml4]
	mov eax, [paging.pdp]
	or eax, 3	
	mov [edi], eax
	add edi, 0xFF8
	mov [edi], eax 

	; Set PDP[0] -> PD
	mov edi, [paging.pdp]
	mov eax, [paging.pd]
	or eax, 3
	mov [edi], eax 
	add edi, 0xFF0
	mov [edi], eax

	; Set PD[0] -> PT
	mov edi, [paging.pd]
	mov eax, [paging.pt]
	mov esi, [paging.pt2]
	or eax, 3
	mov [edi], eax
	add edi, 8
	or esi, 3
	mov [edi], esi

	mov edi, [paging.pt]
	mov esi, [paging.pt2]
	mov eax, 3				; map from 0 -> 2 Gb
	mov edx, 0x00200003		; set pagings flags
	mov ecx, 512			; Map an entire page table, 512 entries
	.loop:
		mov [esi], edx 		; esi contains address of PT2
		mov [edi], eax 		; edi contains address of PT1
		add eax, 0x1000 
		add edx, 0x1000 
		add esi, 8			; go to next entry
		add edi, 8 			; go to next entry
		loop .loop			; if ECX != 0 then jmp .loop

	; Set LME bit in EFER MSR, which is bit 8
	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr 

	; Set PAE bit in CR4
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax

	; Set PG bit in CR0
	mov eax, cr0 
	or eax, 1 << 31
	mov cr0, eax
	
	; Reload segment registers
	mov ax, GDT_DATA64			
	mov ds, ax
	mov es, ax
	mov ss, ax
	mov fs, ax
	mov gs, ax
	; Far jump to 64 bit compliant code segment
	jmp GDT_CODE64:long_mode
	hlt
	
[BITS 64]
long_mode:
	mov rax, 0xFFFFFFFF80300000
	mov rsp, rax
	mov rbp, rax
	mov rdi, data
	call load_elf
	
	hlt


STRUC elf64_ehdr
.type		resb 16
.machine	resb 4 
.version	resb 4
.entry		resb 8
.phoff		resb 8
.shoff		resb 8
.flags		resb 4
.ehsize		resb 2
.phentsize	resb 2
.phnum		resb 2
.shentsize	resb 2
.shnum		resb 2
.shstrndx	resb 2
ENDSTRUC

STRUC elf64_phdr
.type		resb 4
.flags		resb 4
.offset		resb 8
.vaddr		resb 8
.paddr		resb 8
.filesz		resb 8
.memsz		resb 8
.align		resb 8
ENDSTRUC

STRUC elf64_shdr
.name		resb 4
.type		resb 4
.flags		resb 8
.addr		resb 8
.offset		resb 8
.size		resb 8
.link		resb 4
.info		resb 4
.addralign	resb 8
.entsize	resb 8
ENDSTRUC

; SysV ABI states that called function must preserve rbp, rbx, and r12-r15
; arguments are passed in rdi, rsi, rdx, rcx, r8d, r9d,... then in stack
load_elf:
	; R8: first program header
	; R9: last program header

	push rdi 								; rdi contains memory address of ELF binary
	xor rax, rax							; clear rax
	xor rcx, rcx							; clear rcx 
	mov r8,  [rdi + elf64_ehdr.phoff]		; load r8 with addr of program header
	mov ax, [rdi + elf64_ehdr.phentsize]	; load rax with size of phdr entry 
	mov cx, [rdi + elf64_ehdr.phnum]		; load rcx with number of phdr's
	mul cx									; rax = (ehdr->e_phentsize * ehdr->e_phnum)
	mov r9, rax								; r9 = rax 
	add r9, r8 								; r9 = end of phdrs, r8 = beginning of phdrs
	xor r10, r10

	; we need rdi for movsb, so switch to using rax for data buffer
	mov rax, rdi							; rax = memory address of ELF binary
	
	; Parse the program headers, and copy to the destination
	.loop:
		; r9 is the last program header, so if current phdr = r9,
		; then we're done looping
		cmp r8, r9
		je .done
		
		mov rcx, [rax + r8 + elf64_phdr.filesz]		; how many bytes to copy
		mov rdx, [rax + r8 + elf64_phdr.offset]		; offset from data
		
		cmp rcx, 0
		je .bss

		; rsi contains the source address
		mov rsi, rax
		add rsi, rdx

		; rdi contains the destination address	
		mov rdi, [rax + r8 + elf64_phdr.paddr]		; physical address requested
		rep movsb

		; increase by sizeof phdr struct
		add r8, 56 
		jmp .loop

	.bss:
		mov rcx, [rax+r8+elf64_phdr.memsz]
		mov rdi, [rax+r8+elf64_phdr.vaddr]
		push rax
		mov rax, 0
		rep stosq
		pop rax


	.done:
	pop rdi
	xor r9, r9
	mov r9, [rdi + elf64_ehdr.entry]
	mov [elf_ptr], rdi
	mov r10, [rdi + elf64_ehdr.ehsize]
	mov [elf_len], r10
	; clear rax, load it with the size of the memory mapping table
	; that we saved at 0x6FF0 earlier.
	; TODO - save this value directly to the memory location of
	; mmap_len
	; regardless, we load it and then divide by the size (24 bytes)
	; to get the number of entries. We can then just directly pass
	; this to the Rust code, which will construct a slice from the
	; ptr, len pair
	;xor rax, rax
	;mov rax, [0x6FF0]
	; only use the lowest byte 
	;mov bl, 24
	;div bl
	;mov [mmap_len], al
	; load rdi with the address of the boot_struct object, rdi is
	; the register in which the first argument is stored in the Sys V
	; ABI, so we can directly access it in the Rust code 
	lea rdi, [boot_struct]
	
	; entry point was stored in r9
	call r9

	; this should be unreachable!
	cli
	hlt


boot_struct:
	mmap_ptr: dq 0x7000
	mmap_len: dq 0	
	elf_ptr: dq 0
	elf_len: dq 0
