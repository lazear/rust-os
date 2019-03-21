;==============================================================================
;MIT License
;Copyright (c) 2007-2019 Michael Lazear
;
;Permission is hereby granted, free of charge, to any person obtaining a copy
;of this software and associated documentation files (the "Software"), to deal
;in the Software without restriction, including without limitation the rights
;to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
;copies of the Software, and to permit persons to whom the Software is
;furnished to do so, subject to the following conditions:
;
;The above copyright notice and this permission notice shall be included in all
;copies or substantial portions of the Software.
;
;THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
;IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
;FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
;AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
;LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
;OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
;SOFTWARE.
;==============================================================================
; Initial bootloader to take us all the way to 64 bit mode
; 16->32 bit:
;	- disable interrupts
;	- enable A20 line
;	- load GDTR 
;	- set PE (bit 0) of CR0
;	- far jmp to CS32
;	- reload segment registers
; 32->64 bit:
;	- disable paging: bit 31 of CR0
;	- enable PAE: bit 5 of CR4 
;	- load CR3 with physical address of PML4 
;	- enable IA-32E: bit 8 of EFER MSR (0xC0000080)
;	- enable paging: bit 31 of CR0 
;	- jmp to CS with 64 bit descriptor flag enabled
;==============================================================================
; 0x00000000 - 0x000003FF Real mode IVT
; 0x00000400 - 0x000004FF Bios Data Area
; 0x00000500 - 0x00007BFF Guaranteed free for use, memory map placed @ 0x7000 
; 0x00007C00 - 0x00008000 1kb bootloader (this file)
; 0x00008000 - 0x0009FBFF Reserved for second stage bootloader 
; 0x0009FBFF - 0x00100000 EBDA, ROM, Video memory
; 0x00100000 - ...        Reserved for kernel


[BITS 16]
[ORG 0x7C00]
jmp 0:entry

; 0x0000:0x7C00
entry:

	; disable interrupts and set all segments to 0
	cli
	xor ax, ax
	mov ds, ax
	mov es, ax
	mov fs, ax
	mov gs, ax

	; Bochs requires 64K of stack space to read 128 sectors,
	; so we setup the stack segment to give us this space
	mov ax, 0x8c00
	mov ss, ax
	mov sp, 0

	; enable A20 line
	in al, 0x92
	or al, 2
	out 0x92, al

	; now prepare to read the read the next sector which contains
	; the rest of the code for the bootloader
	mov [drive], dl
	mov ax, 0
	int 13h
	call read_disk
	

	;call video_mode


	; memory mapping function
	mov di, 0x7000
	call memory_map
	mov ax, di 
	sub ax, 0x7000
	mov di, 0x6FF0
	mov [di], ax

	; clear ax and load the Global Descriptor Table
	xor ax, ax
	lgdt [gdt_desc] 	

	; Set bit 1 of CR0 - enable 32 bit mode
	mov eax, cr0
	or eax, 1 
	mov cr0, eax

	; Flush CS by performing a jump to new code segment
	jmp GDT_CODE32:protected_mode


