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
use crate::diagnostic::{Diag, Diagnostic, DiagnosticType};
use crate::scope::{Scope, ScopedType};
use crate::state::{Info, PartialItem, StatementSynthData, StatementSynthDataReturn};
use crate::types::{is_subtype, union, Class, Function, PartialFunction, Type, TypeLiteral};

use super::synth_annotation;

fn synth(info: &Info, scope: &mut Scope, ast: Expr) -> Result<Type, Box<dyn Diag>> {
    match ast {
        Expr::NoneLiteral(_) => Ok(Type::None),
        Expr::BooleanLiteral(l) => Ok(Type::Literal(TypeLiteral::BooleanLiteral(l.value))),
        Expr::NumberLiteral(n) => match n.value {
            Number::Int(l) => Ok(Type::Literal(TypeLiteral::IntLiteral(l.as_i64().unwrap()))),
            Number::Float(l) => Ok(Type::Literal(TypeLiteral::FloatLiteral(l.to_string()))),
            Number::Complex { real: _, imag: _ } => unimplemented!(),
        },
        Expr::StringLiteral(s) => Ok(Type::Literal(TypeLiteral::StringLiteral(
            s.value.to_str().to_owned(),
        ))),
        Expr::Name(name) if name.ctx == ExprContext::Load => {
            let name_str = Arc::new(name.id);
            if let Some(scoped) = scope.get(&name_str) {
                Ok(scoped.typ)
            } else {
                Err(Diagnostic::new(
                    format!("Name {} not found in scope.", name_str),
                    DiagnosticType::Error,
                    name.range,
                )
                .into())
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
                        .unwrap_or_else(|| Ok(Type::Unknown))?;
                    let param_name = arg.parameter.name.id;
                    args.push(ann);
                    arg_names.push(Arc::new(param_name));
                }
            }
            let ret = Box::new(synth(info, scope, *lambda.body)?);
            Ok(Type::Function(Function::new(args, arg_names, ret)))
        }
        Expr::Call(mut call) => {
            // Early handling for reveal_type
            let func = match *call.func {
                Expr::Name(func_name) if func_name.id == "reveal_type" => {
                    // TODO: Find out why this isn't giving me a owned value
                    let arg = call.arguments.args.into_iter().nth(0).unwrap();
                    let arg_range = arg.range();
                    let typ = synth(info, scope, arg.clone())?;
                    return Err(RevealTypeDiag {
                        range: arg_range,
                        typ,
                    }
                    .into());
                }
                func => func,
            };
            // Re-assemble the call, we didn't need it in the end
            call.func = Box::new(func);

            // Regular call handling
            let callee_range = call.func.range();
            let call_range = call.range();
            let callee = match synth(info, scope, *call.func)? {
                Type::Function(func) => func,
                type_ => {
                    Err(Diagnostic::error(format!("{} not callable", type_), callee_range).into())?
                }
            };
            if callee.args.len() != call.arguments.len() {
                Err(Diagnostic::error(
                    format!(
                        "expected {} args, got {} args",
                        callee.args.len(),
                        call.arguments.args.len()
                    ),
                    call_range,
                )
                .into())?
            }
            for (expected_arg, got_arg) in
                callee.args.into_iter().zip(call.arguments.args.into_iter())
            {
                check(info, scope, got_arg.clone(), expected_arg)?;
            }
            Ok(*callee.ret)
        }
        e => unimplemented!("Unknown expression for synth: {e:?}"),
    }
}

fn check(info: &Info, scope: &mut Scope, ast: Expr, typ: Type) -> Result<Type, Box<dyn Diag>> {
    let range = ast.range();
    let synth_type = synth(info, scope, ast)?;
    if is_subtype(&synth_type, &typ) {
        Ok(synth_type)
    } else {
        Err(Diagnostic::error(format!("expected {typ}, got {synth_type}"), range).into())
    }
}

