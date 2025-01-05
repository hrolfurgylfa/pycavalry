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

use core::fmt;
use std::{collections::HashMap, fmt::Display, hash::Hash, ops::Deref};

use gc::{Finalize, Gc, Trace};
use ruff_python_ast::{LiteralExpressionRef, Number, StmtFunctionDef};

use crate::scope::ScopedType;

fn write_iter<I, T, F>(f: &mut fmt::Formatter<'_>, vals: I, func: F) -> fmt::Result
where
    I: ExactSizeIterator<Item = T>,
    F: Fn(&mut fmt::Formatter<'_>, T) -> fmt::Result,
{
    let vals_len = vals.len();
    for (i, t) in vals.enumerate() {
        func(f, t)?;
        if i != vals_len - 1 {
            write!(f, ", ")?;
        }
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Default, Trace, Finalize)]
pub struct TType(Gc<Type>);

impl TType {
    pub fn new(typ: Type) -> Self {
        TType(Gc::new(typ))
    }
    pub fn i(&self) -> Gc<Type> {
        self.0.clone()
    }
}

impl From<Type> for TType {
    fn from(value: Type) -> Self {
        Self::new(value)
    }
}

impl std::convert::AsRef<Type> for TType {
    fn as_ref(&self) -> &Type {
        self.0.as_ref()
    }
}

impl Deref for TType {
    type Target = Type;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Display for TType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, PartialEq, Default, Trace, Finalize)]
pub enum Type {
    Any,
    #[default]
    Unknown,
    Never,

    String,
    Int,
    Float,
    Bool,
    None,
    Ellipsis,
    Tuple(Vec<TType>),

    Literal(TypeLiteral),
    Function(Function),
    PartialFunction(PartialFunction),
    Class(Class),

    Union(Vec<TType>),
    Module(String, HashMap<String, ScopedType>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Never => write!(f, "Never"),
            Type::Any => write!(f, "Any"),
            Type::Unknown => write!(f, "Unknown"),
            Type::String => write!(f, "str"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::None => write!(f, "None"),
            Type::Ellipsis => write!(f, "..."),
            Type::Tuple(types) => {
                write!(f, "tuple[")?;
                write_iter(f, types.iter(), |f, t| write!(f, "{}", t))?;
                write!(f, "]")
            }
            Type::Literal(l) => write!(f, "{}", l),
            Type::Function(func) => write!(f, "{}", func),
            Type::PartialFunction(_) => write!(f, "Partial Func"),
            Type::Class(cls) => write!(f, "{}", cls),
            Type::Union(types) => {
                if types.iter().all(|i| matches!(i.deref(), Type::Literal(_))) {
                    write!(f, "Literal[")?;
                    write_iter(f, types.iter(), |f, t| match t.deref() {
                        Type::Literal(l) => display_type_literal_inside(f, l),
                        _ => unreachable!(),
                    })?;
                } else {
                    write!(f, "Union[")?;
                    write_iter(f, types.iter(), |f, t| write!(f, "{}", t))?;
                }
                write!(f, "]")
            }
            Type::Module(name, _) => write!(f, "module[{}]", name),
        }?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
pub struct Function {
    pub args: Vec<TType>,
    pub arg_names: Vec<String>,
    pub ret: TType,
}

#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
pub struct PartialFunction {
    #[unsafe_ignore_trace]
    pub ast: StmtFunctionDef,
    pub args: Option<Vec<TType>>,
    pub arg_names: Option<Vec<String>>,
    pub ret: Option<TType>,
}

impl TryFrom<PartialFunction> for Function {
    type Error = PartialFunction;
    fn try_from(value: PartialFunction) -> Result<Self, Self::Error> {
        if value.args.is_some() && value.arg_names.is_some() && value.ret.is_some() {
            Ok(Function {
                args: value.args.clone().unwrap(),
                arg_names: value.arg_names.clone().unwrap(),
                ret: value.ret.clone().unwrap(),
            })
        } else {
            Err(value)
        }
    }
}

impl Function {
    pub fn new(args: Vec<TType>, arg_names: Vec<String>, ret: TType) -> Function {
        Function {
            args,
            arg_names,
            ret,
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        write_iter(
            f,
            self.arg_names.iter().zip(self.args.iter()),
            |f, (name, typ)| write!(f, "{name}: {typ}"),
        )?;
        write!(f, ") -> {}", self.ret)
    }
}

#[derive(Clone, Debug, PartialEq, Trace, Finalize)]
pub struct Class {
    pub name: String,
    pub functions: Vec<Function>,
    pub parameters: Vec<(String, TType)>,
}

impl Class {
    pub fn new(name: String, functions: Vec<Function>, parameters: Vec<(String, TType)>) -> Class {
        Class {
            name,
            functions,
            parameters,
        }
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "type[{}]", self.name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeClass {
    properties: Vec<TypeClassProperty>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeClassProperty {
    name: String,
    typ: TType,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Trace, Finalize)]
pub enum TypeLiteral {
    StringLiteral(String),
    BytesLiteral(Vec<u8>),
    IntLiteral(i64),
    FloatLiteral(String),
    BooleanLiteral(bool),
    NoneLiteral,
    EllipsisLiteral,
}

fn display_type_literal_inside(f: &mut fmt::Formatter, literal: &TypeLiteral) -> fmt::Result {
    match literal {
        TypeLiteral::StringLiteral(i) => write!(f, "\"{}\"", i),
        TypeLiteral::BytesLiteral(i) => write!(f, "b\"{:?}\"", i),
        TypeLiteral::IntLiteral(i) => write!(f, "{}", i),
        TypeLiteral::FloatLiteral(i) => write!(f, "{}", i),
        TypeLiteral::BooleanLiteral(i) => write!(f, "{}", if *i { "True" } else { "False" }),
        TypeLiteral::NoneLiteral => write!(f, "None"),
        TypeLiteral::EllipsisLiteral => write!(f, "..."),
    }
}

impl fmt::Display for TypeLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Literal[")?;
        display_type_literal_inside(f, self)?;
        write!(f, "]")
    }
}

impl From<LiteralExpressionRef<'_>> for TypeLiteral {
    fn from(value: LiteralExpressionRef) -> Self {
        match value {
            LiteralExpressionRef::StringLiteral(s) => {
                TypeLiteral::StringLiteral(s.value.to_str().to_owned())
            }
            LiteralExpressionRef::BytesLiteral(b) => TypeLiteral::BytesLiteral(
                b.value
                    .iter()
                    .flat_map(|part| part.as_slice().iter().copied())
                    .collect::<Vec<_>>(),
            ),
            LiteralExpressionRef::NumberLiteral(n) => match n.value.clone() {
                Number::Int(i) => TypeLiteral::IntLiteral(i.as_i64().unwrap()),
                Number::Float(f) => TypeLiteral::FloatLiteral(f.to_string()),
                Number::Complex { real: _, imag: _ } => unimplemented!(),
            },
            LiteralExpressionRef::BooleanLiteral(b) => TypeLiteral::BooleanLiteral(b.value),
            LiteralExpressionRef::NoneLiteral(_) => TypeLiteral::NoneLiteral,
            LiteralExpressionRef::EllipsisLiteral(_) => TypeLiteral::EllipsisLiteral,
        }
    }
}
