use crate::prelude::*;
use core::fmt;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Virtual(usize);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Physical(usize);

/// Helper struct that holds the index in each level of the page tables
/// for a virtual address
#[derive(Copy, Clone, PartialEq)]
pub struct TableIndices {
    pub level4: usize,
    pub level3: usize,
    pub level2: usize,
    pub level1: usize,
}

impl fmt::Debug for TableIndices {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "level 4: {:0X}\nlevel 3: {:0X}\nlevel 2: {:0X}\nlevel 1: {:0X}",
            self.level4, self.level3, self.level2, self.level1
        )
    }
}

impl TableIndices {
    pub fn from_virt(vaddr: usize) -> TableIndices {
        TableIndices {
            level4: (vaddr & (0x1FF << 39)) >> 39,
            level3: (vaddr & (0x1FF << 30)) >> 30,
            level2: (vaddr & (0x1FF << 21)) >> 21,
            level1: (vaddr & (0x1FF << 12)) >> 12,
        }
    }

    pub fn to_virt(self) -> usize {
        let mut base =
            (self.level4 << 39) | (self.level3 << 30) | (self.level2 << 21) | (self.level1 << 12);
        let fill = if base.get_bit(47) {
            core::usize::MAX
        } else {
            0
        };
        base.set_bits(47..64, fill);
        base
    }
}
