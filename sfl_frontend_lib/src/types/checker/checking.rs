use super::*;
use crate::parsing::TypeMap;
use crate::{ASTNodeType, Type, AST};

// "Γ ⊢ e ⇐ A ⊣ ∆: Under input context Γ, e checks against input type A, with output context ∆"
pub fn check_type(
    c: Context,
    expected: &Type,
    ast: &AST,
    expr: usize,
    type_map: &TypeMap,
    is_pattern: bool,
) -> Result<Context, TypeError> {
    let node = ast.get(expr);

    #[cfg(debug_assertions)]
    let _expr_str = ast.to_string_sugar(expr, false);

    #[cfg(debug_assertions)]
    let _c_str = format!("{:?}", &c);

    #[cfg(debug_assertions)]
    let _expected_type_str = expected.to_string();

    match (expected, &node.t) {
        // Unit always checks
        (Type::Unit, _) => Ok(c),

        // Follow Alias
        (Type::Alias(_, type_), _) => check_type(c, &type_, ast, expr, type_map, is_pattern),

        // Forall Introduction
        (Type::Forall(var, t), _) => {
            let t = match t
                .as_ref()
                .clone()
                .substitute_type_variable(var, &Type::TypeVariable(var.clone()))
            {
                Ok(t) => t,
                Err(e) => panic!("{}", e),
            };

            let c = c.append(ContextItem::TypeVariable(var.clone()));

            let pred = check_type(c, &t, ast, expr, type_map, is_pattern)?;
            Ok(pred.get_before_item(ContextItem::TypeVariable(var.clone())))
        }

        // Arrow introduction
        (Type::Function(from, to), ASTNodeType::Abstraction) => {
            let var = ast.get_abstr_var(expr);

            let (c, before) = c.recurse_add_to_context(from, &ast, var)?;

            let pred = check_type(c, to, ast, ast.get_abstr_expr(expr), type_map, is_pattern)?;
            Ok(pred.get_before_assignment(before))
        }

        (Type::Product(pt1, pt2), ASTNodeType::Pair) => {
            let pair1 = check_type(c, pt1, ast, ast.get_first(expr), type_map, is_pattern)?;
            let pair2 = check_type(pair1, pt2, ast, ast.get_second(expr), type_map, is_pattern)?;

            Ok(pair2)
        }

        // Sub
        _ => {
            let (synth_t, c) = synthesize_type(c, ast, expr, type_map, is_pattern)?;

            #[cfg(debug_assertions)]
            let _c_str = format!("{:?}", &c);

            #[cfg(debug_assertions)]
            let _synth_t_str = synth_t.to_string();

            let a = c.substitute(&synth_t);
            let b = c.substitute(&expected);

            let st = subtype(c, &a, &b, type_map);

            match st {
                Ok(new_c) => {
                    #[cfg(debug_assertions)]
                    let _c_str = format!("{:?}", &new_c);

                    Ok(new_c)
                }
                Err(e) => Err(type_error(
                    format!(
                        "Cannot figure out how {} could be subtype of {}: {}",
                        a.tv_ify().to_string(),
                        b.tv_ify().to_string(),
                        e
                    ),
                    ast,
                    expr,
                )),
            }
        }
    }
}
