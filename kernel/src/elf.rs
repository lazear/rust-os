//! Parse the kernel's executable file for advanced error-handling

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct ElfHeader {
    ident: [u8; 16],
    object_type: ElfHeaderType,
    machine_type: u16,
    object_ver: u32,
    entry: usize,
    phdr_off: usize,
    shdr_off: usize,
    flags: u32,
    size: u16,
    phdr_size: u16,
    phdr_len: u16,
    shdr_size: u16,
    shdr_len: u16,
    string_table_idx: u16,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum ElfHeaderType {
    None = 0,
    Relocatable = 1,
    Executable = 2,
    Dynamic = 3,
    Core = 4,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum ProgramHeaderType {
    Null = 0,
    Loadable = 1,
    Dynamic = 2,
    Interpreter = 3,
    Note = 4,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct ProgramHeader {
    ty: ProgramHeaderType,
    flags: u32,
    offset: usize,
    pub vaddr: usize,
    pub paddr: usize,
    file_size: usize,
    pub mem_size: usize,
    align: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Elf<'a> {
    pub header: &'a ElfHeader,
    pub segments: &'a [ProgramHeader],
}

impl<'a> Elf<'a> {
    const ELFMAGIC: [u8; 4] = [0x7F, 'E' as u8, 'L' as u8, 'F' as u8];

    pub fn from(data: &'a [u8]) -> Elf<'a> {
        if data[..Self::ELFMAGIC.len()] != Self::ELFMAGIC {
            panic!("Invalid ELF header!");
        }

        let ehdr = unsafe { &*(data.as_ptr() as *const ElfHeader) };

        println!("{:?} {:0X}", ehdr.object_type, ehdr.entry);

        let segments = unsafe {
            core::slice::from_raw_parts(
                data.as_ptr().offset(ehdr.phdr_off as isize) as *const ProgramHeader,
                ehdr.phdr_len as usize,
            )
        };

        Elf {
            header: ehdr,
            segments,
        }
    }
}
