#![allow(clippy::module_name_repetitions)]

use crate::{
    builder::{BuildInstruction, Reg},
    Char,
};

pub trait BuilderExt: BuildInstruction {
    fn char(&mut self, ch: Char, to: Reg) -> &mut Self {
        self.integer(i64::from(ch), to)
    }
}

impl<T: BuildInstruction> BuilderExt for T {}