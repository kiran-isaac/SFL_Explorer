use crate::types::checker::type_error;
use crate::{ASTNodeType, KnownTypeLabelTable, Type, TypeError, AST};
use std::collections::HashSet;

#[derive(Clone, PartialEq, Eq)]
pub enum ContextItem {
    TypeVariable(String),
    TypeAssignment(String, Result<Type, TypeError>),
    Existential(usize, Option<Type>),
    Marker(usize),
}

impl std::fmt::Debug for ContextItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContextItem::TypeVariable(v) => write!(f, "{}", v),
            ContextItem::Existential(v, ass) => match ass {
                Some(t) => write!(f, "{}:{}", Type::Existential(*v).to_string(), t.to_string()),
                None => write!(f, "{}", Type::Existential(*v).to_string()),
            },
            ContextItem::Marker(v) => write!(f, "|{}|", Type::Existential(*v).to_string()),
            ContextItem::TypeAssignment(name, tr) => match tr {
                Ok(t) => write!(f, "{}:{}", name, t.to_string()),
                Err(_) => write!(f, "{}:ERROR", name),
            },
        }
    }
}

#[derive(Clone)]
pub struct Context {
    vec: Vec<ContextItem>,
    next_exid: usize,
    next_placeholder_assignvar_i: usize,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inbuilt_map = KnownTypeLabelTable::new().func_map;
        write!(f, "[")?;
        for item in &self.vec {
            match item {
                ContextItem::TypeAssignment(name, _) => {
                    if !inbuilt_map.contains_key(name) {
                        write!(f, "{:?}, ", item)?;
                    }
                }
                _ => write!(f, "{:?}, ", item)?,
            }
        }
        write!(f, "]")
    }
}

impl Context {
    pub fn from_labels(labels: &KnownTypeLabelTable, yet_to_bind: &HashSet<String>) -> Self {
        let mut vec = vec![];

        for (k, v) in labels.get_type_map() {
            if !yet_to_bind.contains(&k) {
                vec.push(ContextItem::TypeAssignment(
                    k.clone(),
                    Ok(v.clone().unwrap()),
                ));
            }
        }

        Self {
            vec,
            next_exid: 0,
            next_placeholder_assignvar_i: 0,
        }
    }

    pub fn assigns_only(&self) -> Self {
        let mut vec = vec![];

        for i in self.vec.iter() {
            if let ContextItem::TypeAssignment(_, _) = i {
                vec.push(i.clone());
            }
        }

        Self {
            vec,
            next_exid: 0,
            next_placeholder_assignvar_i: 0,
        }
    }

    pub fn append(&self, item: ContextItem) -> Self {
        let mut new = self.clone();

        match &item {
            ContextItem::TypeAssignment(name, _) => {
                if name.starts_with("_") {
                    new.next_placeholder_assignvar_i += 1;
                }
            }
            ContextItem::TypeVariable(name) => {
                if name.starts_with("_") {
                    new.next_placeholder_assignvar_i += 1;
                }
            }
            ContextItem::Existential(e, _) => {
                new.next_exid = std::cmp::max(*e, new.next_exid);
            }
            _ => {}
        }

        new.vec.push(item);
        new
    }

    pub fn get_before_item(&self, item: ContextItem) -> Self {
        let mut new_v = vec![];

        for i in &self.vec {
            if i == &item {
                break;
            }

            new_v.push(i.clone());
        }

        Self {
            vec: new_v,
            next_exid: self.next_exid,
            next_placeholder_assignvar_i: self.next_placeholder_assignvar_i,
        }
    }
    pub fn get_before_assignment(&self, str: String) -> Self {
        #[cfg(debug_assertions)]
        let _c_str = format!("{:?}", &self);
        let mut new_v = vec![];

        for i in &self.vec {
            match i {
                ContextItem::TypeAssignment(v, _) => {
                    if v == &str {
                        break;
                    }
                }
                _ => {}
            }

            new_v.push(i.clone());
        }

        let new_s = Self {
            vec: new_v,
            next_exid: self.next_exid,
            next_placeholder_assignvar_i: self.next_placeholder_assignvar_i,
        };

        #[cfg(debug_assertions)]
        let _new_s_str = format!("{:?}", &new_s);
        new_s
    }

