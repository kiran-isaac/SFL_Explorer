use super::*;
use crate::parsing::TypeMap;
use crate::Type;
use std::iter::zip;

/// A is subtype of B
pub fn subtype(c: Context, a: &Type, b: &Type, type_map: &TypeMap) -> Result<Context, String> {
    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);
    #[cfg(debug_assertions)]
    let _a_str = a.to_string();
    #[cfg(debug_assertions)]
    let _b_str = b.to_string();

    match (a, b) {
        (Type::Alias(_, a_type), _) => subtype(c, a_type, b, type_map),
        (_, Type::Alias(_, a_type)) => subtype(c, a, a_type, type_map),

        // <:InstantiateL
        (Type::Existential(ex), _) => {
            match b {
                Type::Existential(ex2) => {
                    if ex == ex2 {
                        return Ok(c);
                    }
                }
                _ => {}
            }

            if b.contains_existential(*ex) {
                let a = c.substitute(a).tv_ify();
                let b = c.substitute(b).tv_ify();
                return Err(format!(
                    "Cannot instantiate {} to the type {}: the type contains itself",
                    b, a
                ));
            }

            instantiate_l(c, *ex, b, type_map)
        }

        // <:InstantiateR
        (_, Type::Existential(ex)) => {
            if a.contains_existential(*ex) {
                let a = c.substitute(a).tv_ify();
                let b = c.substitute(b).tv_ify();
                return Err(format!(
                    "Cannot instantiate {} to the type {}: the type contains itself",
                    b, a
                ));
            }

            instantiate_r(c, *ex, a, type_map)
        }

        // <:Var
        (Type::TypeVariable(a), Type::TypeVariable(b)) => {
            if a == b {
                Ok(c)
            } else {
                Err(format!("{} is not a subtype of {}", a, b))
            }
        }

        (Type::Primitive(a), Type::Primitive(b)) => {
            if a == b {
                Ok(c)
            } else {
                Err(format!("{} is not a subtype of {}", a, b))
            }
        }

        // <:Unit
        (Type::Unit, Type::Unit) => Ok(c),

        // <:ForallL
        (Type::Forall(var, t), _) => {
            let exst = c.get_next_existential_identifier();
            let c = c
                .append(ContextItem::Marker(exst))
                .append(ContextItem::Existential(exst, None));

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let new_body = t.substitute_type_variable(var, &Type::Existential(exst))?;
            let pred = subtype(c, &new_body, b, type_map)?;
            Ok(pred.get_before_item(ContextItem::Marker(exst)))
        }

        // <:ForallR
        (_, Type::Forall(var, t)) => {
            let c = c.append(ContextItem::TypeVariable(var.clone()));
            let pred = subtype(c, a, t.as_ref(), type_map)?;
            Ok(pred.get_before_item(ContextItem::TypeVariable(var.clone())))
        }

        // <:->
        (Type::Function(a1, a2), Type::Function(b1, b2)) => {
            let pred1 = subtype(c, b1.as_ref(), a1, type_map)?;
            let a2 = &pred1.substitute(a2);
            let b2 = &pred1.substitute(b2);
            let pred2 = subtype(pred1, a2, b2, type_map)?;
            Ok(pred2)
        }

        (Type::Product(pt1_1, pt1_2), a) | (a, Type::Product(pt1_1, pt1_2)) => {
            let (ut2_1, ut2_2) = match a {
                Type::Product(a, b) => (a, b),
                _ => {
                    return Err(format!(
                        "Type {} is not a subtype of product {}",
                        a.tv_ify(),
                        Type::Product(pt1_1.clone(), pt1_2.clone()).tv_ify()
                    ))
                }
            };
            let ut_1_st = subtype(c, pt1_1, ut2_1, type_map)?;
            subtype(ut_1_st, pt1_2, ut2_2, type_map)
        }

        (Type::Union(name1, uargs1), Type::Union(name2, uargs2)) => {
            if uargs1.len() != uargs2.len() || name1 != name2 {
                return Err(format!(
                    "Type {} is not a subtype of union {}",
                    a.tv_ify(),
                    b.tv_ify()
                ));
            }
            let mut c = c;
            for (t1, t2) in zip(uargs1, uargs2) {
                c = subtype(c, t1, t2, type_map)?;
            }
            Ok(c)
        }
        _ => Err("Subtype failiure".to_string()),
    }
}

