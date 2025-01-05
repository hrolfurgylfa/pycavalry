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

use std::io;
use std::sync::Arc;
use std::{path::PathBuf, string::FromUtf8Error};

use ruff_python_parser::{parse, Mode};
use state::StatementSynthData;

pub use diagnostics::{custom::*, Diag, Diagnostic, DiagnosticType};
pub use scope::{Scope, ScopedType};
pub use state::Info;
pub use synth::{check_statement, synth, synth_annotation};
pub use types::{Type, TypeLiteral};

mod diagnostics;
mod scope;
mod state;
mod synth;
mod types;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
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

pub fn error_check_file(name: PathBuf, content: String) -> Result<Info, Error> {
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
