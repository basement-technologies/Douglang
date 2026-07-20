use std::{
	hash::{BuildHasher, Hasher},
	ops::BitXor,
};

use crate::{
	parser::ast::{DougChain, Expr, Reference, Stmt},
	values::{Operator, value::FiveMinuteCodingAdventure},
};

#[derive(Clone)]
pub struct FxHasher {
	hash: usize,
	pub indexes: Vec<i32>,
}

impl FxHasher {
	pub fn new() -> Self {
		Self {
			hash: 0,
			indexes: Vec::new(),
		}
	}
}

const K: usize = 0x517cc1b727220a95;

impl Hasher for FxHasher {
	fn write(&mut self, bytes: &[u8]) {
		let i = bytes
			.iter()
			.take(8)
			.fold(0u64, |acc, &b| (acc << 8) | b as u64) as usize;
		self.hash = self.hash.rotate_left(5).bitxor(i).wrapping_mul(K);
	}

	fn finish(&self) -> u64 {
		self.hash as u64
	}
}

#[derive(Clone, Copy)]
pub struct BuildFxHasher {}
impl BuildHasher for BuildFxHasher {
	type Hasher = FxHasher;

	fn build_hasher(&self) -> Self::Hasher {
		FxHasher::new()
	}
}

pub trait HashNode {
	fn hash_node(&self, hasher: &mut FxHasher);
}

impl HashNode for str {
	fn hash_node(&self, hasher: &mut FxHasher) {
		hasher.write(&(self.len() as u64).to_le_bytes());
		for chunk in self.as_bytes().chunks(8) {
			hasher.write(chunk);
		}
	}
}

impl HashNode for String {
	fn hash_node(&self, hasher: &mut FxHasher) {
		self.as_str().hash_node(hasher);
	}
}

impl HashNode for bool {
	fn hash_node(&self, hasher: &mut FxHasher) {
		hasher.write_u8(*self as u8)
	}
}

impl<T: HashNode> HashNode for Option<T> {
	fn hash_node(&self, hasher: &mut FxHasher) {
		match self {
			Some(v) => {
				hasher.write(&[1]);
				v.hash_node(hasher);
			}
			None => hasher.write(&[0]),
		}
	}
}

impl<T: HashNode> HashNode for [T] {
	fn hash_node(&self, hasher: &mut FxHasher) {
		hasher.write(&(self.len() as u64).to_le_bytes());
		for item in self {
			item.hash_node(hasher);
		}
	}
}

impl<T: HashNode> HashNode for Box<T> {
	fn hash_node(&self, hasher: &mut FxHasher) {
		(**self).hash_node(hasher);
	}
}

impl<T: HashNode> HashNode for Box<[T]> {
	fn hash_node(&self, hasher: &mut FxHasher) {
		(**self).hash_node(hasher);
	}
}

impl HashNode for Reference {
	fn hash_node(&self, hasher: &mut FxHasher) {
		match self {
			Self::Doug(DougChain { count }) => {
				hasher.write_u8(1);
				hasher.write(&count.to_le_bytes());
			}
			Self::Variable(name) => {
				name.hash_node(hasher);
			}
		}
	}
}

impl HashNode for DougChain {
	fn hash_node(&self, hasher: &mut FxHasher) {
		hasher.write_usize(self.count);
	}
}

impl HashNode for Operator {
	fn hash_node(&self, hasher: &mut FxHasher) {
		hasher.write(&[*self as u8]);
	}
}

impl HashNode for Expr {
	fn hash_node(&self, hasher: &mut FxHasher) {
		match self {
			Expr::Literal(_) => {
				hasher.write_i32(1);
			}
			Expr::Variable(v) => {
				v.hash_node(hasher);
			}
			Expr::Rigged { func, args } => {
				func.hash_node(hasher);
				args.clone().into_boxed_slice().hash_node(hasher);
			}
			Expr::Condition {
				left,
				operator,
				right,
			} => {
				left.hash_node(hasher);
				operator.hash_node(hasher);
				right.hash_node(hasher);
			}
			Expr::FiveMinuteCodingAdventureCall { name } => {
				name.hash_node(hasher);
			}
			Expr::DougSequence { chains } => {
				chains.clone().hash_node(hasher);
			}
			Expr::MainTapeDougSequence { chains } => {
				chains.clone().hash_node(hasher);
			}
		}
	}
}

impl HashNode for Stmt {
	fn hash_node(&self, hasher: &mut FxHasher) {
		match self {
			Stmt::Doug { chains, reset } => {
				hasher.write(&[0]);
				chains.hash_node(hasher);
				reset.hash_node(hasher);
			}
			Stmt::Tts {
				msg,
				use_index,
				overlap,
			} => {
				hasher.write(&[1]);
				msg.hash_node(hasher);
				use_index.hash_node(hasher);
				overlap.hash_node(hasher);
			}
			Stmt::Call { name, use_index } => {
				hasher.write(&[2]);
				name.hash_node(hasher);
				use_index.hash_node(hasher);
			}
			Stmt::Guod { value, use_index } => {
				hasher.write(&[3]);
				value.hash_node(hasher);
				use_index.hash_node(hasher);
			}
			Stmt::EndStream => {
				hasher.write_u8(1);
			}
			Stmt::Set { value, oper } => {
				hasher.write(&[4]);
				value.hash_node(hasher);
				oper.hash_node(hasher);
			}
			Stmt::Expr(expr) => {
				hasher.write(&[5]);
				expr.hash_node(hasher);
			}
			Stmt::Loop { body } => {
				hasher.write(&[6]);
				body.hash_node(hasher);
			}
			Stmt::Prediction {
				believe_body,
				doubt_body,
				condition,
			} => {
				hasher.write(&[7]);
				believe_body.hash_node(hasher);
				doubt_body.hash_node(hasher);
				condition.hash_node(hasher);
			}
			Stmt::FiveMinuteCodingAdventure { name, body } => {
				hasher.write(&[8]);
				name.hash_node(hasher);
				body.hash_node(hasher);
			}
		}
	}
}

impl HashNode for FiveMinuteCodingAdventure {
	fn hash_node(&self, hasher: &mut FxHasher) {
		self.get_nodes().hash_node(hasher);
	}
}

/// Hashes a Function down to a negative i32 that's decently far from
/// zero but capped to a modest magnitude, so consecutive/similar
/// functions land in well-spread-out slots without spanning the
/// full i32 range.
pub fn hash_fiveminutecodingadventure(func: &FiveMinuteCodingAdventure, hasher: &mut FxHasher) -> i32 {
	func.hash_node(hasher);
	let raw = hasher.finish();

	// Tune these values!!.
	const MIN_MAG: u64 = 1_000;
	const MAX_MAG: u64 = 2_000_000;
	let span = MAX_MAG - MIN_MAG;
	let mag = MIN_MAG + (raw % span);

	let attempted = -(mag as i32);
	for i in &hasher.indexes {
		if i.abs_diff(attempted) < 16 {
			return hash_fiveminutecodingadventure(func, hasher);
		}
	}
	hasher.indexes.push(attempted);
	attempted
}
