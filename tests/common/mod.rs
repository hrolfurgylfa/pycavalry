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

use std::{ops::Range, path::PathBuf, sync::Arc};

use pycavalry::Diag;
use pycavalry::Scope;
use pycavalry::Type;
use pycavalry::{error_check_file, synth_annotation, Info};
use ruff_python_parser::{parse, Mode};
use ruff_text_size::{TextRange, TextSize};

/// Shorthand to quickly create an **a**rc **st**ring.
#[allow(dead_code)]
pub fn ars(s: impl Into<String>) -> Arc<String> {
    Arc::new(s.into())
}
/// Quckly create a text range from a rust range.
pub fn r(r: Range<u32>) -> TextRange {
    TextRange::new(TextSize::from(r.start), TextSize::from(r.end))
}
/// Quckly create a type from a python annotation.
pub fn ann(s: &str) -> Type {
    let info = Info::default();
    let module = parse(s, Mode::Expression).unwrap();
    let parsed = match module.into_syntax() {
        ruff_python_ast::Mod::Module(_) => unreachable!(),
        ruff_python_ast::Mod::Expression(e) => e,
    };
    let typ = synth_annotation(&info, &mut Scope::new(), Some(*parsed.body));
    assert_errors(&info, vec![]);
    assert_ne!(typ, Type::Unknown);
    typ
}

pub fn assert_errors(info: &Info, expected: Vec<Box<dyn Diag>>) {
    let errors_lock = info.reporter.errors();
    let errors = errors_lock.lock().unwrap();
    if errors.len() != expected.len() {
        println!(
            "\nDifferent count of expected vs received errors. Expected:\n{:?}\n\nReceived:\n{:?}",
            expected, errors
        );
        panic!("");
    }
    for (error, expected) in errors.iter().zip(expected.iter()) {
        assert_eq!(error, expected);
    }
}
pub fn run_with_errors(
    filename: impl Into<PathBuf>,
    content: impl Into<String>,
    expected: Vec<Box<dyn Diag>>,
) {
    let info = error_check_file(filename.into(), content.into()).unwrap();
    assert_errors(&info, expected);
}
