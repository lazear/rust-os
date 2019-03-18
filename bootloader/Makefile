BOOTLOADER = ./bootloader/bootstrap
KERNEL = target/target/release/rust-os
AS	= nasm
ASFLAGS = -f bin
SOURCE = boot.asm elf64.asm
DISK = disk

.PHONY: all

%.o: %.asm

$(BOOTLOADER): $(SOURCE)
	@echo "building bootloader"
	@$(AS) $(ASFLAGS) $< -o $(BOOTLOADER)
	@dd if=/dev/zero of=$(DISK) bs=1k count=2880
	@dd if=$(BOOTLOADER) of=$(DISK) bs=1k count=1 conv=notrunc 

$(KERNEL): src/*.rs 
	@cargo xbuild --target target.json 

all: $(BOOTLOADER)

