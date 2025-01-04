// pycavalry, a type checker for Python programs.
// Copyright (C) 2024  Hrólfur Gylfason
//
// This program is free software: you can redistribute it and/or modify
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

use clap::Parser;
use clio::{ClioPath, Output};
use ruff_python_parser::{parse, Mode};
use scope::Scope;
use state::{Info, StatementSynthData};
use std::{
    fs::read,
    io::{self, Write},
    path::{Path, PathBuf},
    string::FromUtf8Error,
    sync::Arc,
};
use synth::check_statement;

pub mod diagnostics;
pub mod helpers;
pub mod scope;
pub mod state;
pub mod synth;
pub mod types;

#[derive(Parser)]
#[clap(name = "pycavalry")]
struct Opt {
    #[clap()]
    file: PathBuf,

    /// Output file '-' for stdout
    #[clap(long, short, value_parser, default_value = "-")]
    output: Output,

    /// Directory to store log files in
    #[clap(long, short, value_parser = clap::value_parser!(ClioPath).exists().is_dir(), default_value = ".")]
    log_dir: ClioPath,
}

#[allow(dead_code)]
#[derive(Debug)]
enum Error {
    Io(io::Error),
    FromUtf8(FromUtf8Error),
    RuffParse(Vec<ruff_python_parser::ParseError>),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Self::FromUtf8(value)
    }
}

impl From<ruff_python_parser::ParseError> for Error {
    fn from(value: ruff_python_parser::ParseError) -> Self {
        Self::RuffParse(vec![value])
    }
}

impl From<&[ruff_python_parser::ParseError]> for Error {
    fn from(value: &[ruff_python_parser::ParseError]) -> Self {
        Self::RuffParse(value.to_owned())
    }
}

fn error_check_file(name: PathBuf, content: String) -> Result<Info, Error> {
    // Parse the module with ruff
    let module = parse(&content, Mode::Module)?;
    let errors = module.errors();
    if !errors.is_empty() {
        return Err(errors.into());
    }

    let mut scope = Scope::new();
    let info = Info::new(Arc::new(name), Arc::new(content));
    let mut data = StatementSynthData::new(None);
    let module = match module.into_syntax() {
        ruff_python_ast::Mod::Module(m) => m,
        ruff_python_ast::Mod::Expression(_) => unreachable!(),
    };
    for stmt in module.body.into_iter() {
        check_statement(&info, &mut data, &mut scope, stmt);
    }
    Ok(info)
}

fn read_file(file_name: &Path) -> Result<String, Error> {
    let bytes = read(file_name)?;
    let content = String::from_utf8(bytes)?;
    Ok(content)
}

fn read_and_check(file_name: PathBuf) -> Result<Info, Error> {
    let content = read_file(&file_name)?;
    error_check_file(file_name, content)
}

fn main() -> Result<(), Error> {
    let mut opt = Opt::parse();

    match read_and_check(opt.file) {
        Ok(info) => {
            let error_count = info.reporter.len();
            info.reporter.flush(&info, &mut opt.output)?;
            if error_count > 0 {
                writeln!(opt.output, "Found {} errors", error_count)?;
            } else {
                writeln!(opt.output, "No errors found")?;
            }
        }
        Err(e) => match e {
            Error::Io(e) => {
                write!(opt.output, "Failed to open file: {}", e)?;
            }
            Error::FromUtf8(e) => {
                write!(opt.output, "File contains invalid UTF8 sequences: {}", e)?;
            }
            Error::RuffParse(errors) => {
                writeln!(opt.output, "Failed to parse Python into AST:")?;
                for error in errors {
                    write!(opt.output, "{}", error)?;
                }
            }
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use diagnostics::{
        custom::{CantReassignLockedDiag, ExpectedButGotDiag, RevealTypeDiag},
        Diag,
    };
    use indoc::indoc;
    use ruff_text_size::{TextRange, TextSize};
    use synth::synth_annotation;
    use types::Type;

    use super::*;

    /// Shorthand to quickly create an **a**rc **st**ring.
    fn ars(s: impl Into<String>) -> Arc<String> {
        Arc::new(s.into())
    }
    /// Quckly create a text range from a rust range.
    fn r(r: Range<u32>) -> TextRange {
        TextRange::new(TextSize::from(r.start), TextSize::from(r.end))
    }
    /// Quckly create a type from a python annotation.
    fn ann(s: &str) -> Type {
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

    fn assert_errors(info: &Info, expected: Vec<Box<dyn Diag>>) {
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
    fn run_with_errors(
        filename: impl Into<PathBuf>,
        content: impl Into<String>,
        expected: Vec<Box<dyn Diag>>,
    ) {
        let info = error_check_file(filename.into(), content.into()).unwrap();
        assert_errors(&info, expected);
    }

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
                CantReassignLockedDiag::new(Type::Int, ann("Literal[5]"), ars("a"), r(57..74))
                    .into(),
                RevealTypeDiag::new(Type::Int, r(87..88)).into(),
                ExpectedButGotDiag::new(Type::Int, ann("Literal['f']"), r(99..102)).into(),
                CantReassignLockedDiag::new(Type::Int, Type::Int, ars("a"), r(90..102)).into(),
                RevealTypeDiag::new(Type::Int, r(115..116)).into(),
            ],
        );
    }
}
