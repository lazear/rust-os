#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(asm, panic_info_message, naked_functions)]
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

use prelude::*;

use io::{Io, Serial};
use memory::physical::{Allocator, MemoryMap, MemoryMapInfo};

use core::fmt::Write;
use core::panic::PanicInfo;

#[allow(dead_code)]
#[cfg_attr(not(test), panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    let mut serial = unsafe { Serial::global().force() };
    if let Some(loc) = info.location() {
        serial
            .write_fmt(format_args!(
                "\nPanic occured at file {} {}:{}: ",
                loc.file(),
                loc.line(),
                loc.column()
            ))
            .unwrap();
    } else {
        let args = format_args!("\nPanic occured at unknown location: ");
        serial.write_fmt(args).unwrap();
    }

    if let Some(args) = info.message() {
        let _ = serial.write_fmt(*args);
    }
    loop {}
}

extern "C" {
    static _kernel_start: *const usize;
    static _kernel_end: *const usize;
}

#[cfg(not(test))]
#[no_mangle]
extern "C" fn _start(info: &'static MemoryMapInfo) -> ! {
    arch::interrupts::disable();
    {
        let mut idt = arch::idt::InterruptDescriptorTable::global().lock();
        idt.load();

        idt.register(0x20, arch::interrupts::timer);
    }
    
    let _ = arch::pit::Intel8253::init(10000);
    let pic = arch::pic::Controller::global().lock();

    println!(
        "kernel pages: {:?}",
        paging::TableIndices::from_virt(info.elf_ptr as usize)
    );

    let mut allocator = memory::physical::allocator::BumpAllocator::new(info);

    let ehdr = unsafe { core::slice::from_raw_parts(info.elf_ptr, info.elf_len) };
    let elf = elf::Elf::from(ehdr);

    let cr3 = arch::instructions::cr3();

    let pml4 = unsafe { core::slice::from_raw_parts(cr3 as *const u64, 512) };

    {
        let mut stdout = Serial::global().lock();
        for (idx, entry) in pml4.iter().enumerate() {
            if *entry == 0 {
                continue;
            }
            let mut vaddr: u64 = (idx as u64) << 39;
            let fill = if vaddr.get_bit(47) {
                core::u64::MAX
            } else {
                0u64
            };

            vaddr.set_bits(48..vaddr.bits(), fill);
            stdout
                .write_fmt(format_args!(
                    "PML4 entry {} {:0x} {:#016x} projected {:#016X}\n",
                    idx,
                    entry,
                    vaddr,
                    paging::TableIndices::to_virt(paging::TableIndices {
                        level4: idx,
                        level3: 0,
                        level2: 0,
                        level1: 0,
                    })
                ))
                .unwrap();

            let pml3 = unsafe {
                let p = *entry & !(0xFFF);
                core::slice::from_raw_parts(p as *const u64, 512)
            };

            for (idx3, entry3) in pml3.iter().enumerate() {
                if *entry3 == 0 {
                    continue;
                }
                let mut vaddr3: u64 = vaddr | (idx as u64) << 30;
                stdout
                    .write_fmt(format_args!(
                        "PML3 entry {} {:0x} {:#016x}\n",
                        idx3, entry3, vaddr3
                    ))
                    .unwrap();
            }
        }
    }

    println!("cr3 = 0x{:#016X}", cr3);
    arch::interrupts::enable();

    let mut ptr = 0xDEADBEEF as *mut usize;
    unsafe {
        //asm!("int3");

        //asm!("int3");
        //*ptr = 10;
    }

    println!("Entering final loop");
    loop {}
}
