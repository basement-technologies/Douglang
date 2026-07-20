use std::alloc::{Layout, alloc, dealloc};

use crate::values::tape::*;

pub fn alloc_block(size: BlockSize) -> Result<BlockPtr, AllocError> {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, size);

        let ptr = alloc(layout);
        if ptr.is_null() {
            Err(AllocError::OOM)
        } else {
            Ok(NonNull::new_unchecked(ptr))
        }
    }
}

pub fn dealloc_block(ptr: BlockPtr, size: BlockSize) {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, size);

        dealloc(ptr.as_ptr(), layout);
    }
}
