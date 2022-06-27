use std::borrow::Cow;
use std::ops::{Deref, DerefMut, Range};

const INDENTED_LINE_START: &str = "\n    ";
const BLOCK_END: &str = "\nend";

pub struct Label {
    inner: LabelImpl,
    sub_labels: Vec<SubLabel>,
}

impl Label {
    pub fn new(name: &str) -> Label {
        let name_span = {
            let start = "func ".len();
            //              func_   {name}
            let end = start + name.len();
            start..end
        };
        let buf = Self::format_name(name);

        Self { inner: LabelImpl::new(buf, name_span), sub_labels: Vec::new() }
    }

    pub fn push_sub_label(&mut self, sub_label: SubLabel) {
        self.sub_labels.push(sub_label);
    }

    pub fn finish(self) -> String {
        let Label { inner, sub_labels, .. } = self;
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

pub struct SubLabel {
    inner: LabelImpl,
}

impl SubLabel {
    pub fn new(label: &str, name: &str) -> SubLabel {
        let name_span = {
            let start = 1;
            //              @       {label}       .   {name}
            let end = start + label.len() + 1 + name.len();
            start..end
        };
        let buf = Self::format_name(label, name);
        Self { inner: LabelImpl::new(buf, name_span) }
    }

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

pub struct LabelImpl {
    name_span: Range<usize>,
    buf: String,
}

impl LabelImpl {
    fn new(buf: String, name_span: Range<usize>) -> LabelImpl {
        Self { name_span, buf }
    }

    pub fn name(&self) -> &str {
        let name_span = self.name_span.clone();
        &self.buf[name_span]
    }

    pub fn push_line(&mut self, line: Cow<str>) {
        self.buf.push_str(INDENTED_LINE_START);
        self.buf.push_str(line.as_ref());
    }

    fn finish(self) -> String {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Label {
        fn push_str(&mut self, s: &str) -> &mut Self {
            self.push_line(s.into());
            self
        }
    }

    impl SubLabel {
        fn push_str(&mut self, s: &str) -> &mut Self {
            self.push_line(s.into());
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
}
