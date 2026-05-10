use super::checking::check_type;
use super::*;
use crate::parsing::TypeMap;
use crate::*;

// "Γ ⊢ e ⇒ A ⊣ ∆: Under input context Γ, e synthesizes output type A, with output context ∆"
pub(crate) fn synthesize_type(
    c: Context,
    ast: &AST,
    expr: usize,
    type_map: &TypeMap,
    is_pattern: bool,
) -> Result<(Type, Context), TypeError> {
    #[cfg(debug_assertions)]
    let _expr_str = ast.to_string_sugar(expr, false);

    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);

    let node = ast.get(expr);

    match node.t {
        // Var
        ASTNodeType::Identifier => {
            let var = node.get_value();

            #[cfg(debug_assertions)]
            let _var_str = var.clone();
            match c.get_type_assignment(&var) {
                Some(t) => Ok((t?, c)),
                None => {
                    if is_pattern && var.chars().next().unwrap().is_lowercase() {
                        let next_exist = Type::Existential(c.get_next_existential_identifier());
                        Ok((
                            next_exist.clone(),
                            c.append(ContextItem::Existential(
                                c.get_next_existential_identifier(),
                                None,
                            ))
                            .append(ContextItem::TypeAssignment(var.clone(), Ok(next_exist))),
                        ))
                    } else {
                        if var == "_" {
                            let next_exist = Type::Existential(c.get_next_existential_identifier());
                            return Ok((
                                next_exist.clone(),
                                c.append(ContextItem::Existential(
                                    c.get_next_existential_identifier(),
                                    None,
                                )),
                            ));
                        }
                        panic!("Unbound identifier not in pattern: {}", &var)
                    }
                }
            }
        }

        ASTNodeType::Pair => {
            let expr1 = ast.get_first(expr);
            let expr2 = ast.get_second(expr);
            let (expr1t, c) = synthesize_type(c, ast, expr1, type_map, is_pattern)?;
            let (expr2t, c) = synthesize_type(c, ast, expr2, type_map, is_pattern)?;

            // lift foralls
            let expr1fas = expr1t.get_foralls();
            let expr2fas = expr2t.get_foralls();

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            Ok((
                Type::fa(
                    expr1fas,
                    Type::fa(
                        expr2fas,
                        Type::pr(expr1t.strip_foralls(), expr2t.strip_foralls()),
                    ),
                ),
                c,
            ))
        }

        ASTNodeType::Literal => Ok((node.get_lit_type(), c)),

        ASTNodeType::Match => {
            assert_eq!(is_pattern, false);

            let unpack_expr = ast.get_match_unpack_pattern(expr);

            let (unpack_type, c) = if let Some(t) = ast.get(unpack_expr).type_assignment.clone() {
                let c = check_type(c, &t, ast, unpack_expr, type_map, false)?;
                (t.clone(), c)
            } else {
                synthesize_type(c, ast, unpack_expr, type_map, false)?
            };

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);
            let _unpack_type_str = format!("{}", &unpack_type);

            let cases = ast.get_match_cases(expr);

            let expr_exist = c.get_next_existential_identifier();
            let expr_type = Type::Existential(expr_exist);

            let mut c = c.append(ContextItem::Existential(expr_exist, None));

            for (case_pat, case_expr) in cases {
                #[cfg(debug_assertions)]
                let _pat_str = format!("{}", &ast.to_string_sugar(case_pat, false));
                #[cfg(debug_assertions)]
                let _expr_str = format!("{}", &ast.to_string_sugar(case_expr, false));

                let pattern_context = check_type(c, &unpack_type, ast, case_pat, type_map, true)?;
                #[cfg(debug_assertions)]
                let _pat_c_str = format!("{:?}", &pattern_context);

                let expr_context =
                    check_type(pattern_context, &expr_type, ast, case_expr, type_map, false)?;
                #[cfg(debug_assertions)]
                let _expr_c_str = format!("{:?}", &expr_context);

                c = expr_context;
            }

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            c.get_existential(expr_exist)
                .map(|t| (t.unwrap(), c))
                .ok_or_else(|| type_error("Match failed".to_string(), ast, expr))
        }

        // ->I=>
        ASTNodeType::Abstraction => {
            let next_exst = c.get_next_existential_identifier();
            let c = c
                .append(ContextItem::Existential(next_exst, None))
                .append(ContextItem::Existential(next_exst + 1, None));

            let c = if let Some(t) = &ast.get(ast.get_abstr_var(expr)).type_assignment {
                c.set_existential_definition(next_exst, t.clone())
            } else {
                c
            };

            let (c, before) = c.recurse_add_to_context(
                &Type::Existential(next_exst),
                ast,
                ast.get_abstr_var(expr),
            )?;

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let c = check_type(
                c,
                &Type::Existential(next_exst + 1),
                ast,
                ast.get_abstr_expr(expr),
                type_map,
                false,
            )?;

            #[cfg(debug_assertions)]
            let _c_str2 = format!("{:?}", &c);
            #[cfg(debug_assertions)]
            let _x = 1 + 1;

            let abst_type = Type::f(
                Type::Existential(next_exst),
                Type::Existential(next_exst + 1),
            );
            let c = c.get_before_assignment(before);

            #[cfg(debug_assertions)]
            let _c_str3 = format!("{:?}", &c);

            #[cfg(debug_assertions)]
            let _abst_type_str = format!("{}", &abst_type.to_string());

            Ok((abst_type, c))
        }

        // ->E
        ASTNodeType::Application => {
            let lhs = ast.get_func(expr);
            let rhs = ast.get_arg(expr);

            let (f_type, f_c) = synthesize_type(c, ast, lhs, type_map, is_pattern)?;

            #[cfg(debug_assertions)]
            let _f_c_str = format!("{:?}", &f_c);

            #[cfg(debug_assertions)]
            let _f_type_str = f_type.to_string();

            let f_type = f_c.substitute(&f_type);

            #[cfg(debug_assertions)]
            let _f_type_str = f_type.to_string();
            synthesize_app_type(f_c, &f_type, ast, lhs, rhs, type_map, is_pattern)
        }

        _ => unreachable!("Non expression"),
    }
}

