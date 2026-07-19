mod alloc;
mod block;
mod bump;
mod constants;
mod internal;
mod pointers;

pub use alloc::*;
use block::*;
pub use bump::StickyImmixHeap;
pub use pointers::*;

use std::{
    cell::{Cell, UnsafeCell},
    collections::HashMap,
    marker::PhantomData,
    mem::replace,
    ops::Deref,
    ptr::{NonNull, write},
    slice::from_raw_parts_mut,
};

use thiserror::Error;

use crate::{
    runtime::RuntimeError,
    values::{
        Literal,
        hashers::BuildFxHasher,
        value::{Array, FiveMinuteCodingAdventure, Nil, Text},
    },
};

#[derive(Clone, Copy)]
pub enum Mark {
    Allocated,
}

#[derive(Clone, Copy)]
pub enum SizeClass {
    Small,
    Medium,
    Large,
}

impl SizeClass {
    pub fn get_for_size(size: usize) -> Self {
        match (size > constants::LINE_SIZE, size > constants::BLOCK_SIZE) {
            (false, _) => Self::Small,
            (_, false) => Self::Medium,
            _ => Self::Large,
        }
    }
}

#[derive(Error, Debug, Clone, Copy)]
pub enum AllocError {
    #[error("Bad request")]
    BadRequest,
    #[error("Out of memory")]
    OOM,
}

pub trait AllocTypeId: Copy + Clone {}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeList {
    Number,
    String,
    Boolean,
    FiveMinuteCodingAdventure,
    Array,
}

impl AllocTypeId for TypeList {}

#[allow(dead_code)]
struct TypeHeader {
    size: u32,
    mark: Mark,
    type_id: TypeList,
    size_class: SizeClass,
}

impl AllocHeader for TypeHeader {
    type TypeId = TypeList;

    fn new<O: AllocObject<Self::TypeId>>(size: u32, size_class: SizeClass, mark: Mark) -> Self {
        Self {
            size,
            mark,
            type_id: O::TYPE_ID,
            size_class,
        }
    }

    fn new_array(size: super::tape::ArraySize, size_class: SizeClass, mark: Mark) -> Self {
        Self {
            size,
            mark,
            type_id: TypeList::Array,
            size_class,
        }
    }

    fn type_id(&self) -> Self::TypeId {
        self.type_id
    }
}

/// This enum defines an implementor of the [`AllocTypeId`] trait to be used when allocating memory
/// in the heap for Literals.
#[derive(Copy, Clone)]
pub enum LiteralList {
    Number,
    String,
    Invalid,
}

/// A simple header implementor of [`AllocHeader`], used for Read Only data for storing literals in
/// the heap.
#[allow(dead_code)]
pub struct LiteralHeader {
    size: u32,
    mark: Mark,
    type_id: LiteralList,
    size_class: SizeClass,
}

pub struct ROData<'mem> {
    heap: &'mem StickyImmixHeap<LiteralHeader>,
}

impl Clone for ROData<'_> {
    fn clone(&self) -> Self {
        Self { heap: self.heap }
    }
}

impl AllocTypeId for LiteralList {}

impl AllocObject<LiteralList> for f64 {
    const TYPE_ID: LiteralList = LiteralList::Number;
}
impl AllocObject<TypeList> for f64 {
    const TYPE_ID: TypeList = TypeList::Number;
}
impl AllocObject<TypeList> for bool {
    const TYPE_ID: TypeList = TypeList::Boolean;
}

impl AllocObject<LiteralList> for String {
    const TYPE_ID: LiteralList = LiteralList::String;
}

impl AllocHeader for LiteralHeader {
    type TypeId = LiteralList;

    fn new<O: AllocObject<Self::TypeId>>(
        size: ArraySize,
        size_class: SizeClass,
        mark: Mark,
    ) -> Self {
        Self {
            size,
            size_class,
            mark,
            type_id: O::TYPE_ID,
        }
    }

    fn new_array(size: ArraySize, size_class: SizeClass, mark: Mark) -> Self {
        Self {
            size,
            size_class,
            mark,
            type_id: LiteralList::Invalid,
        }
    }

    fn type_id(&self) -> Self::TypeId {
        self.type_id
    }
}

