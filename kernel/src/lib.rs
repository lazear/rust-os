#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(asm, panic_info_message, naked_functions)]

#[macro_use]
mod sync;

#[macro_use]
mod prelude;

#[macro_use]
mod arch;

mod io;

mod term;

use prelude::*;

use io::{Io, Serial};
use sync::Global;
use term::Color;

use core::fmt::Write;
use core::panic::PanicInfo;

#[allow(dead_code)]
#[cfg_attr(not(test), panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    let mut serial = Serial::global().lock();
    let vga = unsafe { term::Terminal::global().force() };
    vga.set_color(Color::Red, Color::Black);

    if let Some(loc) = info.location() {
        vga.write_fmt(format_args!(
            "\nPanic occured at file {} {}:{}: ",
            loc.file(),
            loc.line(),
            loc.column()
        ))
        .unwrap();
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
        vga.write_fmt(args).unwrap();
        serial.write_fmt(args).unwrap();
    }

    if let Some(args) = info.message() {
        let _ = vga.write_fmt(*args);
        let _ = serial.write_fmt(*args);
    }
    loop {}
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello from Rust!");
    unsafe {
        let bit: *mut usize = 0xFFFF_FFFF_8010_1000 as *mut usize;
        println!("\nbytes at 0x{:0x} = 0x{:0x}", bit as usize, *bit);
    }
    
    {
        let idt = arch::idt::InterruptDescriptorTable::global().lock();
        //println!("idt @ {:#016X}", &*idt as *const _ as usize);
    }
    
    
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
                    "PML4 entry {} {:0x} {:#016x}\n",
                    idx, entry, vaddr
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
    loop {}
}
