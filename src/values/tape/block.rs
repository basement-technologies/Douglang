use crate::values::tape::*;

pub type BlockPtr = NonNull<u8>;
pub type BlockSize = usize;

#[derive(Clone)]
pub struct Block {
	ptr: BlockPtr,
	_size: BlockSize,
}

impl Block {
	pub fn new(size: BlockSize) -> Result<Block, AllocError> {
		if !size.is_power_of_two() {
			return Err(AllocError::BadRequest);
		}

		Ok(Block {
			ptr: internal::alloc_block(size)?,
			_size: size,
		})
	}

	pub fn as_ptr(&self) -> *const u8 {
		self.ptr.as_ptr()
	}
}

impl Drop for Block {
	fn drop(&mut self) {
		internal::dealloc_block(self.ptr, self._size);
	}
}

#[derive(Clone, Copy)]
pub struct BlockMeta {
	lines: *mut u8,
}

impl BlockMeta {
	/// Finds the next allocatable hole by bump-allocating downards, returning the end pointer and
	/// the limit of blocks that can be used. If no suitable hole is found, return [`None`].
	pub fn find_next_available_hole(
		&self,
		starting_at: usize,
		alloc_size: usize,
	) -> Option<(usize, usize)> {
		// the cont of available holes.
		let mut count = 0;
		let starting_line = starting_at / constants::LINE_SIZE;
		let lines_required = alloc_size.div_ceil(constants::LINE_SIZE);
		let mut end = starting_line;

		for index in (0..starting_line).rev() {
			let marked = unsafe { *self.lines.add(index) };

			if marked == 0 {
				// count unmarked lines
				count += 1;

				if index == 0 && count >= lines_required {
					let limit = index * constants::LINE_SIZE;
					let cursor = end * constants::LINE_SIZE;
					return Some((cursor, limit));
				}
			} else {
				// This block is marked
				if count > lines_required {
					// But at least 2 previous blocks were not marked. Return the hole, considering the
					// immediately preceding block as conservatively marked
					let limit = (index + 2) * constants::LINE_SIZE;
					let cursor = end * constants::LINE_SIZE;
					return Some((cursor, limit));
				}

				// If this line is marked and we didn't return a new cursor/limit pair by now,
				// reset the hole search state
				count = 0;
				end = index;
			}
		}

		None
	}

	pub fn find_next_hole_size(&self, starting_at: usize) -> Option<usize> {
		let mut count = 0;
		let starting_line = starting_at / constants::LINE_SIZE;
		let mut end = starting_line;

		for index in (0..starting_line).rev() {
			let marked = unsafe { *self.lines.add(index) };

			if marked == 0 {
				count += 1;

				if index == 0 {
					let limit = index * constants::LINE_SIZE;
					let cursor = end * constants::LINE_SIZE;
					return Some(cursor - limit);
				}
			} else {
				if count > 1 {
					let limit = (index + 2) * constants::LINE_SIZE;
					let cursor = end * constants::LINE_SIZE;
					return Some(cursor - limit);
				}

				count = 0;
				end = index;
			}
		}

		None
	}

	pub fn new(ptr: *mut u8) -> Self {
		Self { lines: ptr }
	}
}
