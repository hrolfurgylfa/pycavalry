// This file is part of pycavalry.
//
// pycavalry is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use core::fmt;
use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    io,
    ops::Range,
    path::Path,
};

use ariadne::{Color, Config, Label, Report, ReportKind, Source};
use clio::Output;
use ruff_text_size::TextRange;

use super::{dyn_compare::DynCompare, macros};

pub fn type_to_color(diagnostic_type: &DiagnosticType) -> Color {
    match diagnostic_type {
        DiagnosticType::Error => Color::Red,
        DiagnosticType::Warning => Color::Yellow,
        DiagnosticType::Info => Color::Blue,
    }
}

pub fn type_to_kind(diagnostic_type: &DiagnosticType) -> ReportKind<'static> {
    match diagnostic_type {
        DiagnosticType::Error => ariadne::ReportKind::Error,
        DiagnosticType::Warning => ariadne::ReportKind::Warning,
        DiagnosticType::Info => ariadne::ReportKind::Custom("Info", type_to_color(diagnostic_type)),
    }
}

pub type DiagReport<'a> = Report<'a, (&'a str, std::ops::Range<usize>)>;

pub trait Diag: DynCompare + Debug + Display {
    fn print<'a>(&'a self, file_name: &'a str) -> DiagReport<'a>;

    fn write(&self, f: &mut Output, file_name: &Path, file: &str) -> io::Result<()> {
        let file_name_cow = file_name.to_string_lossy();
        let file_name: &str = file_name_cow.borrow();
        self.print(file_name)
            .write((file_name, Source::from(file)), f)
    }
}

impl PartialEq<dyn Diag> for dyn Diag {
    fn eq(&self, other: &dyn Diag) -> bool {
        self.as_dyn_compare() == other.as_dyn_compare()
    }
}

#[derive(Debug, PartialEq)]
pub enum DiagnosticType {
    Info,
    Warning,
    Error,
}

impl fmt::Display for DiagnosticType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "Info"),
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Diagnostic {
    body: String,
    typ: DiagnosticType,
    range: TextRange,
}

impl Diagnostic {
    pub fn new(body: String, typ: DiagnosticType, range: TextRange) -> Diagnostic {
        Diagnostic { body, typ, range }
    }

    pub fn error(body: String, range: TextRange) -> Diagnostic {
        Diagnostic::new(body, DiagnosticType::Error, range)
    }
    pub fn warn(body: String, range: TextRange) -> Diagnostic {
        Diagnostic::new(body, DiagnosticType::Warning, range)
    }
    pub fn info(body: String, range: TextRange) -> Diagnostic {
        Diagnostic::new(body, DiagnosticType::Info, range)
    }
}

macros::impl_diagnostic_to_box!(Diagnostic);

pub fn convert_range(range: TextRange) -> Range<usize> {
    range.start().to_usize()..range.end().to_usize()
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Diagnostic({}, )", self.typ)
    }
}

impl Diag for Diagnostic {
    fn print<'a>(&'a self, file_name: &'a str) -> DiagReport<'a> {
        let main_color = type_to_color(&self.typ);
        let kind = type_to_kind(&self.typ);
        Report::build(kind, file_name, self.range.start().to_usize())
            .with_label(
                Label::new((file_name, convert_range(self.range)))
                    .with_message(&self.body)
                    .with_color(main_color),
            )
            .with_config(Config::default().with_compact(false))
            .finish()
    }
}
