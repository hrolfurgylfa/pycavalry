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
    path::PathBuf,
    process::exit,
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
    RuffParse(ruff_python_parser::ParseError),
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
        Self::RuffParse(value)
    }
}

fn main() -> Result<(), Error> {
    let mut opt = Opt::parse();
    let file_name = Arc::new(opt.file);
    let file_content = Arc::new(String::from_utf8(read(file_name.as_ref())?)?);

    // Parse the module with ruff
    let module = parse(&file_content, Mode::Module)?;
    let errors = module.errors();
    if !errors.is_empty() {
        for err in errors {
            writeln!(&mut opt.output, "{}", err)?;
        }
        exit(50);
    }

    let mut scope = Scope::new();
    let info = Info::new(file_name.clone(), file_content.clone());
    let mut data = StatementSynthData::new(None);
    let module = match module.into_syntax() {
        ruff_python_ast::Mod::Module(m) => m,
        ruff_python_ast::Mod::Expression(_) => unreachable!(),
    };
    for stmt in module.body.into_iter() {
        check_statement(&info, &mut data, &mut scope, stmt);
    }
    info.reporter.flush(&info, &mut opt.output)?;

    Ok(())
}
