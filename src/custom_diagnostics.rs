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

impl From<RevealTypeDiag> for Box<dyn Diag> {
    fn from(val: RevealTypeDiag) -> Self {
        Box::new(val)
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