fn check_func(
    info: &Info,
    data: &mut StatementSynthData,
    scope: &mut Scope,
    func: &mut PartialFunction,
) -> Vec<Box<dyn Diag>> {
    let mut errors: Vec<Box<dyn Diag>> = vec![];
    let expected_ret = match synth_annotation(info, scope, func.ast.returns.clone().map(|i| *i)) {
        Ok(t) => t,
        Err(e) => {
            errors.push(e.into());
            Type::Unknown
        }
    };

    scope.add_scope();
    // Load function arguments
    let mut args = vec![];
    let mut arg_names = vec![];
    for arg in func.ast.parameters.args.iter() {
        let annotation =
            match synth_annotation(info, scope, arg.parameter.annotation.clone().map(|i| *i)) {
                Ok(t) => t,
                Err(e) => {
                    errors.push(e.into());
                    Type::Unknown
                }
            };
        let mut arg_type_added = false;
        if let Some(default) = arg.default.clone() {
            match check(info, scope, *default, annotation.clone()) {
                Ok(t) => {
                    args.push(t);
                    arg_type_added = true;
                }
                Err(e) => errors.push(e.into()),
            };
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
        match check_statement(info, data, scope, stmt.clone()) {
            Ok(_) => (),
            Err(e) => errors.extend(e),
        }
    }

    // Put the data back for the potential outer function
    let this_func_data = mem::replace(&mut data.returns, prev_data);
    func.ret = Some(Box::new(union(this_func_data.unwrap().found_types)));

    scope.pop_scope();

    errors
}

pub fn check_statement(
    info: &Info,
    mut data: &mut StatementSynthData,
    scope: &mut Scope,
    stmt: Stmt,
) -> Result<(), Vec<Box<dyn Diag>>> {
    match stmt {
        Stmt::AnnAssign(ass) => {
            let mut errors: Vec<Box<dyn Diag>> = vec![];
            let annotation = match synth_annotation(info, scope, Some(*ass.annotation)) {
                Ok(t) => t,
                Err(e) => {
                    errors.push(e);
                    Type::Unknown
                }
            };
            if let Some(value) = ass.value {
                match check(info, scope, *value, annotation.clone()) {
                    Ok(_) => (),
                    Err(e) => {
                        errors.push(e);
                    }
                }
            };
            match *ass.target {
                Expr::Name(name) => {
                    assert_eq!(name.ctx, ExprContext::Store);
                    scope.set(Arc::new(name.id), ScopedType::locked(annotation));
                }
                node => panic!("Node {:?} not expected in type assignment.", node),
            }
            if errors.len() == 0 {
                Ok(())
            } else {
                Err(errors)
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
                        }
                        .map_err(|e| vec![e.into()])?;
                        scope.set(name_str, typ.clone());
                    }
                    node => panic!("Node {:?} not expected in assignment.", node),
                }
            }
            Ok(())
        }
        Stmt::Expr(expr) => {
            synth(info, scope, *expr.value).map_err(|e| vec![e.into()])?;
            Ok(())
        }
        Stmt::Return(ret) => {
            let Some(mut returns) = data.returns.clone() else {
                return Err(vec![Diagnostic::error(
                    format!("Can't return outside of function."),
                    ret.range,
                )
                .into()]);
            };
            let ret = ret
                .value
                .map(|i| check(info, scope, *i, returns.annotation.clone()))
                .unwrap_or(Ok(Type::None))
                .map_err(|e| vec![e.into()])?;
            returns.found_types.push(ret);
            data.returns = Some(returns);
            // TODO: Add the new return value into returns
            Ok(())
        }
        Stmt::FunctionDef(def) => {
            let func_name = Arc::new(def.name.id.clone());

            let mut partial_func = PartialFunction {
                ast: def,
                args: None,
                arg_names: None,
                ret: None,
            };
            let errors = check_func(info, &mut data, scope, &mut partial_func);
            let typ = match Function::try_from(partial_func) {
                Ok(func) => Type::Function(func),
                Err(func) => {
                    data.partial_list
                        .push_back(PartialItem::new(info.file_name.clone(), func_name.clone()));
                    Type::PartialFunction(func)
                }
            };
            scope.set(func_name, typ);

            if errors.len() > 0 {
                Err(errors)
            } else {
                Ok(())
            }
        }
        Stmt::ClassDef(def) => {
            let cls_name = Arc::new(def.name.id.clone());
            scope.set(
                cls_name.clone(),
                Type::Class(Class::new(cls_name.clone(), vec![], vec![])),
            );
            Ok(())
        }
        Stmt::Pass(_) => Ok(()),
        // TODO: Implement imports
        Stmt::Import(_) => Ok(()),
        Stmt::ImportFrom(_) => Ok(()),
        node => panic!("Statement not yet supported: {:?}", node),
    }
}
