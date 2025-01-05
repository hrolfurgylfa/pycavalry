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

use std::{collections::HashMap, iter, sync::Arc};

use gc::{Finalize, Trace};

use crate::{types::TType, Type};

#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
pub struct ScopedType {
    pub typ: TType,
    pub is_locked: bool,
}

impl ScopedType {
    pub fn new(typ: TType) -> ScopedType {
        ScopedType {
            typ,
            is_locked: false,
        }
    }

    pub fn locked(typ: TType) -> ScopedType {
        ScopedType {
            typ,
            is_locked: true,
        }
    }
}

impl From<TType> for ScopedType {
    fn from(value: TType) -> Self {
        Self::new(value)
    }
}

impl From<Type> for ScopedType {
    fn from(value: Type) -> Self {
        Self::new(value.into())
    }
}

type ScopeMap = HashMap<Arc<String>, ScopedType>;

#[derive(Clone, Debug, PartialEq)]
pub struct Scope {
    // builtin: Arc<HashMap<String, ScopedType>>,
    global: ScopeMap,
    scopes: Vec<ScopeMap>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Scope {
        Scope {
            global: HashMap::new(),
            scopes: Vec::new(),
        }
    }
    fn top_scope(&self) -> &ScopeMap {
        self.scopes.last().unwrap_or(&self.global)
    }
    fn top_scope_mut(&mut self) -> &mut ScopeMap {
        self.scopes.last_mut().unwrap_or(&mut self.global)
    }
    fn all_scopes(
        &self,
    ) -> iter::Chain<iter::Rev<std::slice::Iter<ScopeMap>>, iter::Once<&ScopeMap>> {
        self.scopes.iter().rev().chain(iter::once(&self.global))
    }
    pub fn get_top_ref<'a>(&'a self, name: &Arc<String>) -> Option<&'a ScopedType> {
        self.top_scope().get(name)
    }
    /// Get a variable from the top scope or None if that scope doesn't contain the provided
    /// variable
    pub fn get_top(&self, name: &Arc<String>) -> Option<ScopedType> {
        self.get_top_ref(name).cloned()
    }
    pub fn get_top_is_locked(&self, name: &Arc<String>) -> Option<bool> {
        self.get_top_ref(name).map(|i| i.is_locked)
    }
    pub fn get_ref<'a>(&'a self, name: &Arc<String>) -> Option<&'a ScopedType> {
        for scope in self.all_scopes() {
            let maybe_type = scope.get(name);
            if let Some(typ) = maybe_type {
                return Some(typ);
            }
        }

        None
    }
    /// Get a variable from any scope
    pub fn get(&self, name: &Arc<String>) -> Option<ScopedType> {
        self.get_ref(name).cloned()
    }
    pub fn get_is_locked(&self, name: &Arc<String>) -> Option<bool> {
        self.get_ref(name).map(|i| i.is_locked)
    }
    pub fn set(&mut self, name: Arc<String>, value: impl Into<ScopedType>) {
        self.top_scope_mut().insert(name, value.into());
    }
    pub fn add_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }
    pub fn pop_scope(&mut self) {
        assert_ne!(self.scopes.pop(), None)
    }
}
