use core::fmt;
use ruff_python_ast::{LiteralExpressionRef, Number};

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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
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

    Literal(TypeLiteral),
    Function(Function),
    Class(Class),

    Union(Vec<Type>),
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
            Type::Literal(l) => write!(f, "{}", l),
            Type::Function(func) => write!(f, "{}", func),
            Type::Class(cls) => write!(f, "{}", cls),
            Type::Union(types) => {
                write!(f, "Union[")?;
                write_iter(f, types.iter(), |f, t| write!(f, "{}", t))?;
                write!(f, "]")
            }
        }?;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Function {
    pub args: Vec<Type>,
    pub arg_names: Vec<String>,
    pub ret: Box<Type>,
}

impl Function {
    pub fn new(args: Vec<Type>, arg_names: Vec<String>, ret: Box<Type>) -> Function {
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Class {
    pub name: String,
    pub functions: Vec<Function>,
    pub parameters: Vec<(String, Type)>,
}

impl Class {
    pub fn new(name: String, functions: Vec<Function>, parameters: Vec<(String, Type)>) -> Class {
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeClass {
    properties: Vec<TypeClassProperty>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeClassProperty {
    name: String,
    typ: Type,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeLiteral {
    StringLiteral(StringLiteral),
    BytesLiteral(BytesLiteral),
    NumberLiteral(NumberLiteral),
    BooleanLiteral(BooleanLiteral),
    NoneLiteral(NoneLiteral),
    EllipsisLiteral(EllipsisLiteral),
}

impl fmt::Display for TypeLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Literal[{}]", self)
    }
}

impl<'a> From<LiteralExpressionRef<'a>> for TypeLiteral {
    fn from(value: LiteralExpressionRef) -> Self {
        match value {
            LiteralExpressionRef::StringLiteral(s) => TypeLiteral::StringLiteral(StringLiteral {
                val: s.value.to_str().to_owned(),
            }),
            LiteralExpressionRef::BytesLiteral(b) => TypeLiteral::BytesLiteral(BytesLiteral {
                val: b
                    .value
                    .iter()
                    .flat_map(|part| part.as_slice().iter().copied())
                    .collect::<Vec<_>>(),
            }),
            LiteralExpressionRef::NumberLiteral(n) => {
                TypeLiteral::NumberLiteral(n.value.clone().into())
            }
            LiteralExpressionRef::BooleanLiteral(b) => {
                TypeLiteral::BooleanLiteral(BooleanLiteral { val: b.value })
            }
            LiteralExpressionRef::NoneLiteral(_) => TypeLiteral::NoneLiteral(NoneLiteral {}),
            LiteralExpressionRef::EllipsisLiteral(_) => {
                TypeLiteral::EllipsisLiteral(EllipsisLiteral {})
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StringLiteral {
    val: String,
}

impl fmt::Display for StringLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.val)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BytesLiteral {
    val: Vec<u8>,
}

impl fmt::Display for BytesLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "b\"{:?}\"", self.val)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NumberLiteral {
    val: String,
}

impl fmt::Display for NumberLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

impl From<Number> for NumberLiteral {
    fn from(value: Number) -> Self {
        let v = match value {
            Number::Int(i) => i.to_string(),
            Number::Float(f) => f.to_string(),
            Number::Complex { real, imag } => format!("{}+{}j", real, imag),
        };
        NumberLiteral { val: v }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BooleanLiteral {
    val: bool,
}

impl fmt::Display for BooleanLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", if self.val { "True" } else { "False" })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NoneLiteral {}

impl fmt::Display for NoneLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "None")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EllipsisLiteral {}

impl fmt::Display for EllipsisLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "...")
    }
}
