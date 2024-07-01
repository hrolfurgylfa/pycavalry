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

use std::{collections::VecDeque, path::PathBuf, sync::Arc};

use crate::types::Type;

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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Info {
    pub file_name: Arc<PathBuf>,
    pub file_content: Arc<String>,
}

impl Info {
    pub fn new(file_name: Arc<PathBuf>, file_content: Arc<String>) -> Info {
        Info {
            file_name,
            file_content,
        }
    }
}
