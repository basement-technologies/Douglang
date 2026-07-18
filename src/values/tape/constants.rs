pub const BLOCK_SIZE_BITS: usize = 15;
pub const BLOCK_SIZE: usize = 1 << BLOCK_SIZE_BITS;

pub const LINE_SIZE_BITS: usize = 7;
pub const LINE_SIZE: usize = 1 << LINE_SIZE_BITS;

pub const ALIGN_MASK: usize = !(size_of::<usize>() - 1);

const TAG_MASK: usize = 0x1;
pub const TAG_OBJECT: usize = 0x0;
pub const TAG_NUMBER: usize = 0x1;
pub const PTR_MASK: usize = !TAG_MASK;
