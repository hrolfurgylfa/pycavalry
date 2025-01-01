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

use std::{
    collections::VecDeque,
    fmt, hash, io,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use clio::Output;
use ruff_text_size::TextRange;

use crate::{
    diagnostics::{Diag, Diagnostic, DiagnosticType},
    types::Type,
};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct StatementSynthData {
    pub returns: Option<StatementSynthDataReturn>,
    pub partial_list: VecDeque<PartialItem>,
}

impl StatementSynthData {
    pub fn new(returns: Option<StatementSynthDataReturn>) -> StatementSynthData {
        StatementSynthData {
            partial_list: VecDeque::new(),
            returns,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PartialItem {
    pub path: Arc<PathBuf>,
    pub name: Arc<String>,
}

impl PartialItem {
    pub fn new(path: Arc<PathBuf>, name: Arc<String>) -> PartialItem {
        PartialItem { path, name }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct StatementSynthDataReturn {
    pub annotation: Type,
    pub found_types: Vec<Type>,
}

impl StatementSynthDataReturn {
    pub fn new(annotation: Type) -> StatementSynthDataReturn {
        StatementSynthDataReturn {
            annotation,
            found_types: vec![],
        }
    }
}

#[derive(Clone, Default)]
pub struct Reporter(Arc<Mutex<Vec<Box<dyn Diag>>>>);

impl fmt::Debug for Reporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Reporter")
    }
}

impl Reporter {
    pub fn info(&self, body: impl Into<String>, range: TextRange) {
        self.add(Diagnostic::new(body.into(), DiagnosticType::Info, range))
    }
    pub fn warning(&self, body: impl Into<String>, range: TextRange) {
        self.add(Diagnostic::new(body.into(), DiagnosticType::Warning, range))
    }
    pub fn error(&self, body: impl Into<String>, range: TextRange) {
        self.add(Diagnostic::new(body.into(), DiagnosticType::Error, range))
    }
    pub fn add(&self, err: impl Into<Box<dyn Diag>>) {
        let mut errors = self.0.lock().unwrap();
        errors.push(err.into());
    }
    pub fn extend(&self, new_errors: impl Into<Vec<Box<dyn Diag>>>) {
        let mut errors = self.0.lock().unwrap();
        errors.extend(new_errors.into());
    }

    pub fn flush(&self, info: &Info, output: &mut Output) -> io::Result<()> {
        let errors = self.0.lock().unwrap();
        for e in errors.iter() {
            e.write(output, &info.file_name, &info.file_content)?
        }
        Ok(())
    }
    pub fn len(&self) -> usize {
        let errors = self.0.lock().unwrap();
        errors.len()
    }
}

#[derive(Clone, Debug)]
pub struct Info {
    pub file_name: Arc<PathBuf>,
    pub file_content: Arc<String>,
    pub reporter: Reporter,
}

impl hash::Hash for Info {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write(self.file_name.as_os_str().as_bytes());
        state.write(self.file_content.as_bytes());
    }
}

impl PartialEq for Info {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name && self.file_content == other.file_content
    }
}

impl Info {
    pub fn new(file_name: Arc<PathBuf>, file_content: Arc<String>) -> Self {
        Info {
            file_name,
            file_content,
            reporter: Reporter::default(),
        }
    }
}
