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

use std::{
    fs::read,
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;
use clio::{ClioPath, Output};

use pycavalry::{error_check_file, Error, Info};

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
