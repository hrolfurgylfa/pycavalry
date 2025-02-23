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

pub struct Ident(String);

pub enum JinjaStatement {
    Include(),
    Extends(),
    Macro(JinjaMacro),
    For(),
    If(),
    Block(),
    With(),
    Call(),
    Filter(),
    Set(),
    Expression(JinjaExpression),
}

pub struct JinjaMacro {
    name: Ident,
    args: Vec<Ident>,
    body: Vec<JinjaStatement>,
}

pub struct JinjaExpression {
    expr: String,
    filters: Vec<String>
}
