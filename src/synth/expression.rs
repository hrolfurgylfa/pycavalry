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

use ruff_python_ast::{Expr, ExprContext, Number};
use ruff_text_size::Ranged;
use std::sync::Arc;

use crate::custom_diagnostics::RevealTypeDiag;
use crate::scope::Scope;
use crate::state::Info;
use crate::types::{is_subtype, Function, Type, TypeLiteral};

pub fn synth(info: &Info, scope: &mut Scope, ast: Expr) -> Type {
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
            let name_str = Arc::new(name.id.to_string());
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
                    arg_names.push(Arc::new(param_name.to_string()));
                }
            }
            let ret = Box::new(synth(info, scope, *lambda.body));
            Type::Function(Function::new(args, arg_names, ret))
        }
        Expr::Call(mut call) => {
            // Early handling for reveal_type
            let func = match *call.func {
                Expr::Name(func_name) if func_name.id == "reveal_type" => {
                    // TODO: Get an owned value here to avoid the clone
                    let arg = call.arguments.args.first().unwrap().clone();
                    let arg_range = arg.range();
                    let typ = synth(info, scope, arg);
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
        Expr::Attribute(attr) => {
            let value = synth(info, scope, *attr.value);
            match value {
                Type::Module(_, module) => module
                    .get(&attr.attr.id.to_string())
                    .map(|t| t.typ.clone())
                    .unwrap_or(Type::Unknown),
                typ => {
                    info.reporter.error(
                        format!("Unknown attribute \"{}\" for {}", &attr.attr.id, typ),
                        attr.range,
                    );
                    Type::Unknown
                }
            }
        }
        Expr::Tuple(tuple) => Type::Tuple(
            tuple
                .elts
                .into_iter()
                .map(|expr| synth(info, scope, expr))
                .collect(),
        ),
        e => unimplemented!("Unknown expression for synth: {e:?}"),
    }
}

pub fn check(info: &Info, scope: &mut Scope, ast: Expr, typ: Type) -> Type {
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
