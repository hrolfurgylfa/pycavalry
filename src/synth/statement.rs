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

use core::panic;
use ruff_python_ast::{Expr, ExprContext, Stmt};
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use crate::scope::{Scope, ScopedType};
use crate::state::{Info, PartialItem, StatementSynthData, StatementSynthDataReturn};
use crate::synth::synth;
use crate::types::{union, Class, Function, PartialFunction, Type, TypeLiteral};

use super::{check, synth_annotation};

fn check_func(
    info: &Info,
    data: &mut StatementSynthData,
    scope: &mut Scope,
    func: &mut PartialFunction,
) {
    let expected_ret = synth_annotation(info, scope, func.ast.returns.clone().map(|i| *i));

    scope.add_scope();
    // Load function arguments
    let mut args = vec![];
    let mut arg_names = vec![];
    for arg in func.ast.parameters.args.iter() {
        let annotation =
            synth_annotation(info, scope, arg.parameter.annotation.clone().map(|i| *i));
        let mut arg_type_added = false;
        if let Some(default) = arg.default.clone() {
            let t = check(info, scope, *default, annotation.clone());
            args.push(t);
            arg_type_added = true;
        }
        if !arg_type_added {
            args.push(annotation.clone());
        }
        let arg_name = Arc::new(arg.parameter.name.id.to_string());
        scope.set(arg_name.clone(), annotation);
        arg_names.push(arg_name);
    }

    // Get ready for synthasizing the statements
    func.args = Some(args);
    func.arg_names = Some(arg_names);
    func.ret = Some(Box::new(Type::Unknown));
    let new_ret_data = StatementSynthDataReturn::new(expected_ret);
    let prev_data = mem::replace(&mut data.returns, Some(new_ret_data));

    // Synth statements
    for stmt in func.ast.body.iter() {
        check_statement(info, data, scope, stmt.clone());
    }

    // Put the data back for the potential outer function
    let this_func_data = mem::replace(&mut data.returns, prev_data);
    func.ret = Some(Box::new(union(this_func_data.unwrap().found_types)));

    scope.pop_scope();
}

fn load_module(path: &str) -> HashMap<Arc<String>, ScopedType> {
    let mut module = HashMap::new();

    // Add any hardcoded extras to built in modules
    #[allow(clippy::single_match)]
    match path {
        "sys" => {
            module.insert(
                Arc::new("version_info".to_owned()),
                ScopedType::new(Type::Tuple(vec![
                    Type::Literal(TypeLiteral::IntLiteral(3)),
                    Type::Literal(TypeLiteral::IntLiteral(13)),
                ])),
            );
        }
        _ => {}
    }

    module
}

pub fn check_statement(info: &Info, data: &mut StatementSynthData, scope: &mut Scope, stmt: Stmt) {
    match stmt {
        Stmt::AnnAssign(ass) => {
            let annotation = synth_annotation(info, scope, Some(*ass.annotation));
            if let Some(value) = ass.value {
                check(info, scope, *value, annotation.clone());
            };
            match *ass.target {
                Expr::Name(name) => {
                    assert_eq!(name.ctx, ExprContext::Store);
                    scope.set(
                        Arc::new(name.id.to_string()),
                        ScopedType::locked(annotation),
                    );
                }
                node => panic!("Node {:?} not expected in type assignment.", node),
            }
        }
        Stmt::Assign(ass) => {
            for target in ass.targets {
                match target {
                    Expr::Name(name) => {
                        assert_eq!(name.ctx, ExprContext::Store);
                        let name_str = Arc::new(name.id.to_string());
                        let mut skip_assignment = false;
                        let typ = match scope.get_top_ref(&name_str) {
                            Some(scoped) if scoped.is_locked => {
                                let scoped_type = scoped.typ.clone();
                                let checked_type =
                                    check(info, scope, *ass.value.clone(), scoped.typ.clone());
                                if scoped_type != Type::Unknown && checked_type == Type::Unknown {
                                    skip_assignment = true;
                                }
                                checked_type
                            }
                            _ => synth(info, scope, *ass.value.clone()),
                        };
                        if !skip_assignment {
                            scope.set(name_str, typ);
                        }
                    }
                    node => panic!("Node {:?} not expected in assignment.", node),
                }
            }
        }
        Stmt::Expr(expr) => {
            synth(info, scope, *expr.value);
        }
        Stmt::Return(ret) => {
            let Some(mut returns) = data.returns.clone() else {
                info.reporter
                    .error("Can't return outside of function.", ret.range);
                return;
            };
            let ret = ret
                .value
                .map(|i| check(info, scope, *i, returns.annotation.clone()))
                .unwrap_or(Type::None);
            returns.found_types.push(ret);
            data.returns = Some(returns);
            // TODO: Add the new return value into returns
        }
        Stmt::FunctionDef(def) => {
            let func_name = Arc::new(def.name.id.to_string());

            let mut partial_func = PartialFunction {
                ast: def,
                args: None,
                arg_names: None,
                ret: None,
            };
            check_func(info, data, scope, &mut partial_func);
            let typ = match Function::try_from(partial_func) {
                Ok(func) => Type::Function(func),
                Err(func) => {
                    data.partial_list
                        .push_back(PartialItem::new(info.file_name.clone(), func_name.clone()));
                    Type::PartialFunction(func)
                }
            };
            scope.set(func_name, typ);
        }
        Stmt::ClassDef(def) => {
            let cls_name = Arc::new(def.name.id.to_string());
            scope.set(
                cls_name.clone(),
                Type::Class(Class::new(cls_name.clone(), vec![], vec![])),
            );
        }
        Stmt::Pass(_) => (),
        // TODO: Implement imports
        Stmt::Import(import) => {
            for alias in import.names {
                let module = load_module(&alias.name.id);
                let name = Arc::new(alias.name.id.to_string());
                scope.set(
                    name.clone(),
                    Type::Module(
                        alias
                            .asname
                            .map(|i| Arc::new(i.id.to_string()))
                            .unwrap_or(name),
                        module,
                    ),
                );
            }
        }
        Stmt::ImportFrom(import) => {
            let module = load_module(&import.module.expect("From import without module?"));
            for alias in import.names {
                let Some(submodule) = module.get(&alias.name.id.to_string()) else {
                    info.reporter.error(
                        format!("Name \"{}\" not found in scope.", &alias.name.id),
                        alias.range,
                    );
                    continue;
                };

                let name = Arc::new(alias.name.id.to_string());
                scope.set(name.clone(), submodule.clone());
            }
        }
        node => panic!("Statement not yet supported: {:?}", node),
    }
}
