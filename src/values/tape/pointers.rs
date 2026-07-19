use std::fmt::Debug;

use crate::values::tape::constants::{TAG_NUMBER, TAG_OBJECT};

use super::*;

pub struct RawPtr<T: Sized> {
    ptr: NonNull<T>,
}

impl<T> Clone for RawPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for RawPtr<T> {}
impl<T> Deref for RawPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> RawPtr<T> {
    pub fn new(ptr: *const T) -> Self {
        Self {
            ptr: NonNull::new(ptr.cast_mut()).expect("The ptr is null"),
        }
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    pub fn to_void(&self) -> RawPtr<()> {
        RawPtr {
            ptr: NonNull::new(self.as_ptr().cast::<()>()).expect("The ptr is null"),
        }
    }

    pub fn as_word(&self) -> usize {
        self.as_ptr() as usize
    }
}

#[derive(Clone)]
pub struct CellPtr<T: Sized> {
    inner: Cell<RawPtr<T>>,
}

impl<T: Sized> CellPtr<T> {
    pub fn get<'guard>(&self, guard: &'guard dyn MutatorScope) -> ScopedPtr<'guard, T> {
        ScopedPtr::new(guard, self.inner.get().scoped_ref(guard))
    }

    pub fn new_with<'guard>(ptr: &'guard ScopedPtr<'guard, T>) -> Self {
        Self {
            inner: Cell::new(RawPtr::new(ptr.value)),
        }
    }

    pub fn new(mem: &dyn MutatorScope, ptr: T) -> Self {
        Self::new_with(&ScopedPtr::new(mem, &ptr))
    }
}

pub trait ScopedRef<T> {
    fn scoped_ref<'scope>(&self, guard: &'scope dyn MutatorScope) -> &'scope T;
}

impl<T> ScopedRef<T> for RawPtr<T> {
    fn scoped_ref<'scope>(&self, _guard: &'scope dyn MutatorScope) -> &'scope T {
        unsafe { &*self.as_ptr() }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ScopedPtr<'guard, T: Sized> {
    value: &'guard T,
}

impl<'a, T> Deref for ScopedPtr<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'guard, T: Sized> ScopedPtr<'guard, T> {
    pub fn new(_guard: &'guard dyn MutatorScope, ptr: &'guard T) -> Self {
        Self { value: ptr }
    }
}

pub trait Tagged<T> {
    fn tag(self, tag: usize) -> NonNull<T>;
    fn untag(from: NonNull<T>) -> RawPtr<T>;
}

impl<T> Tagged<T> for RawPtr<T> {
    fn tag(self, tag: usize) -> NonNull<T> {
        unsafe { NonNull::new_unchecked((self.as_word() | tag) as *mut T) }
    }
    fn untag(from: NonNull<T>) -> RawPtr<T> {
        RawPtr::new((from.as_ptr() as usize & constants::PTR_MASK) as *const T)
    }
}

#[derive(Clone)]
pub struct TaggedCellPtr {
    inner: Cell<TaggedPtr>,
}

impl PartialEq for TaggedCellPtr {
    fn eq(&self, other: &Self) -> bool {
        self.get(&TestGuard {}).get_value() == other.get(&TestGuard {}).get_value()
    }
}

impl PartialOrd for TaggedCellPtr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let v: crate::values::Value = self.get(&TestGuard {}).get_value().into();
        let v2: crate::values::Value = other.get(&TestGuard {}).get_value().into();

        v.partial_cmp(&v2)
    }
}

pub struct TestGuard {}
impl MutatorScope for TestGuard {}

impl Debug for TaggedCellPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = self.inner.get().as_fat_ptr().as_value(&(TestGuard {}));
        let x: crate::values::Value = x.into();
        write!(f, "{x}")
    }
}

#[derive(Clone)]
pub struct TaggedScopedPtr<'guard> {
    ptr: TaggedPtr,
    value: Value<'guard>,
}

impl<'guard> TaggedScopedPtr<'guard> {
    pub fn new(guard: &'guard dyn MutatorScope, ptr: TaggedPtr) -> Self {
        Self {
            ptr,
            value: ptr.as_fat_ptr().as_value(guard),
        }
    }

    pub fn get_value(&self) -> Value<'guard> {
        self.value.clone()
    }

    pub fn get_ptr(&self) -> TaggedPtr {
        self.ptr
    }
}

