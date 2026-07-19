mod hashers;
pub mod literal;
mod operator;
pub mod tape;
pub mod value;

pub use hashers::{BuildFxHasher, FxHasher, hash_fiveminutecodingadventure};
pub use literal::Literal;
pub use operator::Operator;
pub use value::Value;
