pub fn cs() -> u16 {
    let cs: u16;
    unsafe { asm!("mov $0, cs" : "=r"(cs) ::: "intel", "volatile") }
    cs
}

pub fn cr3() -> u64 {
    let cr3: u64;
    unsafe { asm!("mov $0, cr3" : "=r"(cr3) ::: "intel", "volatile") }
    cr3
}