impl MutatorScope for ROData<'_> {}

impl<'mem> ROData<'mem> {
    /// Parse this `token` into a literal or return `None`.
    ///
    /// This function allocates space in [`Self::heap`] for a [`TaggedCellPtr`] pointing to the
    /// parsed token if successful, or else nothing is allocated and [`None`] is returned.
    pub fn alloc(&'mem self, token: String) -> Option<TaggedCellPtr> {
        let token = if token.chars().nth(0) == Some('\"') {
            let rest = &token[1..];
            let j = rest.find('"')?;
            Some(rest[..j].to_string())
        } else if token.chars().nth(0).is_some_and(char::is_numeric) {
            Some(
                token
                    .chars()
                    .take_while(|d| d.is_numeric() || *d == '.')
                    .collect::<String>(),
            )
        } else {
            None
        }?;

        // Try parsing as a number first, otherwise treat as string
        let literal = if let Ok(n) = token.parse::<f64>() {
            Literal::Number(n)
        } else {
            Literal::String(token)
        };

        match literal {
            Literal::Number(n) => {
                let tagged = TaggedPtr::number(n);
                Some(TaggedCellPtr::new(tagged))
            }
            Literal::String(s) => {
                let raw = self.heap.alloc(Text::from(s)).ok()?;
                let tagged = TaggedPtr::object(raw.to_void());
                Some(TaggedCellPtr::new(tagged))
            }
        }
    }

    pub fn new_with(heap: &'mem StickyImmixHeap<LiteralHeader>) -> ROData<'mem> {
        Self { heap }
    }
}

type TapeMap = HashMap<ArraySize, TaggedCellPtr, BuildFxHasher>;

#[derive(Clone)]
pub struct RuntimeTape {
    values_right: TapeMap,
    values_left: TapeMap,
    pub cursor: i32,
}

impl RuntimeTape {
    fn container(&self, idx: i32) -> (&TapeMap, u32) {
        if idx < 0 {
            (&self.values_left, (idx.abs() - 1).cast_unsigned())
        } else {
            (&self.values_right, (idx).cast_unsigned())
        }
    }

    fn container_mut(&mut self, idx: i32) -> (&mut TapeMap, u32) {
        if idx < 0 {
            (&mut self.values_left, (idx.abs() - 1).cast_unsigned())
        } else {
            (&mut self.values_right, (idx).cast_unsigned())
        }
    }

    pub fn get_values(&self, centered: Option<i32>) -> Vec<(ArraySize, &TaggedCellPtr)> {
        let base = self.values_right.iter().map(|x| (*x.0, x.1));

        if let Some(idx) = centered {
            let (container, index) = self.container(idx);

            let mut scoped = Vec::new();
            for i in 0..16 {
                if let Some(x) = container.get(&(index + i)) {
                    scoped.push((index + i, x))
                }
            }
            scoped.into_iter().chain(base).collect()
        } else {
            base.collect()
        }
    }

    pub fn new() -> Self {
        Self {
            values_left: TapeMap::with_hasher(BuildFxHasher {}),
            values_right: TapeMap::with_hasher(BuildFxHasher {}),
            cursor: 0,
        }
    }

    pub fn get_pointer(&self, idx: i32) -> Option<&TaggedCellPtr> {
        let (container, idx) = self.container(idx);
        container.get(&idx)
    }

    pub fn get(&self, idx: i32, guard: &MutatorView) -> Result<super::Value, RuntimeError> {
        let (container, i) = self.container(idx);

        Ok(guard
            .get_tape()
            .get_value(i, guard, container)
            .unwrap_or(super::Value::Number(0f64)))
    }

    pub fn get_current(&self, guard: &MutatorView) -> Result<super::Value, RuntimeError> {
        self.get(self.cursor, guard)
    }

    pub fn set_cursor(&mut self, idx: i32) {
        self.cursor = idx;
    }

    pub fn move_cursor(&mut self, by: i32) {
        self.cursor += by;
    }

    pub fn set_value(
        &mut self,
        guard: &MutatorView,
        val: super::Value,
    ) -> Result<(), RuntimeError> {
        let (container, idx) = self.container_mut(self.cursor);
        guard.get_tape().upsert_value(idx, val, container)
    }

