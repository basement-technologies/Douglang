use super::*;

pub trait AllocObject<T: AllocTypeId> {
	const TYPE_ID: T;
}

pub type ArraySize = u32;

pub trait AllocRaw {
	/// An implementation of an object header type
	type Header: AllocHeader;

	/// Allocaet a single object of type `T`
	fn alloc<T>(&self, object: T) -> Result<RawPtr<T>, AllocError>
	where
		T: AllocObject<<Self::Header as AllocHeader>::TypeId>;

	/// Allocating an array allows the client to put anything in the resulting data block but the
	/// type of the memory block will simply be `Array`. No other type information will be stored in
	/// the object header.
	/// THe caller is responsible for the content of the array.
	fn alloc_array(&self, size_bytes: ArraySize) -> Result<RawPtr<u8>, AllocError>;

	fn get_header(object: NonNull<()>) -> NonNull<Self::Header>;
	fn get_object(header: NonNull<Self::Header>) -> NonNull<()>;
}

pub trait AllocHeader: Sized {
	/// Associated type that identifies the allocated object type
	type TypeId: AllocTypeId;

	/// Create a new header for object type O
	fn new<O: AllocObject<Self::TypeId>>(
		size: ArraySize,
		size_class: SizeClass,
		mark: Mark,
	) -> Self;

	/// Create a new header for an array type
	fn new_array(size: ArraySize, size_class: SizeClass, mark: Mark) -> Self;

	/*
	/// Set the Mark value to "marked"
	fn mark(&mut self);

	/// Get the current Mark value
	fn is_marked(&self) -> bool;

	/// Get the size class of the object
	fn size_class(&self) -> SizeClass;

	/// Get the size of the object in bytes
	fn size(&self) -> u32;
	*/

	/// Get the type of the object
	fn type_id(&self) -> Self::TypeId;
}

pub trait MutatorScope {}

pub trait Mutator<'a>: Sized {
	type Input;
	type Output;
	type Scope: MutatorScope;

	fn run(
		&mut self,
		mem: &'a Self::Scope,
		input: Self::Input,
	) -> Result<Self::Output, RuntimeError>;
}
