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

use pycavalry_diagnostics::Diag;
use ruff_python_ast::Expr;
use ruff_python_parser::parse;

use crate::jinja_ast::{JinjaExpression, JinjaStatement};

fn parse_jinja(content: &str) -> (Vec<JinjaStatement>, Vec<Box<dyn Diag>>) {
    let mut statements = vec![];
    let mut errors = vec![];

    return (statements, errors);
}
