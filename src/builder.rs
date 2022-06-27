#![allow(clippy::module_name_repetitions)]

use crate::{asm, Int};
use generativity::Id;
use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

pub type Lbl<'id> = &'id str;
pub type Reg<'id> = u8;

pub struct AsmBuilder<'id> {
    asm: asm::Asm,
    main: LabelBuilder<'id>,
    built_main: bool,
    unfinished: Option<LabelBuilder<'id>>,
    __id: Id<'id>,
}

impl<'id> AsmBuilder<'id> {
    #[must_use]
    pub fn new(id: Id<'id>) -> AsmBuilder<'id> {
        Self {
            asm: asm::Asm::new(),
            main: LabelBuilder::new("main", id),
            built_main: false,
            unfinished: None,
            __id: id,
        }
    }

    fn build_main_check(&mut self) {
        assert!(!self.built_main, "cannot build `main` more than once");
        self.built_main = true;
    }

    fn take_unfinished(&mut self) {
        if let Some(prev_builder) = self.unfinished.take() {
            self.asm.push_label(prev_builder.finish());
        }
    }

    /// Panics if `main` has already been built.
    #[must_use]
    pub fn build_main<'a>(&'a mut self) -> LabelBuilderGuard<'a, 'id> {
        self.build_main_check();
        LabelBuilderGuard::new(&mut self.main, self.__id)
    }

    /// Panics if `main` has already been built.
    pub fn main<F>(&mut self, f: F) -> &mut Self
    where
        F: for<'a> FnOnce(&'a mut LabelBuilder<'id>) -> &'a mut LabelBuilder<'id>,
    {
        self.build_main_check();
        f(&mut self.main);
        self
    }

    #[must_use]
    pub fn build_label<'a>(&'a mut self, name: &str) -> LabelBuilderGuard<'a, 'id> {
        self.take_unfinished();
        let builder = LabelBuilder::new(name, self.__id);
        let builder = self.unfinished.insert(builder);
        LabelBuilderGuard::new(builder, self.__id)
    }

    pub fn label<F>(&mut self, name: &str, f: F) -> &mut Self
    where
        F: for<'a> FnOnce(&'a mut LabelBuilder<'id>) -> &'a mut LabelBuilder<'id>,
    {
        self.take_unfinished();
        let mut builder = LabelBuilder::new(name, self.__id);
        f(&mut builder);
        self.asm.push_label(builder.finish());
        self
    }

    #[must_use]
    pub fn finish(mut self) -> asm::Asm {
        self.take_unfinished();
        let AsmBuilder { mut asm, main, .. } = self;
        *asm.main() = main.finish();
        asm
    }
}

pub struct LabelBuilder<'id> {
    lbl: asm::Label,
    unfinished: Option<SubLabelBuilder<'id>>,
    __id: Id<'id>,
}

impl<'id> LabelBuilder<'id> {
    #[must_use]
    pub fn new(name: &str, id: Id<'id>) -> LabelBuilder<'id> {
        Self {
            lbl: asm::Label::new(name),
            unfinished: None,
            __id: id,
        }
    }

    fn take_unfinished(&mut self) {
        if let Some(prev_builder) = self.unfinished.take() {
            self.lbl.push_sub_label(prev_builder.finish());
        }
    }

    #[must_use]
    pub fn build_sub_label<'a>(&'a mut self, name: &str) -> SubLabelBuilderGuard<'a, 'id> {
        self.take_unfinished();
        let builder = SubLabelBuilder::new(self.lbl.name(), name, self.__id);
        let builder = self.unfinished.insert(builder);
        BuilderGuard::new(builder, self.__id)
    }

    pub fn sub_label<F>(&mut self, name: &str, f: F) -> &mut Self
    where
        F: for<'a> FnOnce(&'a mut SubLabelBuilder<'id>) -> &'a mut SubLabelBuilder<'id>,
    {
        self.take_unfinished();
        let mut builder = SubLabelBuilder::new(self.lbl.name(), name, self.__id);
        f(&mut builder);
        self.lbl.push_sub_label(builder.finish());
        self
    }

