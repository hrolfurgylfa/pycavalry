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

use indoc::indoc;
use pycavalry::{CantReassignLockedDiag, ExpectedButGotDiag, RevealTypeDiag, Type};

mod common;
use common::*;

#[test]
fn test_synth_ann_assign_1() {
    run_with_errors(
        "test_synth_ann_assign_1.py",
        indoc! {r#"
            from typing import reveal_type
            a: int = 3
            reveal_type(a)
            a: Literal[5] = 5
            reveal_type(a)
            a: int = "f"
            reveal_type(a) "#
        },
        vec![
            RevealTypeDiag::new(Type::Int, r(54..55)).into(),
            CantReassignLockedDiag::new(Type::Int, ann("Literal[5]"), ars("a"), r(57..74)).into(),
            RevealTypeDiag::new(Type::Int, r(87..88)).into(),
            ExpectedButGotDiag::new(Type::Int, ann("Literal['f']"), r(99..102)).into(),
            CantReassignLockedDiag::new(Type::Int, Type::Int, ars("a"), r(90..102)).into(),
            RevealTypeDiag::new(Type::Int, r(115..116)).into(),
        ],
    );
}
