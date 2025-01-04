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

use std::sync::Arc;

use ariadne::{Fmt, Label, Report};
use ruff_text_size::TextRange;

use super::macros;
use crate::{
    diagnostics::{convert_range, Diag, DiagReport, DiagnosticType},
    types::Type,
};

macros::custom_diagnostic!(
    (RevealTypeDiag, self, DiagnosticType::Info),
    (typ: Type),
    |s: &RevealTypeDiag, c| format!("Type is {}", (&s.typ).fg(c))
);

macros::custom_diagnostic!(
    (NotInScopeDiag, self, DiagnosticType::Error),
    (name: Arc<String>),
    |s: &NotInScopeDiag, _| format!("Name \"{}\" not found in scope.", &s.name)
);

macros::custom_diagnostic!(
    (ExpectedButGotDiag, self, DiagnosticType::Error),
    (expected: Type, got: Type),
    |s: &ExpectedButGotDiag, _| format!("Expected {} but found {}.", s.expected, s.got)
);

macros::custom_diagnostic!(
    (CantReassignLockedDiag, self, DiagnosticType::Error),
    (expected: Type, got: Type, name: Arc<String>),
    |s: &CantReassignLockedDiag, _| format!("\"{0}\" is already defined as {1}, can't redefine as {2} as it was previously defined with a type hint, so it can't be redefined as a different type.", &s.name, s.expected, s.got)
);