    #[must_use]
    pub fn finish(mut self) -> asm::Label {
        self.take_unfinished();
        self.lbl
    }

    fn write_line<'a>(&mut self, line: impl Into<Cow<'a, str>>) {
        self.lbl.push_line(line);
    }
}

pub struct SubLabelBuilder<'id> {
    lbl: asm::SubLabel,
    __id: Id<'id>,
}

impl<'id> SubLabelBuilder<'id> {
    fn new(label: &str, name: &str, id: Id<'id>) -> SubLabelBuilder<'id> {
        Self {
            lbl: asm::SubLabel::new(label, name),
            __id: id,
        }
    }

    fn finish(self) -> asm::SubLabel {
        self.lbl
    }

    fn write_line<'a>(&mut self, line: impl Into<Cow<'a, str>>) {
        self.lbl.push_line(line);
    }
}

pub struct BuilderGuard<'a, 'id, T> {
    inner: &'a mut T,
    finished: drop_bomb::DropBomb,
    __id: Id<'id>,
}

pub type LabelBuilderGuard<'a, 'id> = BuilderGuard<'a, 'id, LabelBuilder<'id>>;
pub type SubLabelBuilderGuard<'a, 'id> = BuilderGuard<'a, 'id, SubLabelBuilder<'id>>;

impl<'a, 'id, T> BuilderGuard<'a, 'id, T> {
    fn new(inner: &'a mut T, id: Id<'id>) -> BuilderGuard<'a, 'id, T> {
        let bomb = drop_bomb::DropBomb::new("builder must be marked as finished using `.finish()`");
        Self {
            inner,
            finished: bomb,
            __id: id,
        }
    }

    pub fn finish(mut self) {
        self.finished.defuse();
    }
}

impl<'a, 'id, T> Deref for BuilderGuard<'a, 'id, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, 'id, T> DerefMut for BuilderGuard<'a, 'id, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

