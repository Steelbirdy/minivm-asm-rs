#![warn(clippy::pedantic)]

pub mod asm;
pub mod builder;
mod ext;

pub use builder::{AsmBuilder, BuildInstruction};
pub use ext::BuilderExt;

pub type ArrayLen = u32;
pub type ArrayIndex = ArrayLen;
pub type Int = i64;
pub type Char = u8;
