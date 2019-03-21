OUTPUT_FORMAT("elf64-x86-64")

KERNEL_VIRT   = 0xFFFFFFFF80000000;
KERNEL_PHYS   = 0x100000;

ENTRY(_start)
SECTIONS
{

    . = KERNEL_VIRT + KERNEL_PHYS;
    . += SIZEOF_HEADERS;
    
    _kernel_start = .;
    .text ALIGN(0x1000) : AT(ADDR(.text) - KERNEL_VIRT)
    {
        *(.text .text.*)
    }

    .rodata ALIGN(0x1000) : AT(ADDR(.rodata) - KERNEL_VIRT) 
    {
        *(.rodata .rodata.*)
    }

   .data ALIGN(0x1000) : AT(ADDR(.data) - KERNEL_VIRT)
   {
        *(.padata)
        . = ALIGN(0x1000);
        *(.data .data.*)
   }

   .bss ALIGN(0x1000): AT(ADDR(.bss) -  KERNEL_VIRT)
   {
       *(.bss .bss.*)
        *(COMMON)
   }

   _kernel_end = .;


   /DISCARD/ :
   {
        *(.comment)
        *(.note .note.*)
    }
}