fn instantiate_l(c: Context, exst: usize, b: &Type, type_map: &TypeMap) -> Result<Context, String> {
    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);
    #[cfg(debug_assertions)]
    let _exst_str = format!("{}", Type::Existential(exst).to_string());
    #[cfg(debug_assertions)]
    let _b_str = format!("{}", &b.to_string());
    match b {
        // InstLReach
        Type::Existential(exst2) => {
            Ok(c.set_existential_definition(*exst2, Type::Existential(exst)))
        }

        // InstLArr
        Type::Function(from, to) => {
            let a1n = c.get_next_existential_identifier();
            let a2n = c.get_next_existential_identifier() + 1;
            let a1 = ContextItem::Existential(a1n, None);
            let a2 = ContextItem::Existential(a2n, None);
            let c = c
                .add_before_existential(exst, a1)
                .add_before_existential(exst, a2)
                .set_existential_definition(
                    exst,
                    Type::f(Type::Existential(a1n), Type::Existential(a2n)),
                );

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let pred1_c = instantiate_r(c, a1n, from.as_ref(), type_map)?;
            let to_subst = pred1_c.substitute(to);

            #[cfg(debug_assertions)]
            let _to_subst_str = format!("{:?}", &to_subst);

            let pred2 = instantiate_l(pred1_c, a2n, &to_subst, type_map)?;

            Ok(pred2)
        }

        // InstLAllR
        Type::Forall(var, t) => {
            let new_c = c.append(ContextItem::TypeVariable(var.clone()));
            let pred = instantiate_l(new_c, exst, t.as_ref(), type_map)?;
            Ok(pred.get_before_item(ContextItem::TypeVariable(var.clone())))
        }

        _ => {
            if !b.is_monotype() {
                return Err("Failed Substitution".to_string());
            }

            if let Some(existential) = c.get_existential(exst) {
                if let Some(existential_type_assignment) = existential {
                    subtype(
                        c.clone(),
                        &existential_type_assignment,
                        &b.clone(),
                        type_map,
                    )?;
                }
            }

            // InstLSolve
            Ok(c.set_existential_definition(exst, b.clone()))
        }
    }
}

fn instantiate_r(c: Context, exst: usize, a: &Type, type_map: &TypeMap) -> Result<Context, String> {
    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);
    #[cfg(debug_assertions)]
    let _exst_str = format!("{}", Type::Existential(exst).to_string());
    #[cfg(debug_assertions)]
    let _a_str = format!("{}", &a.to_string());
    match a {
        // InstRReach
        Type::Existential(exst2) => {
            Ok(c.set_existential_definition(*exst2, Type::Existential(exst)))
        }

        // InstRArr
        Type::Function(from, to) => {
            #[cfg(debug_assertions)]
            let _from_str = format!("{}", &from.to_string());
            #[cfg(debug_assertions)]
            let _to_str = format!("{}", &to.to_string());

            let a1n = c.get_next_existential_identifier();
            let a2n = c.get_next_existential_identifier() + 1;
            let a1 = ContextItem::Existential(a1n, None);
            let a2 = ContextItem::Existential(a2n, None);
            #[cfg(debug_assertions)]
            let _c_str1 = format!("{:?}", &c);

            let c = c
                .add_before_existential(exst, a2)
                .add_before_existential(exst, a1)
                .set_existential_definition(
                    exst,
                    Type::f(Type::Existential(a1n), Type::Existential(a2n)),
                );

            #[cfg(debug_assertions)]
            let _c_str2 = format!("{:?}", &c);

            let pred1_c = instantiate_l(c, a1n, from.as_ref(), type_map)?;
            let to_subst = pred1_c.substitute(to.as_ref());

            #[cfg(debug_assertions)]
            let _to_subst_str = to_subst.to_string();

            let pred2 = instantiate_r(pred1_c, a2n, &to_subst, type_map)?;
            Ok(pred2)
        }

        // InstRAllL
        Type::Forall(var, t) => {
            let next_ext = c.get_next_existential_identifier();
            let c = c
                .append(ContextItem::Marker(next_ext))
                .append(ContextItem::Existential(next_ext, None));

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let t = t.substitute_type_variable(var, &Type::Existential(next_ext))?;
            let pred1 = instantiate_r(c, exst, &t, type_map)?;
            Ok(pred1.get_before_item(ContextItem::Marker(next_ext)))
        }

        _ => {
            if !a.is_monotype() {
                return Err("Failed Substitution".to_string());
            }

            if let Some(existential) = c.get_existential(exst) {
                if let Some(existential_type_assignment) = existential {
                    subtype(
                        c.clone(),
                        &existential_type_assignment,
                        &a.clone(),
                        type_map,
                    )?;
                }
            }

            // InstRSolve
            Ok(c.set_existential_definition(exst, a.clone()))
        }
    }
}
