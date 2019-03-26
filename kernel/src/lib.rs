#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(asm, panic_info_message, naked_functions)]
#![feature(lang_items)]
#![allow(dead_code)]

#[macro_use]
pub mod sync;
#[macro_use]
pub mod prelude;
#[macro_use]
pub mod arch;
pub mod elf;
pub mod io;
pub mod memory;
pub mod paging;
pub mod term;
pub mod timer;

mod panic;

use memory::physical::MemoryMapInfo;
use prelude::*;

#[cfg(not(test))]
#[no_mangle]
extern "C" fn _start(info: &'static MemoryMapInfo) -> ! {
    arch::interrupts::disable();
    arch::devices::init();

    {
        let mut idt = arch::idt::InterruptDescriptorTable::global().lock();
        idt.load();
        idt.register(0x20, timer::timer);
    }

    println!(
        "kernel pages: {:?}",
        paging::TableIndices::from_virt(info.elf_ptr as usize)
    );

    let mut allocator = memory::physical::allocator::BumpAllocator::new(info);

    let ehdr = unsafe { core::slice::from_raw_parts(info.elf_ptr, info.elf_len) };
    let elf = elf::Elf::from(ehdr);
    println!("{:?}", elf.header);
    for section in elf.sections {
        println!("{:?}", section);
    }
    elf.symbol();

    let cr3 = arch::instructions::cr3();

    println!("cr3 = 0x{:#016X}", cr3);
    arch::interrupts::enable();

    println!("Entering final loop");
    loop {}
}
