use crate::io::Serial;
use crate::prelude::*;
use crate::term::{Color, Terminal};
use core::panic::PanicInfo;

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[allow(dead_code)]
#[cfg_attr(not(test), panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    fn write_info<W: core::fmt::Write>(dest: &mut W, info: &PanicInfo) {
        if let Some(loc) = info.location() {
            dest.write_fmt(format_args!(
                "\nPanic occured at file {} {}:{}: ",
                loc.file(),
                loc.line(),
                loc.column()
            ))
            .unwrap();
        } else {
            let args = format_args!("\nPanic occured at unknown location: ");
            dest.write_fmt(args).unwrap();
        }

        if let Some(args) = info.message() {
            let _ = dest.write_fmt(*args);
        }
    }

    crate::arch::interrupts::disable();
    let mut serial = unsafe { Serial::global().force() };
    let mut terminal: &mut Terminal = unsafe { Terminal::global().force() };

    terminal.set_color(Color::White, Color::Red);

    write_info(serial, info);
    write_info(terminal, info);

    loop {}
}
