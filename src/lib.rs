#![warn(clippy::pedantic)]

pub mod asm;
pub mod builder;

pub use builder::AsmBuilder;

pub type ArrayLen = u32;
pub type ArrayIndex = ArrayLen;
pub type Int = i64;
