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

use std::fmt;

use ruff_python_ast::{Expr, Number};
use ruff_text_size::{Ranged, TextRange};

use crate::{
    diagnostic::{Diag, Diagnostic, DiagnosticType},
    scope::Scope,
    state::Info,
    types::{union, Type, TypeLiteral},
};

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

fn verify_annotation(ann: Annotation) -> Result<Type, Box<dyn Diag>> {
    match ann {
        Annotation::Type(t) => Ok(t.value),
        Annotation::PartialAnnotation(t) => match t.annotation {
            PartialAnnotationType::Union => Ok(union(
                t.arguments
                    .into_iter()
                    .map(verify_annotation)
                    .collect::<Result<Vec<Type>, Box<dyn Diag>>>()?,
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
                                )
                                .into());
                            }
                        },
                        Annotation::PartialAnnotation(p) => {
                            return Err(Diagnostic::error(
                                format!("Expecting literal, found {}", p.annotation),
                                p.range,
                            )
                            .into());
                        }
                    }
                }
                Ok(union(literals))
            }
        },
    }
}

pub fn synth_annotation(
    info: &Info,
    scope: &mut Scope,
    maybe_ast: Option<Expr>,
) -> Result<Type, Box<dyn Diag>> {
    let ann = _synth_annotation(info, scope, maybe_ast)?;
    verify_annotation(ann)
}

fn _synth_annotation(
    info: &Info,
    scope: &mut Scope,
    maybe_ast: Option<Expr>,
) -> Result<Annotation, Box<dyn Diag>> {
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
                    )
                    .into());
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
                            )
                            .into());
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
