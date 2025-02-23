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
    collections::VecDeque, hash,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    sync::Arc,
};


use pycavalry_diagnostics::Reporter;

use crate::TType;

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
    pub annotation: TType,
    pub found_types: Vec<TType>,
}

impl StatementSynthDataReturn {
    pub fn new(annotation: TType) -> StatementSynthDataReturn {
        StatementSynthDataReturn {
            annotation,
            found_types: vec![],
        }
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

impl Default for Info {
    fn default() -> Self {
        Self::new(Arc::new("unknown".into()), Arc::new("".into()))
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
