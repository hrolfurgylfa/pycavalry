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
use ruff_python_ast::{Expr, ExprContext, Number, Stmt, StmtFunctionDef};
use ruff_text_size::{Ranged, TextRange};
use std::{fmt, mem};

use crate::diagnostic::{Diagnostic, DiagnosticType};
use crate::helpers::read_exact_from_file;
use crate::scope::Scope;
use crate::state::{Info, StatementSynthData, StatementSynthDataReturn};
use crate::types::{is_subtype, union, Class, Function, Type, TypeLiteral};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Annotation {
    Type(RangedType),
    PartialAnnotation(PartialAnnotation),
}

impl Ranged for Annotation {
    fn range(&self) -> TextRange {
        match self {
            Annotation::Type(a) => a.range.range(),
            Annotation::PartialAnnotation(a) => a.range.range(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PartialAnnotationType {
    Union,
    Literal,
}

impl fmt::Display for PartialAnnotationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match *self {
            Self::Union => "Union",
            Self::Literal => "Literal",
        };
        write!(f, "{}", name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PartialAnnotation {
    range: TextRange,
    annotation: PartialAnnotationType,
    arguments: Vec<Annotation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct RangedType {
    range: TextRange,
    value: Type,
}

fn verify_annotation(ann: Annotation) -> Result<Type, Diagnostic> {
    match ann {
        Annotation::Type(t) => Ok(t.value),
        Annotation::PartialAnnotation(t) => match t.annotation {
            PartialAnnotationType::Union => Ok(union(
                t.arguments
                    .into_iter()
                    .map(verify_annotation)
                    .collect::<Result<Vec<Type>, Diagnostic>>()?,
            )),
            PartialAnnotationType::Literal => {
                let mut literals = Vec::with_capacity(t.arguments.len());
                for arg in t.arguments {
                    match arg {
                        Annotation::Type(t) => match t.value {
                            Type::Literal(l) => literals.push(Type::Literal(l)),
                            other => {
                                return Err(Diagnostic::error(
                                    format!("Expecting literal, found {}", other),
                                    t.range,
                                ));
                            }
                        },
                        Annotation::PartialAnnotation(p) => {
                            return Err(Diagnostic::error(
                                format!("Expecting literal, found {}", p.annotation),
                                p.range,
                            ));
                        }
                    }
                }
                Ok(union(literals))
            }
        },
    }
}

fn synth_annotation(
    info: &Info,
    scope: &mut Scope,
    maybe_ast: Option<Expr>,
) -> Result<Type, Diagnostic> {
    let ann = _synth_annotation(info, scope, maybe_ast)?;
    verify_annotation(ann)
}

fn _synth_annotation(
    info: &Info,
    scope: &mut Scope,
    maybe_ast: Option<Expr>,
) -> Result<Annotation, Diagnostic> {
    let Some(ast) = maybe_ast else {
        return Ok(Annotation::Type(RangedType {
            value: Type::Unknown,
            range: TextRange::default(),
        }));
    };

    match ast {
        // TODO: Make sure Literal get arguments!
        Expr::Subscript(s) => {
            let value_range = s.value.range();
            let mut value = match _synth_annotation(info, scope, Some(*s.value))? {
                Annotation::PartialAnnotation(value) => value,
                Annotation::Type(typ) => {
                    return Err(Diagnostic::error(
                        format!("Type {} doesn't support type arguments.", typ.value),
                        value_range,
                    ));
                }
            };
            match *s.slice {
                Expr::Tuple(tuple) => {
                    for elem in tuple.elts.into_iter() {
                        let arg = _synth_annotation(info, scope, Some(elem))?;
                        value.arguments.push(arg);
                    }
                }
                other => {
                    let slice = _synth_annotation(info, scope, Some(other))?;
                    value.arguments.push(slice);
                }
            };
            Ok(Annotation::PartialAnnotation(value))
        }
        Expr::Name(n) => {
            let range = n.range();
            let typ = match scope.get(&n.id) {
                Some(t) => t.typ,
                None => {
                    // Parse partial annotations
                    if let Some(partial_annotation_type) = match n.id.as_str() {
                        "Union" => Some(PartialAnnotationType::Union),
                        "Literal" => Some(PartialAnnotationType::Literal),
                        _ => None,
                    } {
                        return Ok(Annotation::PartialAnnotation(PartialAnnotation {
                            annotation: partial_annotation_type,
                            arguments: vec![],
                            range,
                        }));
                    };

                    // Parse regular types
                    match n.id.as_str() {
                        // TODO: Remove this hardcoded non-import
                        "Any" => Type::Any,
                        "Unknown" => Type::Unknown,
                        "str" => Type::String,
                        "int" => Type::Int,
                        "float" => Type::Float,
                        "bool" => Type::Bool,
                        "None" => Type::None,
                        "..." => Type::Ellipsis,
                        unknown => {
                            return Err(Diagnostic::new(
                                format!("Name {} not found in scope.", unknown),
                                DiagnosticType::Error,
                                n.range(),
                            ));
                        }
                    }
                }
            };
            Ok(Annotation::Type(RangedType { range, value: typ }))
        }
        Expr::StringLiteral(l) => Ok(Annotation::Type(RangedType {
            value: Type::Literal(TypeLiteral::StringLiteral(l.value.to_str().to_owned())),
            range: l.range(),
        })),
        Expr::BytesLiteral(_) => unimplemented!("Bytes literal not supported."),
        Expr::NumberLiteral(l) => {
            let range = l.range();
            let literal = match l.value {
                Number::Int(i) => TypeLiteral::IntLiteral(i.as_i64().unwrap()),
                Number::Float(i) => TypeLiteral::FloatLiteral(i.to_string()),
                Number::Complex { real: _, imag: _ } => {
                    unimplemented!("Complex numbers not supported.")
                }
            };
            Ok(Annotation::Type(RangedType {
                value: Type::Literal(literal),
                range,
            }))
        }
        Expr::BooleanLiteral(l) => Ok(Annotation::Type(RangedType {
            value: Type::Literal(TypeLiteral::BooleanLiteral(l.value)),
            range: l.range(),
        })),
        Expr::NoneLiteral(l) => Ok(Annotation::Type(RangedType {
            value: Type::Literal(TypeLiteral::NoneLiteral),
            range: l.range(),
        })),
        Expr::EllipsisLiteral(l) => Ok(Annotation::Type(RangedType {
            value: Type::Literal(TypeLiteral::EllipsisLiteral),
            range: l.range(),
        })),
        e => unimplemented!("{:?}", e),
    }
}

fn synth(info: &Info, scope: &mut Scope, ast: Expr) -> Result<Type, Diagnostic> {
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
            if let Some(scoped) = scope.get(&name.id) {
                Ok(scoped.typ)
            } else {
                Err(Diagnostic::new(
                    format!("Name {} not found in scope.", name.id),
                    DiagnosticType::Error,
                    name.range,
                ))
            }
        }
        Expr::Lambda(lambda) => {
            let mut args: Vec<Type> = vec![];
            let mut arg_names: Vec<String> = vec![];
            if let Some(params) = lambda.parameters {
                for arg in params.args.into_iter() {
                    let ann = arg
                        .parameter
                        .annotation
                        .map(|a| synth(info, scope, *a))
                        .unwrap_or_else(|| Ok(Type::Unknown))?;
                    let param_name = arg.parameter.name.id;
                    args.push(ann);
                    arg_names.push(param_name);
                }
            }
            let ret = Box::new(synth(info, scope, *lambda.body)?);
            Ok(Type::Function(Function::new(args, arg_names, ret)))
        }
        Expr::Call(call) => {
            // Early handling for reveal_type
            match *call.func.clone() {
                Expr::Name(func_name) => {
                    if func_name.id == "reveal_type" {
                        let arg = call.arguments.args.into_iter().next().unwrap();
                        let typ = synth(info, scope, arg.clone())?;
                        return Err(Diagnostic::info(
                            format!(
                                "Type of \"{}\" is \"{}\"",
                                read_exact_from_file(&info.file_content, arg.range()),
                                typ
                            ),
                            func_name.range,
                        ));
                    }
                }
                _ => (),
            }

            // Regular call handling
            let callee_range = call.func.range();
            let call_range = call.range();
            let callee = match synth(info, scope, *call.func)? {
                Type::Function(func) => func,
                type_ => Err(Diagnostic::error(
                    format!("{} not callable", type_),
                    callee_range,
                ))?,
            };
            if callee.args.len() != call.arguments.len() {
                Err(Diagnostic::error(
                    format!(
                        "expected {} args, got {} args",
                        callee.args.len(),
                        call.arguments.args.len()
                    ),
                    call_range,
                ))?
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

fn check(info: &Info, scope: &mut Scope, ast: Expr, typ: Type) -> Result<Type, Diagnostic> {
    let range = ast.range();
    let synth_type = synth(info, scope, ast)?;
    if is_subtype(&synth_type, &typ) {
        Ok(synth_type)
    } else {
        Err(Diagnostic::error(
            format!("expected {typ}, got {synth_type}"),
            range,
        ))
    }
}

fn check_func(
    info: &Info,
    data: &mut StatementSynthData,
    scope: &mut Scope,
    def: StmtFunctionDef,
) -> (Function, Vec<Diagnostic>) {
    let mut errors = vec![];
    let expected_ret = match synth_annotation(info, scope, def.returns.map(|i| *i)) {
        Ok(t) => t,
        Err(e) => {
            errors.push(e);
            Type::Unknown
        }
    };

    scope.add_scope();
    // Load function arguments
    let mut args = vec![];
    let mut arg_names = vec![];
    for arg in def.parameters.args {
        let annotation = match synth_annotation(info, scope, arg.parameter.annotation.map(|i| *i)) {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                Type::Unknown
            }
        };
        let mut arg_type_added = false;
        if let Some(default) = arg.default {
            match check(info, scope, *default, annotation.clone()) {
                Ok(t) => {
                    args.push(t);
                    arg_type_added = true;
                }
                Err(e) => errors.push(e),
            };
        }
        if !arg_type_added {
            args.push(annotation.clone());
        }
        scope.set(arg.parameter.name.id.clone(), annotation);
        arg_names.push(arg.parameter.name.id);
    }

    // Get ready for synthasizing the statements
    let mut func = Function {
        args,
        arg_names,
        ret: Box::new(Type::Unknown),
    };
    let new_ret_data = StatementSynthDataReturn::new(expected_ret);
    let prev_data = mem::replace(&mut data.returns, Some(new_ret_data));

    // Synth statements
    for stmt in def.body {
        match check_statement(info, data, scope, stmt) {
            Ok(_) => (),
            Err(e) => errors.extend(e),
        }
    }

    // Put the data back for the potential outer function
    let this_func_data = mem::replace(&mut data.returns, prev_data);
    func.ret = Box::new(union(this_func_data.unwrap().found_types));

    scope.pop_scope();

    (func, errors)
}

pub fn check_statement(
    info: &Info,
    mut data: &mut StatementSynthData,
    scope: &mut Scope,
    stmt: Stmt,
) -> Result<(), Vec<Diagnostic>> {
    match stmt {
        Stmt::AnnAssign(ass) => {
            let mut errors = vec![];
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
                    scope.set(name.id, annotation);
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
            let typ = synth(info, scope, *ass.value).map_err(|e| vec![e])?;
            for target in ass.targets {
                match target {
                    Expr::Name(name) => {
                        assert_eq!(name.ctx, ExprContext::Store);
                        scope.set(name.id, typ.clone());
                    }
                    node => panic!("Node {:?} not expected in assignment.", node),
                }
            }
            Ok(())
        }
        Stmt::Expr(expr) => {
            synth(info, scope, *expr.value).map_err(|e| vec![e])?;
            Ok(())
        }
        Stmt::Return(ret) => {
            let Some(mut returns) = data.returns.clone() else {
                return Err(vec![Diagnostic::error(
                    format!("Can't return outside of function."),
                    ret.range,
                )]);
            };
            let ret = ret
                .value
                .map(|i| check(info, scope, *i, returns.annotation.clone()))
                .unwrap_or(Ok(Type::None))
                .map_err(|e| vec![e])?;
            returns.found_types.push(ret);
            data.returns = Some(returns);
            // TODO: Add the new return value into returns
            Ok(())
        }
        Stmt::FunctionDef(def) => {
            let func_name = def.name.id.clone();

            let (parsed_func, errors) = check_func(info, &mut data, scope, def);
            scope.set(func_name, Type::Function(parsed_func));

            if errors.len() > 0 {
                Err(errors)
            } else {
                Ok(())
            }
        }
        Stmt::ClassDef(def) => {
            let cls_name = def.name.id.clone();
            scope.set(
                cls_name.clone(),
                Type::Class(Class::new(cls_name, vec![], vec![])),
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
