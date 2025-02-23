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
    fmt, io, path::Path, sync::{Arc, Mutex}
};

use clio::Output;
use ruff_text_size::TextRange;

use crate::{diagnostic::{Diagnostic, DiagnosticType}, Diag};

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

    pub fn flush(&self, file_name: &Path, file_content: &str, output: &mut Output) -> io::Result<()> {
        let errors = self.0.lock().unwrap();
        for e in errors.iter() {
            e.write(output, file_name, file_content)?
        }
        Ok(())
    }
    pub fn len(&self) -> usize {
        let errors = self.0.lock().unwrap();
        errors.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn errors(&self) -> Arc<Mutex<Vec<Box<dyn Diag>>>> {
        self.0.clone()
    }
}
