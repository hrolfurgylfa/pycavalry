use super::Type;

pub fn is_subtype(a: &Type, b: &Type) -> bool {
    if a == b {
        return true;
    }

    match (a, b) {
        (Type::Any | Type::Unknown, _) => true,
        (_, Type::Any | Type::Unknown) => true,
        (Type::Int, Type::Float) => true,
        (Type::Function(f1), Type::Function(f2)) => {
            f1.args.len() == f2.args.len()
                && f1
                    .args
                    .iter()
                    .enumerate()
                    .all(|(i, t1)| is_subtype(&f2.args[i], t1))
                && is_subtype(&f1.ret, &f2.ret)
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
pub fn union(mut types: Vec<Type>) -> Type {
    types = flatten(types);
    types = collapse_subtypes(types);

    if types.len() == 0 {
        Type::Never
    } else if types.len() == 1 {
        types.pop().unwrap()
    } else {
        Type::Union(types)
    }
}