    pub fn get_type_assignment(&self, var: &str) -> Option<Result<Type, TypeError>> {
        for i in &self.vec {
            match i {
                ContextItem::TypeAssignment(v, t) => {
                    if v == var {
                        return Some(t.clone());
                    }
                }
                _ => {}
            }
        }

        None
    }

    pub fn add_before_existential(&self, existential: usize, item: ContextItem) -> Self {
        let mut new_v = vec![];
        let mut next_placeholder_assignvar_i = self.next_placeholder_assignvar_i;
        let mut next_exid = self.next_exid;

        match item {
            ContextItem::Existential(e2, _) => {
                next_exid = std::cmp::max(e2, next_exid);
            }
            _ => {}
        };

        for i in &self.vec {
            match i {
                ContextItem::TypeAssignment(name, _) => {
                    if name.starts_with("_") {
                        next_placeholder_assignvar_i += 1;
                    }
                }
                ContextItem::Existential(e, _) => {
                    if *e == existential {
                        new_v.push(item.clone());
                    }
                    next_exid = std::cmp::max(*e, next_exid);
                }
                _ => {}
            }
            new_v.push(i.clone());
        }

        Self {
            vec: new_v,
            next_exid,
            next_placeholder_assignvar_i,
        }
    }

    pub fn set_existential_definition(&self, existential_being_set: usize, t: Type) -> Self {
        #[cfg(debug_assertions)]
        let _c_str = format!("{:?}", &self);
        #[cfg(debug_assertions)]
        let _t_str = format!("{:?}", &t);

        let mut new_v = vec![];

        for i in &self.vec {
            match i {
                // If this is another existential that references the one being substituted then
                // Substitute this one too
                ContextItem::Existential(e, Some(Type::Existential(e2))) => {
                    if *e2 == existential_being_set {
                        new_v.push(ContextItem::Existential(*e, Some(t.clone())));
                        continue;
                    }
                }
                // If this is an existential with a product that refers to the one being set, set the
                // types within the product too
                // ContextItem::Existential(e, Some(Type::Product(t1, t2))) => {
                //     let mut t1 = t1.as_ref().clone();
                //     if let Type::Existential(e1) = t1 {
                //         if e1 == existential_being_set {
                //             t1 = t.clone();
                //         }
                //     }
                //
                //     let mut t2 = t2.as_ref().clone();
                //     if let Type::Existential(e2) = t2 {
                //         if e2 == existential_being_set {
                //             t2 = t.clone();
                //         }
                //     }
                //     new_v.push(ContextItem::Existential(*e, Some(t.clone())));
                //     continue;
                // }
                ContextItem::Existential(e, _) => {
                    if *e == existential_being_set {
                        new_v.push(ContextItem::Existential(*e, Some(t.clone())));
                        continue;
                    }
                }
                _ => {}
            }
            new_v.push(i.clone());
        }

        #[cfg(debug_assertions)]
        let _new_v_str = format!("{:?}", new_v);

        Self {
            vec: new_v,
            next_exid: self.next_exid,
            next_placeholder_assignvar_i: self.next_placeholder_assignvar_i,
        }
    }

    pub fn get_next_existential_identifier(&self) -> usize {
        self.next_exid + 1
    }

    pub fn get_next_placeholder_assignvar(&self) -> String {
        "_".to_string() + &self.next_placeholder_assignvar_i.to_string()
    }

    // Stupid name
    pub fn get_existential(&self, ex: usize) -> Option<Option<Type>> {
        for i in &self.vec {
            #[cfg(debug_assertions)]
            let _i_str = format!("{:?}", i);
            match i {
                ContextItem::Existential(ex2, o) => {
                    if ex == *ex2 {
                        return Some(o.clone());
                    }
                }
                _ => {}
            }
        }

        None
    }

