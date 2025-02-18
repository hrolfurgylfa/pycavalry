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

use std::ops::Deref;
use std::{ops::Range, path::PathBuf, sync::Arc};

use pycavalry::Scope;
use pycavalry::TType;
use pycavalry::Type;
use pycavalry::{error_check_file, synth_annotation, Info};
use ruff_python_parser::{parse, Mode};
use ruff_python_trivia::CommentRanges;
use ruff_text_size::{Ranged, TextRange, TextSize};

/// Shorthand to quickly create an **a**rc **st**ring.
#[allow(dead_code)]
pub fn ars(s: impl Into<String>) -> Arc<String> {
    Arc::new(s.into())
}
/// Quickly create a text range from a rust range.
pub fn r(r: Range<u32>) -> TextRange {
    TextRange::new(TextSize::from(r.start), TextSize::from(r.end))
}
/// Quickly create a type from a python annotation.
pub fn ann(s: &str) -> TType {
    let info = Info::default();
    let module = parse(s, Mode::Expression).unwrap();
    let parsed = match module.into_syntax() {
        ruff_python_ast::Mod::Module(_) => unreachable!(),
        ruff_python_ast::Mod::Expression(e) => e,
    };
    let typ = synth_annotation(&info, &mut Scope::new(), Some(*parsed.body));
    assert_errors(&info, vec![]);
    assert_ne!(typ, Type::Unknown.into());
    typ
}

pub fn assert_errors(info: &Info, expected: Vec<&str>) {
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
        assert_eq!(&format!("{}", error), expected);
    }
}

fn get_all_expected_comments(content: &str) -> Vec<&str> {
    let module = parse(content, Mode::Module).unwrap();
    let errors = module.errors();
    if !errors.is_empty() {
        panic!("Failed to parse test module: {:?}", errors);
    }
    let comment_ranges = CommentRanges::from(module.tokens());

    let mut expected_errors = vec![];
    for range in comment_ranges.deref() {
        let found = &content[range.range()];
        println!("Found: \"{}\"", found);
        expected_errors.extend(
            found
                .lines()
                .filter_map(|a| {
                    a.strip_prefix("#")
                        .unwrap_or(a)
                        .trim_start()
                        .strip_prefix("Debug:")
                })
                .map(|a| a.trim()),
        );
    }

    expected_errors
}

pub fn run_with_errors(filename: impl Into<PathBuf>, content: impl Into<String>) {
    let content: String = content.into();
    let expected = get_all_expected_comments(&content);
    let info = error_check_file(filename.into(), content.clone()).unwrap();
    assert_errors(&info, expected);
}
