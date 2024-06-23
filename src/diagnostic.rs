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
    io::{self, Write},
    path::PathBuf,
};

use clio::Output;
use ruff_text_size::TextRange;
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

    pub fn write(&self, f: &mut Output, file: &PathBuf) -> io::Result<()> {
        writeln!(
            f,
            "{}:{:?}: {}: {}",
            file.to_str().unwrap_or("Unknown"),
            self.range.start(),
            self.typ,
            self.body
        )
    }
}