    pub fn clone_into(&self, idx: i32, values_within: ArraySize) -> Self {
        let left: TapeMap = HashMap::with_hasher(BuildFxHasher {});
        let mut right: TapeMap = HashMap::with_hasher(BuildFxHasher {});

        for i in 0..values_within {
            if let Some(x) = self.get_pointer(idx + i.cast_signed()) {
                right.insert(i, x.clone());
            }
        }

        Self {
            values_left: left,
            values_right: right,
            cursor: 0,
        }
    }
}

impl Default for RuntimeTape {
    fn default() -> Self {
        Self::new()
    }
}

struct Tape {
    heap: StickyImmixHeap<TypeHeader>,
}

impl Tape {
    #[must_use]
    fn new() -> Self {
        let heap: StickyImmixHeap<TypeHeader> = StickyImmixHeap::new();
        Self { heap }
    }

    fn alloc<T>(&self, object: T) -> Result<RawPtr<T>, RuntimeError>
    where
        T: AllocObject<TypeList>,
    {
        Ok(self.heap.alloc(object)?)
    }

    fn alloc_tagged<T>(&self, object: T) -> Result<TaggedPtr, RuntimeError>
    where
        FatPtr: From<RawPtr<T>>,
        T: AllocObject<TypeList>,
    {
        Ok(TaggedPtr::from(FatPtr::from(self.alloc(object)?)))
    }

    pub fn get_value(
        &self,
        idx: ArraySize,
        guard: &dyn MutatorScope,
        map: &HashMap<ArraySize, TaggedCellPtr, BuildFxHasher>,
    ) -> Option<super::Value> {
        let ptr = map.get(&idx)?;
        let value = ptr.get(guard).get_value().into();
        Some(value)
    }

    pub fn upsert_value(
        &self,
        idx: ArraySize,
        value: super::Value,
        map: &mut HashMap<ArraySize, TaggedCellPtr, BuildFxHasher>,
    ) -> Result<(), RuntimeError> {
        let ptr: TaggedPtr = match value {
            super::Value::String(s) => {
                let text = Text::from(s);
                self.alloc_tagged(text)?
            }
            super::Value::Err(e) => {
                let text = Text::from(e.to_string());
                self.alloc_tagged(text)?
            }
            super::Value::FiveMinuteCodingAdventure(f) => self.alloc_tagged(f)?,
            super::Value::Number(n) => self.alloc_tagged(n)?,
            super::Value::Boolean(b) => self.alloc_tagged(b)?,
            super::Value::Nil => self.alloc_tagged(Nil {})?,
        };

        if let Some(val) = map.get_mut(&idx) {
            *val = TaggedCellPtr::new(ptr);
        } else {
            map.insert(idx, TaggedCellPtr::new(ptr));
        }

        Ok(())
    }
}

impl From<AllocError> for RuntimeError {
    fn from(value: AllocError) -> Self {
        RuntimeError::AllocError(value.to_string())
    }
}

pub struct MutatorView<'memory> {
    tape: Tape,
    literals: ROData<'memory>,
}

impl MutatorScope for MutatorView<'_> {}

impl<'memory> MutatorView<'memory> {
    pub fn alloc<T>(&self, object: T) -> Result<ScopedPtr<'_, T>, RuntimeError>
    where
        T: AllocObject<TypeList>,
    {
        Ok(ScopedPtr::new(
            self,
            self.tape.alloc(object)?.scoped_ref(self),
        ))
    }

    pub fn alloc_tagged<T>(&self, object: T) -> Result<TaggedScopedPtr<'_>, RuntimeError>
    where
        T: AllocObject<TypeList>,
        FatPtr: From<RawPtr<T>>,
    {
        let raw = self.tape.alloc_tagged(object)?;
        Ok(TaggedScopedPtr::new(self, raw))
    }

    fn get_tape(&'memory self) -> &'memory Tape {
        &self.tape
    }

    pub fn get_data(&self) -> &'memory ROData<'_> {
        &self.literals
    }

    pub fn new_with(literals: &'memory StickyImmixHeap<LiteralHeader>) -> Self {
        let tape = Tape::new();
        let rodata = ROData::new_with(literals);

        Self {
            tape,
            literals: rodata,
        }
    }
}
