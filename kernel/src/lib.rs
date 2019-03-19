#![no_std]
#![feature(asm)]

mod io;
mod term;

use core::fmt::Write;
use core::panic::PanicInfo;


#[cfg_attr(not(test), panic_handler)]
fn panic(_info: &PanicInfo) -> ! {
    let mut writer = term::Writer::default();
    // writer.set_color(
    //     term::Color::White,
    //     term::Color::Magenta,
    // );

    writer.write_char('c');

    loop {}
}

static HELLO: &[u8] = b"Hello World from Rust!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    let mut writer = term::Writer::default();

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    writer.write_char('c');
    writer.write_char('c');
    writer.write_char('c');
    writer.write_char('c');

    writer
        .write_str("test str, gello!\nhello from rust")
        .unwrap();

    let loc = 08;
    write!(writer, "\n0x{:0x}", (&loc as *const i32) as usize).unwrap();

    unsafe {
        let bit: *mut usize = 0xFFFF_FFFF_8010_0000 as *mut usize;
        write!(writer, "\n0x{:0x}", *bit);
    }

    panic!("test");

    loop {}
}
