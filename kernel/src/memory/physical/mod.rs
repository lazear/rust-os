pub mod allocator;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum RegionType {
    Usable = 1,
    Reserved = 2,
    Reclaimable = 3,
    NVS = 4,
    BadMemory = 5,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct MemoryMap {
    base: usize,
    len: usize,
    region_type: RegionType,
    acpi_attributes: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct MemoryMapInfo {
    ptr: *const MemoryMap,
    len: usize,
    pub elf_ptr: *const u8,
    pub elf_len: usize,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Frame {
    physical_addr: usize,
}

pub trait Allocator {
    fn allocate(&mut self) -> Option<Frame>;
    fn deallocate(&mut self, frame: Frame);
}