impl TaggedCellPtr {
    pub fn get<'guard>(&self, guard: &'guard dyn MutatorScope) -> TaggedScopedPtr<'guard> {
        TaggedScopedPtr::new(guard, self.inner.get())
    }

    pub fn new(raw: TaggedPtr) -> TaggedCellPtr {
        Self {
            inner: Cell::new(raw),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Value<'guard> {
    String(ScopedPtr<'guard, Text>),
    Function(ScopedPtr<'guard, Function>),
    Array(ScopedPtr<'guard, Array>),
    Number(f64),
    Integer(isize),
    Nil,
}

#[derive(Clone, Copy)]
pub enum FatPtr {
    String(RawPtr<Text>),
    Function(RawPtr<Function>),
    Array(RawPtr<Array>),
    Number(f64),
    Integer(isize),
    Nil,
}

#[derive(Copy, Clone)]
pub union TaggedPtr {
    tag: usize,
    number: f64,
    integer: isize,
    object: NonNull<()>,
}

impl FatPtr {
    pub fn as_value<'guard>(&self, guard: &'guard dyn MutatorScope) -> Value<'guard> {
        match self {
            FatPtr::Array(raw_ptr) => {
                Value::Array(ScopedPtr::new(guard, raw_ptr.scoped_ref(guard)))
            }
            FatPtr::String(raw_ptr) => {
                Value::String(ScopedPtr::new(guard, raw_ptr.scoped_ref(guard)))
            }
            FatPtr::Function(raw_ptr) => {
                Value::Function(ScopedPtr::new(guard, raw_ptr.scoped_ref(guard)))
            }
            FatPtr::Integer(num) => Value::Integer(*num),
            FatPtr::Number(num) => Value::Number(*num),
            FatPtr::Nil => Value::Nil,
        }
    }
}

impl From<RawPtr<Function>> for FatPtr {
    fn from(value: RawPtr<Function>) -> Self {
        FatPtr::Function(value)
    }
}
impl From<RawPtr<Text>> for FatPtr {
    fn from(value: RawPtr<Text>) -> Self {
        FatPtr::String(value)
    }
}
impl From<RawPtr<f64>> for FatPtr {
    fn from(value: RawPtr<f64>) -> Self {
        FatPtr::Number(*value)
    }
}
impl From<RawPtr<bool>> for FatPtr {
    fn from(value: RawPtr<bool>) -> Self {
        FatPtr::Number(i32::from(*value) as f64)
    }
}
impl From<RawPtr<Nil>> for FatPtr {
    fn from(_value: RawPtr<Nil>) -> Self {
        FatPtr::Nil
    }
}

impl TaggedPtr {
    pub fn nil() -> TaggedPtr {
        TaggedPtr { tag: 0 }
    }
    pub fn number(value: f64) -> TaggedPtr {
        let as_f32 = value as f32;
        let bits = (as_f32.to_bits() as u64) << 2 | constants::TAG_NUMBER as u64;
        TaggedPtr {
            number: f64::from_bits(bits),
        }
    }
    pub fn integer(value: isize) -> TaggedPtr {
        TaggedPtr {
            integer: (((value as usize) << 2) | constants::TAG_NUMBER) as isize,
        }
    }
    pub fn object(ptr: RawPtr<()>) -> TaggedPtr {
        TaggedPtr {
            object: ptr.tag(constants::TAG_OBJECT),
        }
    }

    fn as_fat_ptr(&self) -> FatPtr {
        unsafe {
            if self.tag == 0 {
                FatPtr::Nil
            } else {
                match self.tag & 0b11 {
                    TAG_NUMBER => {
                        let bits = (self.number.to_bits() >> 2) as u32;
                        FatPtr::Number(f32::from_bits(bits) as f64)
                    }
                    TAG_OBJECT => {
                        let object_ptr = RawPtr::untag(self.object).ptr;
                        let header_ptr: NonNull<TypeHeader> =
                            StickyImmixHeap::get_header(object_ptr);

                        header_ptr.as_ref().get_object_fatptr()
                    }

                    _ => panic!("Invalid TaggedPtr type tag!"),
                }
            }
        }
    }
}

impl From<FatPtr> for TaggedPtr {
    fn from(ptr: FatPtr) -> TaggedPtr {
        match ptr {
            FatPtr::Number(num) => TaggedPtr::number(num),
            FatPtr::Integer(num) => TaggedPtr::integer(num),
            FatPtr::String(raw) => TaggedPtr::object(raw.to_void()),
            FatPtr::Function(raw) => TaggedPtr::object(raw.to_void()),
            FatPtr::Array(raw) => TaggedPtr::object(raw.to_void()),
            FatPtr::Nil => TaggedPtr::nil(),
        }
    }
}

impl From<TaggedPtr> for FatPtr {
    fn from(ptr: TaggedPtr) -> Self {
        ptr.as_fat_ptr()
    }
}
