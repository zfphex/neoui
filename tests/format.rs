use neoui::*;
use std::fmt;

struct Nested<'ui, 'frame, 'text>(&'ui FrameContext<'frame, 'text>);

impl fmt::Display for Nested<'_, '_, '_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str("before ")?;
        let value = std::hint::black_box("nested");
        let first = self.0.fmt(format_args!("{value}"));
        for index in 0..1024 {
            self.0.fmt(format_args!("nested {index}"));
        }
        output.write_str(first)?;
        output.write_str(" after")
    }
}

#[test]
fn formatting_pool() {
    let mut context = ui_hidden(32, 32);
    context.accessability = false;
    let value = std::hint::black_box("héllo 世界");
    let long = "λ".repeat(4096);
    let mut saved = "";

    context.frame(|ui| {
        let literal = "literal";
        assert_eq!(ui.fmt(format_args!("literal")).as_ptr(), literal.as_ptr());
        assert_eq!(ui.fmt(format_args!("")), "");
        saved = ui.fmt(format_args!("{value}"));
        let nested = ui.fmt(format_args!("prefix {} suffix", Nested(ui)));
        let large = ui.fmt(format_args!("{long}"));
        let mut strings = Vec::new();
        for index in 0..1024 {
            strings.push(ui.fmt(format_args!("value {index}")));
        }
        for (index, text) in strings.iter().enumerate() {
            assert_eq!(*text, format!("value {index}"));
        }
        assert_eq!(saved, value);
        assert_eq!(nested, "prefix before nested after suffix");
        assert_eq!(large, long);
    });

    assert_eq!(saved, value);
    let pointer = saved.as_ptr();
    context.frame(|ui| {
        let text = ui.fmt(format_args!("{value}"));
        assert_eq!(text, value);
        assert_eq!(text.as_ptr(), pointer);
    });
    context.window.close();
}
