use std::borrow::Cow;
use std::ops::{Deref, DerefMut, Range};

const INDENTED_LINE_START: &str = "\n    ";
const BLOCK_END: &str = "\nend";
const ENTRY_POINT: &str = r"@__entry
    r0 <- call main
    exit";

#[derive(Clone)]
pub struct Asm {
    main: Label,
    buf: String,
}

impl Asm {
    #[must_use]
    pub fn new() -> Asm {
        let main = Label::new("main");
        Self {
            main,
            buf: ENTRY_POINT.to_string(),
        }
    }

    #[must_use]
    pub fn main(&mut self) -> &mut Label {
        &mut self.main
    }

    pub fn push_label(&mut self, label: Label) {
        let label = label.finish();
        self.buf.push_str("\n\n");
        self.buf.push_str(&label);
    }

    #[must_use]
    pub fn finish(self) -> String {
        let Asm { main, mut buf } = self;
        let main = main.finish();
        buf.push_str("\n\n");
        buf.push_str(&main);
        buf
    }
}

impl Default for Asm {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct Label {
    inner: LabelImpl,
    sub_labels: Vec<SubLabel>,
}

impl Label {
    #[must_use]
    pub fn new(name: &str) -> Label {
        let name_span = {
            let start = "func ".len();
            //              func_   {name}
            let end = start + name.len();
            start..end
        };
        let buf = Self::format_name(name);

        Self {
            inner: LabelImpl::new(buf, name_span),
            sub_labels: Vec::new(),
        }
    }

    pub fn push_sub_label(&mut self, sub_label: SubLabel) {
        self.sub_labels.push(sub_label);
    }

    #[must_use]
    pub fn finish(self) -> String {
        let Label {
            inner, sub_labels, ..
        } = self;
        let mut buf = inner.finish();
        for sub_label in sub_labels {
            let asm = sub_label.finish();
            buf.push('\n');
            buf.push_str(&asm);
        }
        buf.push_str(BLOCK_END);
        buf
    }

    fn format_name(label_name: &str) -> String {
        format!("func {label_name}")
    }
}

impl Deref for Label {
    type Target = LabelImpl;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Label {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Clone)]
pub struct SubLabel {
    inner: LabelImpl,
}

impl SubLabel {
    #[must_use]
    pub fn new(label: &str, name: &str) -> SubLabel {
        let name_span = {
            let start = 1;
            //              @       {label}       .   {name}
            let end = start + label.len() + 1 + name.len();
            start..end
        };
        let buf = Self::format_name(label, name);
        Self {
            inner: LabelImpl::new(buf, name_span),
        }
    }

    #[must_use]
    pub fn finish(self) -> String {
        self.inner.finish()
    }

    fn format_name(label_name: &str, sub_label_name: &str) -> String {
        format!("@{label_name}.{sub_label_name}")
    }
}

impl Deref for SubLabel {
    type Target = LabelImpl;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SubLabel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Clone)]
pub struct LabelImpl {
    name_span: Range<usize>,
    buf: String,
}

impl LabelImpl {
    fn new(buf: String, name_span: Range<usize>) -> LabelImpl {
        Self { name_span, buf }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        let name_span = self.name_span.clone();
        &self.buf[name_span]
    }

    pub fn push_raw<'a>(&mut self, raw: impl Into<Cow<'a, str>>) {
        self.buf.push_str(raw.into().as_ref());
    }

    pub fn push_line<'a>(&mut self, line: impl Into<Cow<'a, str>>) {
        self.buf.push_str(INDENTED_LINE_START);
        self.buf.push_str(line.into().as_ref());
    }

    fn finish(self) -> String {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl LabelImpl {
        fn push_str(&mut self, s: &str) -> &mut Self {
            self.push_line(s);
            self
        }
    }

    #[test]
    fn test_sub_label_to_string() {
        let mut sub_label = SubLabel::new("fib", "else");
        sub_label
            .push_str("r0 <- int 1")
            .push_str("r1 <- sub r1 r0")
            .push_str("r0 <- sub r1 r0")
            .push_str("r1 <- call fib r1")
            .push_str("r0 <- call fib r0")
            .push_str("r0 <- add r0 r1")
            .push_str("ret r0");

        assert_eq!(
            sub_label.finish(),
            r"@fib.else
    r0 <- int 1
    r1 <- sub r1 r0
    r0 <- sub r1 r0
    r1 <- call fib r1
    r0 <- call fib r0
    r0 <- add r0 r1
    ret r0",
        );
    }

    #[test]
    fn test_label_to_string() {
        let mut fib_label = Label::new("fib");
        fib_label
            .push_str("r0 <- int 2")
            .push_str("blt r1 r0 fib.else fib.then");

        let mut fib_then_sub_label = SubLabel::new(fib_label.name(), "then");
        fib_then_sub_label.push_str("ret r1");

        let mut fib_else_sub_label = SubLabel::new(fib_label.name(), "else");
        fib_else_sub_label
            .push_str("r0 <- int 1")
            .push_str("r1 <- sub r1 r0")
            .push_str("r0 <- sub r1 r0")
            .push_str("r1 <- call fib r1")
            .push_str("r0 <- call fib r0")
            .push_str("r0 <- add r0 r1")
            .push_str("ret r0");

        fib_label.push_sub_label(fib_then_sub_label);
        fib_label.push_sub_label(fib_else_sub_label);

        assert_eq!(
            fib_label.finish(),
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
end"
        );
    }

    #[test]
    fn test_asm_to_string() {
        let mut asm = Asm::new();

        asm.main()
            .push_str("r0 <- int 35")
            .push_str("r0 <- call fib r0")
            .push_str("r0 <- call putn r0")
            .push_str("r0 <- int 10")
            .push_str("putchar r0")
            .push_str("exit");

        let mut fib_label = Label::new("fib");
        fib_label
            .push_str("r0 <- int 2")
            .push_str("blt r1 r0 fib.else fib.then");

        let mut fib_then_sub_label = SubLabel::new(fib_label.name(), "then");
        fib_then_sub_label.push_str("ret r1");

        let mut fib_else_sub_label = SubLabel::new(fib_label.name(), "else");
        fib_else_sub_label
            .push_str("r0 <- int 1")
            .push_str("r1 <- sub r1 r0")
            .push_str("r0 <- sub r1 r0")
            .push_str("r1 <- call fib r1")
            .push_str("r0 <- call fib r0")
            .push_str("r0 <- add r0 r1")
            .push_str("ret r0");

        fib_label.push_sub_label(fib_then_sub_label);
        fib_label.push_sub_label(fib_else_sub_label);

        asm.push_label(fib_label);

        let mut putn_label = Label::new("putn");
        putn_label.push_str("bb r1 putn.ret putn.digit");

        let mut putn_digit_sub_label = SubLabel::new("putn", "digit");
        putn_digit_sub_label
            .push_str("r0 <- int 10")
            .push_str("r0 <- div r1 r0")
            .push_str("r0 <- call putn r0")
            .push_str("r0 <- int 10")
            .push_str("r1 <- mod r1 r0")
            .push_str("r0 <- int 48")
            .push_str("r1 <- add r1 r0")
            .push_str("putchar r1");

        let mut putn_ret_sub_label = SubLabel::new("putn", "ret");
        putn_ret_sub_label
            .push_str("r0 <- int 0")
            .push_str("ret r0");

        putn_label.push_sub_label(putn_digit_sub_label);
        putn_label.push_sub_label(putn_ret_sub_label);

        asm.push_label(putn_label);

        assert_eq!(
            asm.finish(),
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
}
