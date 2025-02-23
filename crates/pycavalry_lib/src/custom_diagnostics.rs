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

use pycavalry_diagnostics::{convert_range, custom_diagnostic, Diag, DiagReport, DiagnosticType};

use crate::TType;
use ariadne::{Fmt, Label, Report};
use ruff_text_size::TextRange;

custom_diagnostic!(
    (RevealTypeDiag, self, DiagnosticType::Info),
    (typ: TType),
    |s: &RevealTypeDiag, c| format!("Type is {}", (&s.typ).fg(c))
);

custom_diagnostic!(
    (NotInScopeDiag, self, DiagnosticType::Error),
    (name: Arc<String>),
    |s: &NotInScopeDiag, _| format!("Name \"{}\" not found in scope.", &s.name)
);

custom_diagnostic!(
    (ExpectedButGotDiag, self, DiagnosticType::Error),
    (expected: TType, got: TType),
    |s: &ExpectedButGotDiag, _| format!("Expected {} but found {}.", s.expected, s.got)
);

custom_diagnostic!(
    (CantReassignLockedDiag, self, DiagnosticType::Error),
    (expected: TType, got: TType, name: Arc<String>),
    |s: &CantReassignLockedDiag, _| format!("\"{0}\" is already defined as {1}, can't redefine as {2} as it was previously defined with a type hint, so it can't be redefined as a different type.", &s.name, s.expected, s.got)
);
