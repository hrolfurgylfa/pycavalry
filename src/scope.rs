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

use std::{collections::HashMap, iter};

use crate::types::Type;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedType {
    pub typ: Type,
    pub is_locked: bool,
}

impl ScopedType {
    fn new(typ: Type) -> ScopedType {
        ScopedType {
            typ,
            is_locked: false,
        }
    }
}

impl From<Type> for ScopedType {
    fn from(value: Type) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    global: HashMap<String, ScopedType>,
    scopes: Vec<HashMap<String, ScopedType>>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            global: HashMap::new(),
            scopes: Vec::new(),
        }
    }
    fn top_scope(&self) -> &HashMap<String, ScopedType> {
        self.scopes.last().unwrap_or(&self.global)
    }
    fn top_scope_mut(&mut self) -> &mut HashMap<String, ScopedType> {
        self.scopes.last_mut().unwrap_or(&mut self.global)
    }
    fn all_scopes<'a>(
        &'a self,
    ) -> iter::Chain<
        iter::Rev<std::slice::Iter<'a, HashMap<String, ScopedType>>>,
        iter::Once<&HashMap<String, ScopedType>>,
    > {
        self.scopes.iter().rev().chain(iter::once(&self.global))
    }
    /// Get a variable from the top scope or None if that scope doesn't contain the provided
    /// variable
    pub fn get_top(&self, name: &str) -> Option<ScopedType> {
        self.top_scope().get(name).map(|v| v.clone())
    }
    // Get a variable from any scope
    pub fn get(&self, name: &str) -> Option<ScopedType> {
        for scope in self.all_scopes() {
            let maybe_type = scope.get(name);
            if let Some(typ) = maybe_type {
                return Some(typ.clone());
            }
        }

        None
    }
    pub fn set(&mut self, name: String, value: impl Into<ScopedType>) {
        self.top_scope_mut().insert(name, value.into());
    }
    pub fn add_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }
    pub fn pop_scope(&mut self) {
        assert_ne!(self.scopes.pop(), None)
    }
}
