#![allow(clippy::module_name_repetitions)]

use crate::{
    builder::{BuildInstruction, Reg},
    Char,
};

pub trait BuilderExt<'id>: BuildInstruction<'id> {
    fn char(&mut self, ch: Char, to: Reg<'id>) -> &mut Self {
        self.integer(i64::from(ch), to)
    }
}
