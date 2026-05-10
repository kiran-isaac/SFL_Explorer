use std::collections::HashMap;

use inbuilt_arith::*;

use crate::*;
mod inbuilt_arith;

#[cfg(test)]
mod test;

fn assert_prim_type(x: &Type, p: Primitive) {
    match x {
        Type::Primitive(prim) => {
            if *prim != p {
                panic!("ASSERT_PRIM_TYPE failed: Invalid type, wrong primitive")
            }
        }
        _ => panic!("ASSERT_PRIM_TYPE failed: Invalid type, not a primitive"),
    }
}

type InbuiltFuncPointer = fn(&ASTNode, Vec<&ASTNode>) -> AST;

#[derive(Clone, Debug)]
pub struct Label {
    /// Arity needed to reduce the function.
    pub inbuilt_reduction_arity: Option<usize>,

    inbuilt: Option<InbuiltFuncPointer>,
    pub label_type: Option<Type>,
}

impl Label {
    pub fn call_inbuilt(&self, call: &ASTNode, args: Vec<&ASTNode>) -> AST {
        assert_eq!(self.inbuilt_reduction_arity, Some(args.len()));
        assert!(self.inbuilt.is_some());
        self.inbuilt.unwrap()(call, args)
    }

    pub fn is_inbuilt(&self) -> bool {
        self.inbuilt.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct KnownTypeLabelTable {
    /// Sorted by arity. So inbuilts[0] will be all inbuilts with arity 0
    /// inbuilts[1] will be all inbuilts with arity 1, etc.
    pub func_map: HashMap<String, Label>,
}

impl KnownTypeLabelTable {
    pub fn new() -> Self {
        let mut s = Self {
            func_map: HashMap::new(),
        };
        s.populate_inbuilts();
        s
    }

    pub fn get_type_map(&self) -> HashMap<String, Option<Type>> {
        self.func_map
            .iter()
            .map(|(name, label)| (name.clone(), label.label_type.clone()))
            .collect()
    }

    pub fn get_type(&self, name: &str) -> Option<Option<Type>> {
        if let Some(label) = self.get(&name.to_string()) {
            Some(label.label_type.clone())
        } else {
            None
        }
    }

    /// Arity here is the number of arguments the inbuilt function needs to reduce
    /// It is not necessarily the same as the number of arguments the function takes
    /// as the function may be curried, for example 'if' takes one bool argument to
    /// reduce but it has a type of Bool -> A -> A -> A
    fn add_inbuilt(
        &mut self,
        name: String,
        arity: usize,
        func: InbuiltFuncPointer,
        func_type: Type,
    ) {
        self.func_map.insert(
            name,
            Label {
                inbuilt_reduction_arity: Some(arity),
                inbuilt: Some(func),
                label_type: Some(func_type),
            },
        );
    }

    pub fn add(&mut self, name: String, type_: Type) {
        self.func_map.insert(
            name,
            Label {
                inbuilt_reduction_arity: None,
                inbuilt: None,
                label_type: Some(type_),
            },
        );
    }

    pub fn add_no_type(&mut self, name: String) {
        self.func_map.insert(
            name,
            Label {
                inbuilt_reduction_arity: None,
                inbuilt: None,
                label_type: None,
            },
        );
    }

    pub fn consume_from_module(&mut self, ast: &AST, module: usize) -> Result<(), TypeError> {
        for (name, assign) in ast.get_assigns_map(module) {
            let ass_n = &ast.get(assign);
            let proclaimed_type = match &ass_n.type_assignment {
                None => {
                    return Err(TypeError {
                        e: format!("Label {} has no type assignment", name),
                        line: ast.get(assign).line,
                        col: ast.get(assign).col,
                    })
                }
                Some(t) => t.clone(),
            };

            self.add(name.clone(), proclaimed_type);
        }

        Ok(())
    }

    pub fn get(&self, name: &String) -> Option<&Label> {
        self.func_map.get(name)
    }

    fn populate_inbuilts(&mut self) {
        let binary_int_type = Type::Function(
            Box::new(Type::Primitive(Primitive::Int64)),
            Box::new(Type::Function(
                Box::new(Type::Primitive(Primitive::Int64)),
                Box::new(Type::Primitive(Primitive::Int64)),
            )),
        );
        let binary_int_bool_type = Type::Function(
            Box::new(Type::Primitive(Primitive::Int64)),
            Box::new(Type::Function(
                Box::new(Type::Primitive(Primitive::Int64)),
                Box::new(Type::Primitive(Primitive::Bool)),
            )),
        );
        let binary_float_type = Type::Function(
            Box::new(Type::Primitive(Primitive::Float64)),
            Box::new(Type::Function(
                Box::new(Type::Primitive(Primitive::Float64)),
                Box::new(Type::Primitive(Primitive::Float64)),
            )),
        );
        let binary_float_bool_type = Type::Function(
            Box::new(Type::Primitive(Primitive::Float64)),
            Box::new(Type::Function(
                Box::new(Type::Primitive(Primitive::Float64)),
                Box::new(Type::Primitive(Primitive::Bool)),
            )),
        );
        let unary_int_type = Type::Function(
            Box::new(Type::Primitive(Primitive::Int64)),
            Box::new(Type::Primitive(Primitive::Int64)),
        );
        let unary_float_type = Type::Function(
            Box::new(Type::Primitive(Primitive::Float64)),
            Box::new(Type::Primitive(Primitive::Float64)),
        );

        self.add_inbuilt(
            "add".to_string(),
            2,
            inbuilt_int_add,
            binary_int_type.clone(),
        );
        self.add_inbuilt("+".to_string(), 2, inbuilt_int_add, binary_int_type.clone());
        self.add_inbuilt(
            "sub".to_string(),
            2,
            inbuilt_int_sub,
            binary_int_type.clone(),
        );
        self.add_inbuilt("-".to_string(), 2, inbuilt_int_sub, binary_int_type.clone());
        self.add_inbuilt(
            "mul".to_string(),
            2,
            inbuilt_int_mul,
            binary_int_type.clone(),
        );
        self.add_inbuilt("*".to_string(), 2, inbuilt_int_mul, binary_int_type.clone());
        self.add_inbuilt(
            "div".to_string(),
            2,
            inbuilt_int_div,
            binary_int_type.clone(),
        );
        self.add_inbuilt("/".to_string(), 2, inbuilt_int_div, binary_int_type.clone());
        self.add_inbuilt(
            "mod".to_string(),
            2,
            inbuilt_int_mod,
            binary_int_type.clone(),
        );
        self.add_inbuilt("%".to_string(), 2, inbuilt_int_mod, binary_int_type.clone());

        self.add_inbuilt(
            "eq".to_string(),
            2,
            inbuilt_int_eq,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "==".to_string(),
            2,
            inbuilt_int_eq,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "lte".to_string(),
            2,
            inbuilt_int_lte,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "<=".to_string(),
            2,
            inbuilt_int_lte,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "lt".to_string(),
            2,
            inbuilt_int_lt,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "<".to_string(),
            2,
            inbuilt_int_lt,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "gte".to_string(),
            2,
            inbuilt_int_gte,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            ">=".to_string(),
            2,
            inbuilt_int_gte,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(
            "gt".to_string(),
            2,
            inbuilt_int_gt,
            binary_int_bool_type.clone(),
        );
        self.add_inbuilt(">".to_string(), 2, inbuilt_int_gt, binary_int_bool_type);

        self.add_inbuilt(
            "addf".to_string(),
            2,
            inbuilt_float_add,
            binary_float_type.clone(),
        );
        self.add_inbuilt(
            "subf".to_string(),
            2,
            inbuilt_float_sub,
            binary_float_type.clone(),
        );
        self.add_inbuilt(
            "mulf".to_string(),
            2,
            inbuilt_float_mul,
            binary_float_type.clone(),
        );
        self.add_inbuilt("divf".to_string(), 2, inbuilt_float_div, binary_float_type);
        self.add_inbuilt(
            "eqf".to_string(),
            2,
            inbuilt_float_eq,
            binary_float_bool_type.clone(),
        );
        self.add_inbuilt(
            "ltef".to_string(),
            2,
            inbuilt_float_lte,
            binary_float_bool_type.clone(),
        );
        self.add_inbuilt(
            "ltf".to_string(),
            2,
            inbuilt_float_lt,
            binary_float_bool_type.clone(),
        );
        self.add_inbuilt(
            "gtef".to_string(),
            2,
            inbuilt_float_gte,
            binary_float_bool_type.clone(),
        );
        self.add_inbuilt(
            "gtf".to_string(),
            2,
            inbuilt_float_gt,
            binary_float_bool_type,
        );

        // self.add_inbuilt("if".to_string(), 1, inbuilt_if, if_type);

        self.add_inbuilt("neg".to_string(), 1, inbuilt_int_neg, unary_int_type);
        self.add_inbuilt("negf".to_string(), 1, inbuilt_float_neg, unary_float_type);

        #[cfg(test)]
        self.add_inbuilt(
            "zero_ary_test".to_string(),
            0,
            inbuilt_int_zero,
            Type::int64(),
        );
    }

    /// Get all strings that are inbuilts so that they can be added to the bound checker
    pub fn get_starting_bindings_map() -> Vec<String> {
        let mut bindings = vec![];
        for inbuilt in &KnownTypeLabelTable::new().func_map {
            bindings.push(inbuilt.0.clone());
        }

        bindings
    }

    pub fn get_non_builtin_type_map(&self) -> HashMap<String, Option<Type>> {
        let builtin_type_map = Self::new().func_map;
        let mut type_map = HashMap::new();
        for (name, inbuilt) in &self.func_map {
            if builtin_type_map.contains_key(name) {
                continue;
            }
            type_map.insert(name.clone(), inbuilt.label_type.clone());
        }

        type_map
    }
}
