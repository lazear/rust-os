#![no_std]
#![feature(asm, panic_info_message)]

mod io;
mod term;

use io::{Io, Serial};
use term::Color;

use core::fmt::Write;
use core::panic::PanicInfo;


#[cfg_attr(not(test), panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    let mut serial = Serial::default();
    
    let _ = if let Some(loc) = info.location() {
        write!(serial, "Panic occured at file {} {}:{}\n", loc.file(), loc.line(), loc.column())
    } else {
        write!(serial, "Panic:\n")
    };
    if let Some(args) = info.message() {
        let _ = serial.write_fmt(*args);
    }    
    loop {}
}


#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut writer = term::Writer::default();
    writer.set_color(Color::White, Color::Magenta);
    writer
        .write_str("Hello from rust")
        .unwrap();
    
    writer.set_color(Color::LightGray, Color::Black);

    unsafe {
        let bit: *mut usize = 0xFFFF_FFFF_8010_0000 as *mut usize;
        write!(writer, "\nbytes at 0x{:0x} = 0x{:0x}", bit as usize, *bit);
    }

    loop {}
}
