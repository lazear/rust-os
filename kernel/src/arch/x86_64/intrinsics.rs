
pub fn cs() -> u16 {
    let cs: u16;
    unsafe {
        asm!("mov %cs, $0" : "=r"(cs))
    }
    cs
}

pub fn cr3() -> u64 {
    let cr3: u64;
    unsafe {
        asm!("mov %cr3, $0" : "=r"(cr3))
    }
    cr3
}

pub fn enable_interrupts() {
    unsafe {
        asm!("sti" :::: "volatile");
    }
}

pub fn disable_interrupts() {
    unsafe {
        asm!("cli" :::: "volatile");
    }
}