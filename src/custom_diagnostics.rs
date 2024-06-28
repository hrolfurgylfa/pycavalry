use ariadne::{Color, Fmt, Label, Report};
use ruff_text_size::TextRange;

use crate::{
    diagnostic::{convert_range, Diag, DiagReport},
    types::Type,
};

#[derive(Debug)]
pub struct RevealTypeDiag {
    pub typ: Type,
    pub range: TextRange,
}

impl Into<Box<dyn Diag>> for RevealTypeDiag {
    fn into(self) -> Box<dyn Diag> {
        Box::new(self) as Box<dyn Diag>
    }
}

impl Diag for RevealTypeDiag {
    fn print<'a>(&'a self, file_name: &'a str) -> DiagReport<'a> {
        let color = Color::Cyan;
        let kind = ariadne::ReportKind::Custom("Info", color);
        Report::build(kind, file_name, self.range.start().to_usize())
            .with_label(
                Label::new((file_name, convert_range(self.range)))
                    .with_message(format!("Type is {}", (&self.typ).fg(color)))
                    .with_color(color),
            )
            .finish()
    }
}