pub trait BuildInstruction<'a> {
    /// Return to caller, cleanup GC.
    fn exit(&mut self) -> &mut Self;

    /// Move contents of `rY` into `rX`.
    fn register_move(&mut self, from: Reg<'a>, to: Reg<'a>) -> &mut Self;

    /// Jump to `label.a`.
    fn label_jump(&mut self, label: Lbl<'a>) -> &mut Self;

    /// Jump to `label.a`.
    /// Argument in `rA` is moved to `r1`, `rB` to `r2`, `rC` to `r3`, and so on.
    /// Once the function is done, all registers are restored. The return value is put into `rX`.
    fn label_call(&mut self, label: Lbl<'a>, args: &[Reg<'a>], to: Reg<'a>) -> &mut Self;

    /// Store the address of `label.a` in `rX`.
    fn label_address(&mut self, label: Lbl<'a>, to: Reg<'a>) -> &mut Self;

    /// Jump to the address stored in `rX`. Usually this is obtained from [`label_address`](BuildInstruction::label_address).
    fn dynamic_jump(&mut self, reg: Reg<'a>) -> &mut Self;

    /// Jump to the address stored in `rX`. Usually this is obtained from [`label_address`](BuildInstruction::label_address).
    /// Argument in `rA` is moved to `r1`, `rB` to `r2`, `rC` to `r3`, and so on.
    /// Once the function is done, all registers are restored. The return value is put into `rX`.
    fn dynamic_call(&mut self, reg: Reg<'a>, args: &[Reg<'a>], to: Reg<'a>) -> &mut Self;

    /// Store the value stored in `rY` in the `rX` from [`label_call`](BuildInstruction::label_call) or [`dynamic_call`](BuildInstruction::dynamic_call).
    fn return_(&mut self, reg: Reg<'a>) -> &mut Self;

    /// Store `N` in `rX`.
    fn integer(&mut self, value: Int, to: Reg<'a>) -> &mut Self;

    /// Store the result of the operation `-rY` into `rX`.
    fn neg(&mut self, from: Reg<'a>, to: Reg<'a>) -> &mut Self;

    /// Store the result of the operation `rY + rZ` into `rX`.
    fn add(&mut self, lhs: Reg<'a>, rhs: Reg<'a>, to: Reg<'a>) -> &mut Self;

    /// Store the result of the operation `rY - rZ` into `rX`.
    fn sub(&mut self, lhs: Reg<'a>, rhs: Reg<'a>, to: Reg<'a>) -> &mut Self;

    /// Store the result of the operation `rY * rZ` into `rX`.
    fn mul(&mut self, lhs: Reg<'a>, rhs: Reg<'a>, to: Reg<'a>) -> &mut Self;

    /// Store the result of the operation `rY / rZ` into `rX`.
    fn div(&mut self, lhs: Reg<'a>, rhs: Reg<'a>, to: Reg<'a>) -> &mut Self;

    /// Store the result of the operation `rY % rZ` into `rX`.
    fn mod_(&mut self, lhs: Reg<'a>, rhs: Reg<'a>, to: Reg<'a>) -> &mut Self;

    /// Jump to `label.a` if the contents of `rX` is zero, otherwise jump to `label.b`.
    fn branch_boolean(
        &mut self,
        reg: Reg<'a>,
        label_true: Lbl<'a>,
        label_false: Lbl<'a>,
    ) -> &mut Self;

    /// Jump to `label.t` if the contents of `rX` is equal to the contents of `rY`, otherwise jump to `label.f`.
    fn branch_equal(
        &mut self,
        reg1: Reg<'a>,
        reg2: Reg<'a>,
        label_true: Lbl<'a>,
        label_false: Lbl<'a>,
    ) -> &mut Self;

    /// Jump to `label.t` if the contents of `rX` is less than the contents of `rY`, otherwise jump to `label.f`.
    fn branch_less_than(
        &mut self,
        reg1: Reg<'a>,
        reg2: Reg<'a>,
        label_true: Lbl<'a>,
        label_false: Lbl<'a>,
    ) -> &mut Self;

    /// Store an array with the ascii data representing `"text-1"` into `rX`.
    fn string(&mut self, text: &str, to: Reg<'a>) -> &mut Self;

    /// Store an empty array of length `rY` into `rX`.
    fn array(&mut self, len: Reg<'a>, to: Reg<'a>) -> &mut Self;

    /// Store `rZ` into `rX` at index `rY`.
    fn set_array_index(&mut self, array: Reg<'a>, index: Reg<'a>, value: Reg<'a>) -> &mut Self;

    /// Store into `rX` the element at index `rZ` of `rY`.
    fn get_array_index(&mut self, array: Reg<'a>, index: Reg<'a>, to: Reg<'a>) -> &mut Self;

    /// Store into `rX` the length of the array in `rY`.
    fn array_length(&mut self, array: Reg<'a>, to: Reg<'a>) -> &mut Self;

    /// Store `0` into `rX` if the data in `rY` is an integer.
    /// Store `1` into `rX` if the data in `rY` is an array.
    fn object_type(&mut self, object: Reg<'a>, to: Reg<'a>) -> &mut Self;

    /// Print the character stored in `rX` to stdout.
    fn put_char(&mut self, ch: Reg<'a>) -> &mut Self;
}

macro_rules! impl_build_instruction {
    [$lt:lifetime: $($ty:ty),*] => {
        $(
        impl<$lt> BuildInstruction<$lt> for $ty {
            fn exit(&mut self) -> &mut Self {
                self.write_line("exit");
                self
            }

            fn register_move(&mut self, from: Reg<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("{to} <- reg r{from}"));
                self
            }

            fn label_jump(&mut self, label: Lbl<$lt>) -> &mut Self {
                self.write_line(format!("jump {label}"));
                self
            }

            fn label_call(&mut self, label: Lbl<$lt>, args: &[Reg<$lt>], to: Reg<$lt>) -> &mut Self {
                let mut buf = format!("r{to} <- call {label}");
                for arg in args {
                    buf.push(' ');
                    buf.push('r');
                    buf.push_str(&arg.to_string());
                }
                self.write_line(buf);
                self
            }

            fn label_address(&mut self, label: Lbl<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- addr {label}"));
                self
            }

            fn dynamic_jump(&mut self, reg: Reg<$lt>) -> &mut Self {
                self.write_line(format!("djump r{reg}"));
                self
            }

            fn dynamic_call(&mut self, reg: Reg<$lt>, args: &[Reg<$lt>], to: Reg<$lt>) -> &mut Self {
                let mut buf = format!("r{to} <- dcall r{reg}");
                for arg in args {
                    buf.push(' ');
                    buf.push('r');
                    buf.push_str(&arg.to_string());
                }
                self.write_line(buf);
                self
            }

            fn return_(&mut self, reg: Reg<$lt>) -> &mut Self {
                self.write_line(format!("ret r{reg}"));
                self
            }

            fn integer(&mut self, value: Int, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- int {value}"));
                self
            }

            fn neg(&mut self, from: Reg<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- neg r{from}"));
                self
            }

            fn add(&mut self, lhs: Reg<$lt>, rhs: Reg<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- add r{lhs} r{rhs}"));
                self
            }

            fn sub(&mut self, lhs: Reg<$lt>, rhs: Reg<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- sub r{lhs} r{rhs}"));
                self
            }

            fn mul(&mut self, lhs: Reg<$lt>, rhs: Reg<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- mul r{lhs} r{rhs}"));
                self
            }

            fn div(&mut self, lhs: Reg<$lt>, rhs: Reg<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- div r{lhs} r{rhs}"));
                self
            }

            fn mod_(&mut self, lhs: Reg<$lt>, rhs: Reg<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- mod r{lhs} r{rhs}"));
                self
            }

            fn branch_boolean(&mut self, reg: Reg<$lt>, label_true: Lbl<$lt>, label_false: Lbl<$lt>) -> &mut Self {
                self.write_line(format!("bb r{reg} {label_false} {label_true}"));
                self
            }

            fn branch_equal(&mut self, reg1: Reg<$lt>, reg2: Reg<$lt>, label_true: Lbl<$lt>, label_false: Lbl<$lt>) -> &mut Self {
                self.write_line(format!("beq r{reg1} r{reg2} {label_false} {label_true}"));
                self
            }

            fn branch_less_than(&mut self, reg1: Reg<$lt>, reg2: Reg<$lt>, label_true: Lbl<$lt>, label_false: Lbl<$lt>) -> &mut Self {
                self.write_line(format!("blt r{reg1} r{reg2} {label_false} {label_true}"));
                self
            }

            fn string(&mut self, text: &str, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- str :{text}"));
                self
            }

            fn array(&mut self, len: Reg<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- arr r{len}"));
                self
            }

            fn set_array_index(&mut self, array: Reg<$lt>, index: Reg<$lt>, value: Reg<$lt>) -> &mut Self {
                self.write_line(format!("set r{array} r{index} r{value}"));
                self
            }

            fn get_array_index(&mut self, array: Reg<$lt>, index: Reg<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- get r{array} r{index}"));
                self
            }

            fn array_length(&mut self, array: Reg<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- len r{array}"));
                self
            }

            fn object_type(&mut self, object: Reg<$lt>, to: Reg<$lt>) -> &mut Self {
                self.write_line(format!("r{to} <- type r{object}"));
                self
            }

            fn put_char(&mut self, ch: Reg<$lt>) -> &mut Self {
                self.write_line(format!("putchar r{ch}"));
                self
            }
        }
        )*
    };
}

impl_build_instruction!['id: LabelBuilder<'id>, SubLabelBuilder<'id>, LabelBuilderGuard<'_, 'id>, SubLabelBuilderGuard<'_, 'id>];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asm_builder_build() {
        generativity::make_guard!(guard);

        let mut builder = AsmBuilder::new(guard.into());

        builder.main(|main_builder| {
            main_builder
                .integer(35, 0)
                .label_call("fib", &[0], 0)
                .label_call("putn", &[0], 0)
                .integer(10, 0)
                .put_char(0)
                .exit()
        });

        builder.label("fib", |fib_builder| {
            fib_builder
                .integer(2, 0)
                .branch_less_than(1, 0, "fib.then", "fib.else")
                .sub_label("then", |fib_then_builder| fib_then_builder.return_(1))
                .sub_label("else", |fib_else_builder| {
                    fib_else_builder
                        .integer(1, 0)
                        .sub(1, 0, 1)
                        .sub(1, 0, 0)
                        .label_call("fib", &[1], 1)
                        .label_call("fib", &[0], 0)
                        .add(0, 1, 0)
                        .return_(0)
                })
        });

        let mut putn_label = builder.build_label("putn");
        putn_label
            .branch_boolean(1, "putn.digit", "putn.ret")
            .sub_label("digit", |putn_digit_builder| {
                putn_digit_builder
                    .integer(10, 0)
                    .div(1, 0, 0)
                    .label_call("putn", &[0], 0)
                    .integer(10, 0)
                    .mod_(1, 0, 1)
                    .integer(48, 0)
                    .add(1, 0, 1)
                    .put_char(1)
            })
            .sub_label("ret", |putn_ret_builder| {
                putn_ret_builder.integer(0, 0).return_(0)
            });
        putn_label.finish();

        assert_eq!(
            builder.finish().finish(),
            r"@__entry
    r0 <- call main
    exit

func fib
    r0 <- int 2
    blt r1 r0 fib.else fib.then
@fib.then
    ret r1
@fib.else
    r0 <- int 1
    r1 <- sub r1 r0
    r0 <- sub r1 r0
    r1 <- call fib r1
    r0 <- call fib r0
    r0 <- add r0 r1
    ret r0
end

func putn
    bb r1 putn.ret putn.digit
@putn.digit
    r0 <- int 10
    r0 <- div r1 r0
    r0 <- call putn r0
    r0 <- int 10
    r1 <- mod r1 r0
    r0 <- int 48
    r1 <- add r1 r0
    putchar r1
@putn.ret
    r0 <- int 0
    ret r0
end

func main
    r0 <- int 35
    r0 <- call fib r0
    r0 <- call putn r0
    r0 <- int 10
    putchar r0
    exit
end",
        );
    }

    #[test]
    fn test_label_builder_build() {
        generativity::make_guard!(guard);

        let mut builder = LabelBuilder::new("fib", guard.into());
        builder
            .integer(2, 0)
            .branch_less_than(1, 0, "fib.then", "fib.else");

        let mut then_builder = builder.build_sub_label("then");
        then_builder.return_(1);
        then_builder.finish();

        builder.sub_label("else", |else_builder| {
            else_builder
                .integer(1, 0)
                .sub(1, 0, 1)
                .sub(1, 0, 0)
                .label_call("fib", &[1], 1)
                .label_call("fib", &[0], 0)
                .add(0, 1, 0)
                .return_(0)
        });

        assert_eq!(
            builder.finish().finish(),
            r"func fib
    r0 <- int 2
    blt r1 r0 fib.else fib.then
@fib.then
    ret r1
@fib.else
    r0 <- int 1
    r1 <- sub r1 r0
    r0 <- sub r1 r0
    r1 <- call fib r1
    r0 <- call fib r0
    r0 <- add r0 r1
    ret r0
end",
        );
    }

    #[test]
    fn test_sub_label_builder_build() {
        generativity::make_guard!(guard);

        let mut builder = SubLabelBuilder::new("fib", "else", guard.into());
        builder
            .integer(1, 0)
            .sub(1, 0, 1)
            .sub(1, 0, 0)
            .label_call("fib", &[1], 1)
            .label_call("fib", &[0], 0)
            .add(0, 1, 0)
            .return_(0);

        assert_eq!(
            builder.finish().finish(),
            r"@fib.else
    r0 <- int 1
    r1 <- sub r1 r0
    r0 <- sub r1 r0
    r1 <- call fib r1
    r0 <- call fib r0
    r0 <- add r0 r1
    ret r0"
        );
    }

    #[test]
    #[should_panic]
    fn test_sub_label_builder_panics_without_finish() {
        generativity::make_guard!(guard);

        let mut label = LabelBuilder::new("test", guard.into());
        let mut sub_label = label.build_sub_label("0");

        sub_label.integer(0, 0).return_(0);
    }
}
