//! Parse the kernel's executable file for advanced error-handling

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Header {
    ident: [u8; 16],
    object_type: HeaderType,
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
pub enum HeaderType {
    None = 0,
    Relocatable = 1,
    Executable = 2,
    Dynamic = 3,
    Core = 4,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum SegmentType {
    Null = 0,
    Loadable = 1,
    Dynamic = 2,
    Interpreter = 3,
    Note = 4,
    Reserved = 5,
    ProgramHeaderTable = 6,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Segment {
    ty: SegmentType,
    flags: u32,
    offset: usize,
    pub vaddr: usize,
    pub paddr: usize,
    file_size: usize,
    pub mem_size: usize,
    align: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum SectionType {
    Null = 0,
    Program = 1,
    Symbols = 2,
    Strings = 3,
    Relocation_a = 4,
    Hash = 5,
    Dynamic = 6,
    Note = 7,
    Uninitialized = 8,
    Relocation = 9,
    Reserved = 10,
    DynamicSymbol = 11,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Section {
    /// Offset, in bytes, to the section name, relative to the start
    /// of the section name string table
    name: u32,
    /// Section type
    ty: SectionType,
    /// Section attributes
    flags: usize,
    /// Virtual address of the beginning of the section in memory
    vaddr: usize,
    /// Offset, in bytes, of the beginning of the section contents of the file
    offset: usize,
    /// Size, in bytes, of the section
    size: usize,
    /// Section index of an associated section
    link: u32,
    /// Extra information about the section
    info: u32,
    /// Required alignment of the section. invariant that it is power of 2
    align: usize,
    /// size, in bytes, of each entry, for sections that contain fixed-size
    /// entries.
    entry_size: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
pub struct Symbol {
    /// Offset, in bytes, to the symbol name, relative to the start of the
    /// symbol string table. If zero, symbol has no name
    name: u32,
    /// Symbol type and scope
    info: u8,
    res: u8,
    /// Section index of the section in which the symbol is defined
    idx: u16,
    /// Value of the symbol. May be absolute or relocatable
    value: usize,
    /// Size associated with the symbol
    size: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Elf<'a> {
    pub header: &'a Header,
    pub segments: &'a [Segment],
    pub sections: &'a [Section],
}

impl<'a> Elf<'a> {
    const ELFMAGIC: [u8; 4] = [0x7F, 'E' as u8, 'L' as u8, 'F' as u8];

    pub fn symbol(&self) -> &'a str {
        let symtab = self
            .sections
            .iter()
            .filter(|s| s.ty == SectionType::Strings)
            .next()
            .unwrap();

        unsafe {
            let ptr = (self.header as *const _ as *const Section).offset(symtab.offset as isize);
            println!("symtab pointer {:0X}", ptr as usize);
        }

        ""
    }

    /// TODO: pointer alignment issues?
    pub fn from(data: &'a [u8]) -> Elf<'a> {
        if data[..Self::ELFMAGIC.len()] != Self::ELFMAGIC {
            panic!("Invalid ELF header!");
        }

        let ehdr = unsafe { &*(data.as_ptr() as *const Header) };

        assert_eq!(ehdr.shdr_size as usize, core::mem::size_of::<Section>());
        let segments = unsafe {
            core::slice::from_raw_parts(
                data.as_ptr().offset(ehdr.phdr_off as isize) as *const Segment,
                ehdr.phdr_len as usize,
            )
        };

        let sections = unsafe {
            core::slice::from_raw_parts(
                data.as_ptr().offset(ehdr.shdr_off as isize) as *const Section,
                ehdr.shdr_len as usize,
            )
        };

        Elf {
            header: ehdr,
            segments,
            sections,
        }
    }
}