// "Γ ⊢ A • e ⇒⇒ C ⊣ ∆: Under input context Γ, applying a function of type A to e synthesizes type C, with output context ∆"
fn synthesize_app_type(
    c: Context,
    applied_type: &Type,
    ast: &AST,
    f: usize,
    expr: usize,
    type_map: &TypeMap,
    is_pattern: bool,
) -> Result<(Type, Context), TypeError> {
    #[cfg(debug_assertions)]
    let _expr_str = ast.to_string_sugar(expr, false);

    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);

    #[cfg(debug_assertions)]
    let _applied_type = applied_type.to_string();

    match applied_type {
        // Forall App
        Type::Forall(var, t) => {
            let new_c = c.append(ContextItem::Existential(
                c.get_next_existential_identifier(),
                None,
            ));

            #[cfg(debug_assertions)]
            let _new_c_str = format!("{:?}", &new_c);

            let a_subst = match t.substitute_type_variable(
                &var.clone(),
                &Type::Existential(c.get_next_existential_identifier()),
            ) {
                Ok(t) => t,
                Err(s) => {
                    panic!("Failed to substitute in forall app: {}", s)
                }
            };
            synthesize_app_type(new_c, &a_subst, ast, f, expr, type_map, is_pattern)
        }

        // -> App
        Type::Function(from, to) => {
            let pred = check_type(c, &from, ast, expr, type_map, is_pattern)?;

            Ok((to.as_ref().clone(), pred))
        }

        Type::Existential(var) => {
            let a1n = c.get_next_existential_identifier();
            let a2n = c.get_next_existential_identifier() + 1;
            let a1 = ContextItem::Existential(a1n, None);
            let a2 = ContextItem::Existential(a2n, None);
            let c = c
                .add_before_existential(*var, a2)
                .add_before_existential(*var, a1);
            let a1t = Type::Existential(a1n);
            let a2t = Type::Existential(a2n);
            let c = c.set_existential_definition(*var, Type::f(a1t.clone(), a2t.clone()));

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            let c = check_type(c, &a1t, ast, expr, type_map, is_pattern)?;

            Ok((a2t.clone(), c))
        }

        Type::Product(_, _) | Type::Union(_, _) | Type::Primitive(_) => Err(type_error(
            format!(
                "Cannot apply {} (of type {}) to {}",
                ast.to_string_sugar(f, false),
                applied_type.tv_ify(),
                ast.to_string_sugar(expr, false)
            ),
            ast,
            expr,
        )),

        _ => Err(type_error(format!("App synthesis error. Failed to understand the application of type {} to expression {}", applied_type, ast.to_string_sugar(expr, false)), ast, expr)),
    }
}