read_disk:
	mov ax, 0x07e0
	mov si, packet		; address of "disk address packet"
	mov ah, 0x42		; extended read
	mov dl, [drive]		; drive number 0 (OR the drive # with 0x80)
	int 0x13
	jc .error
	ret
	.error:
		hlt

; Reserve some space for our disk packet
packet:
	db	0x10	; packet size (16 bytes)
	db	0		; always 0
.count:		
	dw	120		; number of sectors to transfer
.dest:		
	dw	0		; destination offset (0:7c00)
	dw	0x7e0	; destination segment
.lba:
	dd	1		; put the lba  to read in this spot
	dd	0		; more storage bytes only for big lba
drive db 0

; Bios function INT 15h, AX=E820h
; EBX must contain 0 on first call, and remain unchanged
; 	on subsequent calls until it is zero again
; Code adapted from OSDEV wiki
; Memory map is 24 byte struct placed at [ES:DI]
memory_map:
	xor ebx, ebx				; ebx must be 0 to start
	mov edx, 0x0534D4150		; Place "SMAP" into edx
	mov eax, 0xE820
	mov [es:di + 20], dword 1	; force a valid ACPI 3.X entry
	mov ecx, 24					; ask for 24 bytes
	int 0x15
	jc short .fail				; carry set on first call means "unsupported function"

	mov edx, 0x0534D4150		; 
	cmp eax, edx				; on success, eax must have been set to "SMAP"
	jne short .fail

	test ebx, ebx				; ebx = 0 implies list is only 1 entry long (worthless)
	je short .fail
	jmp short .loop

	.e820lp:
		mov eax, 0xe820				; eax, ecx get trashed on every int 0x15 call
		mov [es:di + 20], dword 1	; force a valid ACPI 3.X entry
		mov ecx, 24					; ask for 24 bytes again
		int 0x15
		jc short .done				; carry set means "end of list already reached"
		mov edx, 0x0534D4150		; repair potentially trashed register
	.loop:
		jcxz .skip					; skip any 0 length entries
		cmp cl, 20					; got a 24 byte ACPI 3.X response?
		jbe short .notext
		test byte [es:di + 20], 1	; if so: is the "ignore this data" bit clear?
		je short .skip
	.notext:
		mov ecx, [es:di + 8]		; get lower uint32_t of memory region length
		or ecx, [es:di + 12]		; "or" it with upper uint32_t to test for zero
		jz .skip					; if length uint64_t is 0, skip entry
		add di, 24
		
	.skip:
		test ebx, ebx				; if ebx resets to 0, list is complete
		jne short .e820lp
	.done:
		clc							; there is "jc" on end of list to this point, so the carry must be cleared
		ret
	.fail:
		stc							; "function unsupported" error exit
		ret

;==============================================================================
; Disk sector 1
;==============================================================================
[BITS 32]
protected_mode:
	; clear VGA display
	mov eax, 0x0700
	mov ecx, 80*25
	mov edi, 0xB8000
	rep stosw
	xor eax, eax

	; Reload segment registers
	mov ax, GDT_DATA32 			
	mov ds, ax
	mov es, ax
	mov ss, ax
	mov fs, ax
	mov gs, ax

	; check to make sure we can use extended functions
	mov eax, 0x80000000
	cpuid 
	cmp eax, 0x80000001 
	jb nolongmode 

	mov eax, 0x80000001
	cpuid 

	; test bit 29 - x86_64 enabled
	test edx, 1 << 29
	jz nolongmode
	jmp preplongmode

nolongmode:
	mov esi, msg_nolong
	mov ecx, 0xB8000
	.loop:
		; move byte at address DS:ESI into AL
		lodsb

		; test for NULL terminated string
		or al, al 
		jz .done
		mov [ecx], al
		add ecx, 2
		jmp .loop
	.done:
	hlt 


;==============================================================================
paging:
	.pml4 	dd 0x003F1000 	; PML4
	.pdp	dd 0x003F2000	; Page directory pointer
	.pd 	dd 0x003F3000	; Page directory 
	.pt 	dd 0x003F4000	; Page table, 0x00000000-0x00200000
	.pt2	dd 0x003F5000

msg_long 		db "Succesfully made it to long mode!", 0
msg_nolong 		db "Error: processor is not x86_64 enabled!", 0

;;; GLOBAL DESCRIPTOR TABLE
;;; Use a very simply GDT with both 32 and 64 bit segments just to bootstrap
align 32
gdt_null:
	dd 0
	dd 0

GDT_CODE32 equ $ - gdt_null
	dw 0xFFFF 	; Limit 0xFFFF
	dw 0		; Base 0:15
	db 0		; Base 16:23
	db 0x9A 	; Present, Ring 0, Code, Non-conforming, Readable
	db 0xCF		; Page-granular
	db 0 		; Base 24:31

GDT_DATA32 equ $ - gdt_null               
	dw 0xFFFF 	; Limit 0xFFFF
	dw 0		; Base 0:15
	db 0		; Base 16:23
	db 0x92 	; Present, Ring 0, Code, Non-conforming, Readable
	db 0xCF		; Page-granular
	db 0 		; Base 24:31

GDT_CODE64 equ $ - gdt_null
    dw 0
    dw 0
    db 0
    db 0x9A
    db 0x20
    db 0

GDT_DATA64 equ $ - gdt_null
    dw 0
    dw 0
    db 0
    db 0x92
    db 0x20
    db 0

gdt_desc:					; The GDT descriptor
	dw $ - gdt_null - 1		; Limit (size)
	dd gdt_null 			; Address of the GDT


times 510-($-$$) db 0 		; Fill up the file with zeros
dw 0xAA55 					; Last 2 bytes = Boot sector identifyer

%include "stage2.asm"

times 1024-($-$$) db 0 		; Fill up the file with zeros

data: