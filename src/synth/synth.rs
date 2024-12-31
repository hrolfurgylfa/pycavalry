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
use ruff_python_ast::{Expr, ExprContext, Number, Stmt};
use ruff_text_size::Ranged;
use std::mem;
use std::sync::Arc;

use crate::custom_diagnostics::RevealTypeDiag;
use crate::scope::{Scope, ScopedType};
use crate::state::{Info, PartialItem, StatementSynthData, StatementSynthDataReturn};
use crate::types::{is_subtype, union, Class, Function, PartialFunction, Type, TypeLiteral};

use super::synth_annotation;

fn synth(info: &Info, scope: &mut Scope, ast: Expr) -> Type {
    match ast {
        Expr::NoneLiteral(_) => Type::None,
        Expr::BooleanLiteral(l) => Type::Literal(TypeLiteral::BooleanLiteral(l.value)),
        Expr::NumberLiteral(n) => match n.value {
            Number::Int(l) => Type::Literal(TypeLiteral::IntLiteral(l.as_i64().unwrap())),
            Number::Float(l) => Type::Literal(TypeLiteral::FloatLiteral(l.to_string())),
            Number::Complex { real: _, imag: _ } => unimplemented!(),
        },
        Expr::StringLiteral(s) => {
            Type::Literal(TypeLiteral::StringLiteral(s.value.to_str().to_owned()))
        }
        Expr::Name(name) if name.ctx == ExprContext::Load => {
            let name_str = Arc::new(name.id);
            if let Some(scoped) = scope.get(&name_str) {
                scoped.typ
            } else {
                info.reporter.error(
                    format!("Name \"{}\" not found in scope.", name_str),
                    name.range,
                );
                Type::Unknown
            }
        }
        Expr::Lambda(lambda) => {
            let mut args: Vec<Type> = vec![];
            let mut arg_names = vec![];
            if let Some(params) = lambda.parameters {
                for arg in params.args.into_iter() {
                    let ann = arg
                        .parameter
                        .annotation
                        .map(|a| synth(info, scope, *a))
                        .unwrap_or(Type::Unknown);
                    let param_name = arg.parameter.name.id;
                    args.push(ann);
                    arg_names.push(Arc::new(param_name));
                }
            }
            let ret = Box::new(synth(info, scope, *lambda.body));
            Type::Function(Function::new(args, arg_names, ret))
        }
        Expr::Call(mut call) => {
            // Early handling for reveal_type
            let func = match *call.func {
                Expr::Name(func_name) if func_name.id == "reveal_type" => {
                    // TODO: Find out why this isn't giving me a owned value
                    let arg = call.arguments.args.iter().nth(0).unwrap();
                    let arg_range = arg.range();
                    let typ = synth(info, scope, arg.clone());
                    info.reporter.add(RevealTypeDiag {
                        range: arg_range,
                        typ,
                    });
                    return Type::Unknown;
                }
                func => func,
            };
            // Re-assemble the call, we didn't need it in the end
            call.func = Box::new(func);

            // Regular call handling
            let callee_range = call.func.range();
            let call_range = call.range();
            let callee = match synth(info, scope, *call.func) {
                Type::Function(func) => func,
                type_ => {
                    info.reporter
                        .error(format!("{} not callable", type_), callee_range);
                    return Type::Unknown;
                }
            };
            if callee.args.len() != call.arguments.len() {
                info.reporter.error(
                    format!(
                        "expected {} args, got {} args",
                        callee.args.len(),
                        call.arguments.args.len()
                    ),
                    call_range,
                );
                return Type::Unknown;
            }
            for (expected_arg, got_arg) in callee.args.into_iter().zip(call.arguments.args.iter()) {
                check(info, scope, got_arg.clone(), expected_arg);
            }
            *callee.ret
        }
        e => unimplemented!("Unknown expression for synth: {e:?}"),
    }
}

fn check(info: &Info, scope: &mut Scope, ast: Expr, typ: Type) -> Type {
    let range = ast.range();
    let synth_type = synth(info, scope, ast);
    if is_subtype(&synth_type, &typ) {
        synth_type
    } else {
        info.reporter
            .error(format!("expected {typ}, got {synth_type}"), range);
        Type::Unknown
    }
}

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
        let arg_name = Arc::new(arg.parameter.name.id.clone());
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

pub fn check_statement(
    info: &Info,
    mut data: &mut StatementSynthData,
    scope: &mut Scope,
    stmt: Stmt,
) {
    match stmt {
        Stmt::AnnAssign(ass) => {
            let annotation = synth_annotation(info, scope, Some(*ass.annotation));
            if let Some(value) = ass.value {
                check(info, scope, *value, annotation.clone());
            };
            match *ass.target {
                Expr::Name(name) => {
                    assert_eq!(name.ctx, ExprContext::Store);
                    scope.set(Arc::new(name.id), ScopedType::locked(annotation));
                }
                node => panic!("Node {:?} not expected in type assignment.", node),
            }
        }
        Stmt::Assign(ass) => {
            for target in ass.targets {
                match target {
                    Expr::Name(name) => {
                        assert_eq!(name.ctx, ExprContext::Store);
                        let name_str = Arc::new(name.id);
                        let typ = match scope.get_top_ref(&name_str) {
                            Some(scoped) if scoped.is_locked == true => {
                                check(info, scope, *ass.value.clone(), scoped.typ.clone())
                            }
                            _ => synth(info, scope, *ass.value.clone()),
                        };
                        scope.set(name_str, typ.clone());
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
                    .error(format!("Can't return outside of function."), ret.range);
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
            let func_name = Arc::new(def.name.id.clone());

            let mut partial_func = PartialFunction {
                ast: def,
                args: None,
                arg_names: None,
                ret: None,
            };
            check_func(info, &mut data, scope, &mut partial_func);
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
            let cls_name = Arc::new(def.name.id.clone());
            scope.set(
                cls_name.clone(),
                Type::Class(Class::new(cls_name.clone(), vec![], vec![])),
            );
        }
        Stmt::Pass(_) => (),
        // TODO: Implement imports
        Stmt::Import(_) => (),
        Stmt::ImportFrom(_) => (),
        node => panic!("Statement not yet supported: {:?}", node),
    }
}
