use super::*;

#[derive(Clone)]
pub struct BumpBlock {
    cursor: *const u8,
    limit: *const u8,
    block: Block,
    meta: BlockMeta,
}

impl BumpBlock {
    pub fn new() -> Self {
        let block = Block::new(constants::BLOCK_SIZE).expect("Failed to allocate block");
        let meta = BlockMeta::new(block.as_ptr().cast_mut());

        let cursor = unsafe { block.as_ptr().add(constants::BLOCK_SIZE) };
        let limit = block.as_ptr();

        BumpBlock {
            cursor,
            limit,
            block,
            meta,
        }
    }

    pub fn current_hole_size(&self) -> usize {
        self.meta
            .find_next_hole_size(self.block.as_ptr() as usize)
            .unwrap_or(0)
    }

    pub fn inner_alloc(&mut self, alloc_size: usize) -> Option<*const u8> {
        let ptr = self.cursor as usize;
        let limit = self.limit as usize;

        let next_ptr = ptr.checked_sub(alloc_size)? & constants::ALIGN_MASK;
        if next_ptr < limit {
            let block_relative_limit =
                unsafe { self.limit.sub(self.block.as_ptr() as usize) } as usize;

            if block_relative_limit > 0
                && let Some((cursor, limit)) = self
                    .meta
                    .find_next_available_hole(block_relative_limit, alloc_size)
            {
                self.cursor = unsafe { self.block.as_ptr().add(cursor) };
                self.limit = unsafe { self.block.as_ptr().add(limit) };
                return self.inner_alloc(alloc_size);
            }

            None
        } else {
            self.cursor = next_ptr as *const u8;
            Some(self.cursor)
        }
    }
}

struct BlockList {
    head: Option<BumpBlock>,
    overflow: Option<BumpBlock>,
    rest: Vec<BumpBlock>,
}

impl BlockList {
    pub fn new() -> Self {
        Self {
            head: None,
            overflow: None,
            rest: Vec::new(),
        }
    }

    fn overflow_alloc(&mut self, alloc_size: usize) -> Result<*const u8, AllocError> {
        match self.overflow {
            None => {
                let mut overflow = BumpBlock::new();

                let space = overflow
                    .inner_alloc(alloc_size)
                    .expect("We expected this object to fit!");

                self.overflow = Some(overflow);

                Ok(space)
            }
            Some(ref mut overflow) => match overflow.inner_alloc(alloc_size) {
                Some(space) => Ok(space),
                None => {
                    let previous = replace(overflow, BumpBlock::new());

                    self.rest.push(previous);

                    Ok(overflow.inner_alloc(alloc_size).expect("Unexpected error!"))
                }
            },
        }
    }
}

pub struct StickyImmixHeap<H> {
    blocks: UnsafeCell<BlockList>,
    _header_type: PhantomData<*const H>,
}

impl<T> StickyImmixHeap<T> {
    pub fn new() -> Self {
        Self {
            blocks: UnsafeCell::new(BlockList::new()),
            _header_type: PhantomData,
        }
    }

    fn find_space(
        &self,
        alloc_size: usize,
        size_class: SizeClass,
    ) -> Result<*const u8, AllocError> {
        if let SizeClass::Large = size_class {
            return Err(AllocError::BadRequest);
        }

        let blocks = unsafe { &mut *self.blocks.get() };

        match blocks.head {
            None => {
                let mut head = BumpBlock::new();

                let space = head
                    .inner_alloc(alloc_size)
                    .expect("We expected this object to fit");

                blocks.head = Some(head);
                Ok(space)
            }
            Some(ref mut head) => {
                if let SizeClass::Medium = size_class
                    && alloc_size > head.current_hole_size()
                {
                    return blocks.overflow_alloc(alloc_size);
                }

                if let Some(space) = head.inner_alloc(alloc_size) {
                    Ok(space)
                } else {
                    blocks.rest.push(head.clone());
                    let mut head = BumpBlock::new();

                    let space = head
                        .inner_alloc(alloc_size)
                        .expect("We expected this object to fit");
                    blocks.head = Some(head);
                    Ok(space)
                }
            }
        }
    }
}

impl<H: AllocHeader> AllocRaw for StickyImmixHeap<H> {
    type Header = H;

    fn alloc<T>(&self, object: T) -> Result<RawPtr<T>, AllocError>
    where
        T: AllocObject<<Self::Header as AllocHeader>::TypeId>,
    {
        let header_size = size_of::<Self::Header>();
        let object_size = size_of::<T>();
        let total_size = header_size + object_size;

        let alloc_size = total_size & constants::ALIGN_MASK;
        let size_class = SizeClass::get_for_size(alloc_size);

        let space = self.find_space(alloc_size, size_class)?;
        let header = Self::Header::new::<T>(object_size as ArraySize, size_class, Mark::Allocated);

        unsafe {
            write(space as *mut Self::Header, header);
        }

        let object_space = unsafe { space.add(header_size) };
        unsafe { write(object_space as *mut T, object) };

        Ok(RawPtr::new(object_space as *const T))
    }

    fn alloc_array(&self, size_bytes: ArraySize) -> Result<RawPtr<u8>, AllocError> {
        let header_size = size_of::<Self::Header>();
        let total_size = header_size + size_bytes as usize;

        let alloc_size = total_size & constants::ALIGN_MASK;
        let size_class = SizeClass::get_for_size(alloc_size);

        let space = self.find_space(alloc_size, size_class)?;
        let header = Self::Header::new_array(size_bytes, size_class, Mark::Allocated);

        unsafe { write(space as *mut Self::Header, header) };

        let array_space = unsafe { space.add(header_size) };
        let array = unsafe { from_raw_parts_mut(array_space as *mut u8, size_bytes as usize) };

        for byte in array {
            *byte = 0;
        }

        Ok(RawPtr::new(array_space.cast()))
    }

    fn get_header(object: NonNull<()>) -> NonNull<Self::Header> {
        unsafe { NonNull::new_unchecked(object.cast::<Self::Header>().as_ptr().offset(-1)) }
    }

    fn get_object(header: NonNull<Self::Header>) -> NonNull<()> {
        unsafe { NonNull::new_unchecked(header.as_ptr().offset(1).cast::<()>()) }
    }
}

impl TypeHeader {
    pub unsafe fn get_object_fatptr(&self) -> FatPtr {
        match self.type_id {
            TypeList::Array => FatPtr::Array(RawPtr::new(
                StickyImmixHeap::get_object(self.into()).as_ptr().cast(),
            )),
            TypeList::String => FatPtr::String(RawPtr::new(
                StickyImmixHeap::get_object(self.into()).as_ptr().cast(),
            )),
            TypeList::Function => FatPtr::Function(RawPtr::new(
                StickyImmixHeap::get_object(self.into()).as_ptr().cast(),
            )),
            _ => panic!("Using object tag with type that isn't an object"),
        }
    }
}
