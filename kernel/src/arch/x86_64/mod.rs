pub mod idt;
pub mod intrinsics;

#[repr(u16)]
pub enum PrivilegeLevel {
    Ring0,
    Ring1,
    Ring2,
    Ring3,
}