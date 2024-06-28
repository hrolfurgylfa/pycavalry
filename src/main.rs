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
use diagnostic::Diag;
use ruff_python_parser::{parse, Mode};
use scope::Scope;
use state::{Info, StatementSynthData};
use std::{
    fs::read,
    io::{self, Write},
    path::PathBuf,
    process::exit,
    string::FromUtf8Error,
    sync::Arc,
};
use synth::check_statement;

pub mod custom_diagnostics;
pub mod diagnostic;
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

#[derive(Debug)]
enum Error {
    IoError(io::Error),
    FromUtf8Error(FromUtf8Error),
    RuffParseError(ruff_python_parser::ParseError),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Self::FromUtf8Error(value)
    }
}

impl From<ruff_python_parser::ParseError> for Error {
    fn from(value: ruff_python_parser::ParseError) -> Self {
        Self::RuffParseError(value)
    }
}

fn main() -> Result<(), Error> {
    let mut opt = Opt::parse();
    let file_name = opt.file;
    let file_content = Arc::new(String::from_utf8(read(&file_name)?)?);

    // Parse the module with ruff
    let module = parse(&file_content, Mode::Module)?;
    let errors = module.errors();
    if errors.len() != 0 {
        for err in errors {
            writeln!(&mut opt.output, "{}", err)?;
        }
        exit(50);
    }

    let mut scope = Scope::new();
    let info = Info::new(file_name.clone(), file_content.clone());
    let mut data = StatementSynthData::new(None);
    let statements = match module.into_syntax() {
        ruff_python_ast::Mod::Module(m) => m,
        ruff_python_ast::Mod::Expression(_) => unreachable!(),
    };
    for stmt in statements.body.into_iter() {
        match check_statement(&info, &mut data, &mut scope, stmt) {
            Ok(()) => (),
            Err(errors) => {
                for e in errors {
                    e.write(&mut opt.output, &file_name, &file_content)?;
                }
            }
        }
    }
    Ok(())
}
