use crate::io::Serial;
use crate::prelude::*;
use core::panic::PanicInfo;

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

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
