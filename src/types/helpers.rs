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

use super::{Type, TypeLiteral};

/// Check if a is a subtype of b, A is a subtype of b if a can do everything b can.
pub fn is_subtype(a: &Type, b: &Type) -> bool {
    if a == b {
        return true;
    }

    if let Type::Literal(literal) = a {
        return match literal {
            TypeLiteral::StringLiteral(_) => is_subtype(&Type::String, b),
            TypeLiteral::BytesLiteral(_) => unimplemented!(),
            TypeLiteral::IntLiteral(_) => is_subtype(&Type::Int, b),
            TypeLiteral::FloatLiteral(_) => is_subtype(&Type::Float, b),
            TypeLiteral::BooleanLiteral(_) => is_subtype(&Type::Bool, b),
            TypeLiteral::NoneLiteral => is_subtype(&Type::None, b),
            TypeLiteral::EllipsisLiteral => is_subtype(&Type::Ellipsis, b),
        };
    }

    match (a, b) {
        (Type::Any | Type::Unknown, _) => true,
        (_, Type::Any | Type::Unknown) => true,
        (Type::Int, Type::Float) => true,
        (Type::Never, _) => false,
        (Type::Union(union), b) => union.iter().all(|a| is_subtype(a, b)),
        (a, Type::Union(union)) => union.iter().any(|b| is_subtype(a, b)),
        (Type::Function(f1), Type::Function(f2)) => {
            f1.args.len() == f2.args.len()
                && f1
                    .args
                    .iter()
                    .enumerate()
                    .all(|(i, t1)| is_subtype(&f2.args[i], t1))
                && is_subtype(&f1.ret, &f2.ret)
        }
        (Type::Tuple(t1), Type::Tuple(t2)) => {
            if t1.len() == t2.len() {
                t1.iter().zip(t2.iter()).all(|(t1, t2)| is_subtype(t1, t2))
            } else {
                false
            }
        }
        _ => false,
    }
}

fn collapse_subtypes(types: Vec<Type>) -> Vec<Type> {
    let mut keep = vec![false; types.len()];
    for (i1, t1) in types.iter().enumerate() {
        keep[i1] = types.iter().enumerate().all(
            //                                      an arm is kept if
            |(i2, t2)|                           // for every arm
                i1 == i2 ||                      // (except itself)
                !is_subtype(t1, t2) ||           // it's not a subtype of the other arm
                (is_subtype(t2, t1) && i1 < i2), // or it's equivalent to the other arm
                                                 // and this is the first equivalent arm
        )
    }

    types
        .into_iter()
        .zip(keep)
        .filter_map(|(t, keep)| if keep { Some(t) } else { None })
        .collect()
}
fn flatten(types: Vec<Type>) -> Vec<Type> {
    let mut flattened: Vec<Type> = Vec::with_capacity(types.len());
    for typ in types.into_iter() {
        match typ {
            Type::Union(types) => flattened.extend(types.into_iter()),
            other => flattened.push(other),
        };
    }
    flattened
}
fn collapse_union_types(mut types: Vec<Type>) -> Vec<Type> {
    types = flatten(types);
    types = collapse_subtypes(types);
    types
}
pub fn union(mut types: Vec<Type>) -> Type {
    types = collapse_union_types(types);

    if types.len() == 0 {
        Type::Never
    } else if types.len() == 1 {
        types.pop().unwrap()
    } else {
        Type::Union(types)
    }
}
