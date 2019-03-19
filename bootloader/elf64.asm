[BITS 64]

global load_elf

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
	xor rax, rax
	mov rax, [rdi + elf64_ehdr.entry]
	;hlt
	call rax
	ret