    pub fn substitute(&self, t: &Type) -> Type {
        #[cfg(debug_assertions)]
        let _c_str = format!("{:?}", self);
        #[cfg(debug_assertions)]
        let _t_str = format!("{}", t.to_string());

        match t {
            Type::Existential(ex) => match self.get_existential(*ex) {
                Some(o) => match o {
                    Some(t2) => self.substitute(&t2.clone()),
                    None => t.clone(),
                },
                None => t.clone(),
            },
            Type::Function(from, to) => Type::Function(
                Box::new(self.substitute(from.as_ref())),
                Box::new(self.substitute(to.as_ref())),
            ),
            Type::Product(t1, t2) => {
                Type::Product(Box::new(self.substitute(t1)), Box::new(self.substitute(t2)))
            }
            Type::Forall(var, t) => {
                Type::Forall(var.clone(), Box::new(self.substitute(t.as_ref())))
            }
            Type::Union(name, types) => {
                let mut new_types = vec![];
                for t in types {
                    new_types.push(self.substitute(t));
                }
                Type::Union(name.clone(), new_types)
            }
            _ => t.clone(),
        }
    }

    pub fn recurse_add_to_context(
        &self,
        expected: &Type,
        ast: &AST,
        expr: usize,
    ) -> Result<(Context, String), TypeError> {
        let pn = ast.get(expr);
        #[cfg(debug_assertions)]
        let _c_str = format!("{:?}", &self);
        #[cfg(debug_assertions)]
        let _expr_str = format!("{}", ast.to_string_sugar(expr, false));
        #[cfg(debug_assertions)]
        let _expected_str = format!("{}", &expected.to_string());

        match (expected, &pn.t) {
            (Type::Alias(_, t), _) => self.recurse_add_to_context(t, ast, expr),
            (_, ASTNodeType::Identifier) => {
                let mut var_name = ast.get(expr).get_value();
                if self.get_type_assignment(var_name.as_str()).is_some() {
                    return Err(type_error(
                        format!(
                            "Type of {} is defined elsewhere, so cannot rebind",
                            var_name
                        ),
                        ast,
                        expr,
                    ));
                }
                if var_name.starts_with("_") {
                    var_name = self.get_next_placeholder_assignvar();
                }
                let new_ass = ContextItem::TypeAssignment(var_name.clone(), Ok(expected.clone()));
                Ok((self.append(new_ass.clone()), var_name))
            }
            (Type::Product(pt1, pt2), ASTNodeType::Pair) => {
                let pv1 = ast.get_first(expr);
                let pv2 = ast.get_second(expr);
                let (c, before) = self.recurse_add_to_context(pt1, &ast, pv1)?;
                let (c, _) = c.recurse_add_to_context(pt2, &ast, pv2)?;
                Ok((c, before))
            }
            (Type::Existential(e), ASTNodeType::Pair) => {
                let pv1 = ast.get_first(expr);
                let pv2 = ast.get_second(expr);
                let pt1 = self.get_next_existential_identifier();
                let pt2 = self.get_next_existential_identifier() + 1;

                let c = self.add_before_existential(*e, ContextItem::Existential(pt2, None));
                let c = c.add_before_existential(*e, ContextItem::Existential(pt1, None));

                #[cfg(debug_assertions)]
                let _c_str = format!("{:?}", &c);

                let (c, before) = c.recurse_add_to_context(&Type::Existential(pt1), &ast, pv1)?;
                let (c, _) = c.recurse_add_to_context(&Type::Existential(pt2), &ast, pv2)?;

                #[cfg(debug_assertions)]
                let _c_str2 = format!("{:?}", &c);

                let c = c.set_existential_definition(
                    *e,
                    Type::pr(Type::Existential(pt1), Type::Existential(pt2)),
                );
                Ok((c, before))
            }
            _ => Err(type_error("recurse add issue".to_string(), ast, expr)),
        }
    }
}
