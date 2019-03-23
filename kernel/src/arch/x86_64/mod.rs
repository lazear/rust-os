pub mod idt;
pub mod instructions;
pub mod interrupts;

#[repr(u16)]
pub enum PrivilegeLevel {
    Ring0,
    Ring1,
    Ring2,
    Ring3,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C, packed)]
pub struct DescriptorTablePtr {
    limit: u16,
    base: usize,
}
