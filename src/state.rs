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

use std::path::PathBuf;

use crate::types::Type;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct StatementSynthData {
    pub returns: Option<StatementSynthDataReturn>,
}

impl StatementSynthData {
    pub fn new(returns: Option<StatementSynthDataReturn>) -> StatementSynthData {
        StatementSynthData { returns }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
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
    pub file_name: PathBuf,
    pub file_content: String,
}

impl Info {
    pub fn new(file_name: PathBuf, file_content: String) -> Info {
        Info {
            file_name,
            file_content,
        }
    }
}
