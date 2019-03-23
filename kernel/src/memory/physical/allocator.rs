use super::*;

#[derive(Debug)]
pub struct BumpAllocator {
    first_frame: Frame,
    last_frame: Frame,
    next_frame: Option<Frame>,
}

impl BumpAllocator {
    pub fn new(info: &MemoryMapInfo) -> BumpAllocator {
        /// Unsafe because we are trusting that the bootloader has given us
        /// the correct pointer and length to the memory map
        let regions = unsafe { core::slice::from_raw_parts(info.ptr, info.len) };

        let mut alloc = BumpAllocator {
            first_frame: Frame { physical_addr: 0 },
            last_frame: Frame { physical_addr: 0 },
            next_frame: None,
        };

        for r in regions {
            if r.region_type == RegionType::Usable {
                alloc = BumpAllocator {
                    first_frame: Frame {
                        physical_addr: r.base,
                    },
                    last_frame: Frame {
                        physical_addr: r.base + r.len,
                    },
                    next_frame: Some(Frame {
                        physical_addr: r.base,
                    }),
                };
            }
        }
        alloc
    }
}

impl Allocator for BumpAllocator {
    fn allocate(&mut self) -> Option<Frame> {
        match self.next_frame.take() {
            Some(frame) => {
                if frame == self.last_frame {
                    self.next_frame = None;
                    Some(frame)
                } else {
                    self.next_frame = Some(Frame {
                        physical_addr: frame.physical_addr + 0x1000,
                    });
                    Some(frame)
                }
            }
            None => None,
        }
    }

    fn deallocate(&mut self, frame: Frame) {
        unimplemented!()
    }
}
