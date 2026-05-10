use super::*;

impl Parser {
    fn parse_multiple_constructors(
        &mut self,
        type_table: &HashMap<String, Type>,
        params: &Vec<String>,
        union_type: &Type,
    ) -> Result<HashMap<String, Type>, ParserError> {
        let mut constructors = HashMap::new();
        let bound_type_vars: HashSet<String> = params.iter().cloned().collect();

        loop {
            let t = self.peek(0)?;
            match t.tt {
                TokenType::UppercaseId => {
                    let (constructor_name, constructor_params) =
                        self.parse_constructor(type_table, &bound_type_vars)?;

                    let mut constructor_type = union_type.clone();
                    for param in constructor_params.iter().rev() {
                        constructor_type = Type::f(param.clone(), constructor_type);
                    }

                    // forall-ify it
                    constructor_type = Type::fa(params.clone(), constructor_type);

                    #[cfg(debug_assertions)]
                    let _constructor_type_str = constructor_type.to_string();

                    constructors.insert(constructor_name, constructor_type);
                }
                TokenType::Bar => {
                    self.advance();
                }
                TokenType::Newline | TokenType::EOF => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(self.parse_error(format!(
                        "Unexpected token during data declaration: {}",
                        t.value
                    )))
                }
            }
        }

        Ok(constructors)
    }

    fn parse_constructor(
        &mut self,
        type_table: &HashMap<String, Type>,
        bound_type_vars: &HashSet<String>,
    ) -> Result<(String, Vec<Type>), ParserError> {
        let t = self.consume()?;
        if t.tt != TokenType::UppercaseId {
            return Err(self.parse_error(format!("Expected varient name, got {}", t.value)));
        }
        let constructor_name = t.value;

        let mut constructor_params = vec![];
        loop {
            let t = self.peek(0)?;
            match t.tt {
                TokenType::Id => {
                    self.advance();
                    if bound_type_vars.contains(&t.value) {
                        constructor_params.push(Type::TypeVariable(t.value));
                    } else {
                        return Err(
                            self.parse_error(format!("Unbound type parameter: {}", &t.value))
                        );
                    }
                }
                TokenType::UppercaseId => {
                    self.advance();
                    if let Some(type_) = type_table.get(&t.value) {
                        constructor_params.push(type_.clone());
                    } else {
                        return Err(
                            self.parse_error(format!("Unbound type parameter: {}", &t.value))
                        );
                    }
                }
                TokenType::LParen => {
                    self.advance();

                    let type_ = self.parse_type_expression(type_table, Some(bound_type_vars))?;
                    constructor_params.push(type_);
                    assert_eq!(self.consume()?.tt, TokenType::RParen);
                }

                _ => return Ok((constructor_name, constructor_params)),
            }
        }
    }

    pub(super) fn parse_data_decl(
        &mut self,
        type_table: &mut HashMap<String, Type>,
    ) -> Result<HashMap<String, Type>, ParserError> {
        assert_eq!(self.consume()?.tt, TokenType::KWData);

        let t = self.consume()?;
        let name = match t.tt {
            TokenType::UppercaseId => t.value,
            TokenType::Id => {
                return Err(self.parse_error(format!(
                    "Type IDs must begin with a capital letter. Got {}",
                    t.value
                )))
            }
            _ => {
                return Err(self.parse_error(format!(
                    "Expected type ID after data keyword, got {}",
                    t.value
                )))
            }
        };

        // parse the params
        let mut tparams = Vec::new();
        let mut t = self.consume()?;
        while t.tt == TokenType::Id {
            if tparams.contains(&t.value) {
                return Err(self.parse_error(format!("Duplicate data parameter: {}", t.value)));
            }
            tparams.push(t.value);
            t = self.consume()?;
        }

        if t.tt != TokenType::Assignment {
            return Err(self.parse_error(format!(
                "Expected \"=\" after data keyword, got {}",
                t.value
            )));
        }

        let union_type = Type::Union(
            name.clone(),
            tparams.iter().map(|v| Type::tv(v.clone())).collect(),
        );

        if let Some(_) = type_table.get(&name) {
            return Err(self.parse_error(format!("Type {} declared more than once", &name)));
        }

        type_table.insert(name.clone(), Type::fa(tparams.clone(), union_type.clone()));

        let constructors = self.parse_multiple_constructors(type_table, &tparams, &union_type)?;

        for constructor in constructors.keys() {
            if self.bound.contains(constructor) {
                return Err(
                    self.parse_error(format!("Constructor {} declared more than once", &name))
                );
            }

            self.bind(constructor.clone());
        }

        Ok(constructors)
    }

    /// Takes type table, returns the name of the data and also the type constructors
    pub(super) fn parse_type_alias_decl(
        &mut self,
        type_table: &HashMap<String, Type>,
    ) -> Result<(String, Type), ParserError> {
        assert_eq!(self.consume()?.tt, TokenType::KWType);

        let t = self.consume()?;
        let name = match t.tt {
            TokenType::UppercaseId => t.value,
            TokenType::Id => {
                return Err(self.parse_error(format!(
                    "Type IDs must begin with a capital letter. Got {}",
                    t.value
                )))
            }
            _ => {
                return Err(self.parse_error(format!(
                    "Expected type ID after type assignment, got {}",
                    t.value
                )))
            }
        };

        let t = self.consume()?;
        match t.tt {
            TokenType::Assignment => {}
            _ => {
                return Err(self.parse_error(format!(
                    "Expected \"=\" after type assignment, got {}",
                    t.value
                )))
            }
        }

        Ok((
            name,
            self.parse_type_expression(type_table, Some(&HashSet::new()))?,
        ))
    }

    pub(super) fn parse_type_expression(
        &mut self,
        type_table: &HashMap<String, Type>,
        bound_type_vars: Option<&HashSet<String>>,
    ) -> Result<Type, ParserError> {
        let mut left = self.parse_type_expression_primary(type_table, bound_type_vars)?;

        loop {
            let next = self.peek(0)?;

            match next.tt {
                TokenType::RArrow => {
                    self.advance();
                    let right = self.parse_type_expression(type_table, bound_type_vars)?;

                    match left.parser_error_if_incomplete() {
                        Ok(()) => {}
                        Err(s) => return Err(self.parse_error(s)),
                    }

                    left = Type::Function(Box::new(left), Box::new(right));
                }

                TokenType::Comma => {
                    self.advance();
                    match left.parser_error_if_incomplete() {
                        Ok(()) => {}
                        Err(s) => return Err(self.parse_error(s)),
                    }
                    left = Type::pr(
                        left,
                        self.parse_type_expression(type_table, bound_type_vars)?,
                    );
                }

                TokenType::UppercaseId | TokenType::Id | TokenType::LParen => {
                    match next.tt {
                        // If this is in an abstraction, and the next token is a double colon, then we're done because
                        // the next ID is another abst variable
                        TokenType::Id => {
                            if self.peek(1)?.tt == TokenType::DoubleColon {
                                return Ok(left);
                            }
                        }
                        _ => {}
                    }

                    let t2 = self.parse_type_expression_primary(type_table, bound_type_vars)?;
                    left = match left.type_app(&t2) {
                        Ok(t) => t,
                        Err(e) => return Err(self.parse_error(e.to_string())),
                    }
                }

                TokenType::RParen
                | TokenType::Newline
                | TokenType::EOF
                | TokenType::Dot
                | TokenType::LBrace => {
                    return match left.parser_error_if_incomplete() {
                        Ok(()) => Ok(left),
                        Err(s) => Err(self.parse_error(s)),
                    }
                }

                _ => {
                    return Err(self
                        .parse_error(format!("Unexpected token in type expression: {:?}", next)))
                }
            }
        }
    }

    fn parse_type_expression_primary(
        &mut self,
        type_table: &HashMap<String, Type>,
        bound_type_vars: Option<&HashSet<String>>,
    ) -> Result<Type, ParserError> {
        let t = self.consume()?;

        match t.tt {
            TokenType::Id => {
                if let None = bound_type_vars {
                    return Ok(Type::TypeVariable(t.value));
                }

                let id = t.value;
                if bound_type_vars.unwrap().contains(&id) {
                    Ok(Type::TypeVariable(id))
                } else {
                    Err(self.parse_error(format!("Type variable {} is not bound", id)))
                }
            }
            TokenType::UppercaseId => {
                let id = t.value;
                if let Some(t_match) = type_table.get(&id) {
                    // Ok(Type::Alias(id, Box::new(t_match.clone())))
                    Ok(t_match.clone())
                } else {
                    Err(self.parse_error(format!("Type {} is not defined", id)))
                }
            }
            TokenType::LParen => {
                let inner = self.parse_type_expression(type_table, bound_type_vars)?;
                self.advance();
                Ok(inner)
            }
            _ => Err(self.parse_error(format!(
                "Unexpected token in type expression primary: {:?}",
                t
            ))),
        }
    }

    fn parse_type_annotation(
        &mut self,
        type_table: &HashMap<String, Type>,
    ) -> Result<Type, ParserError> {
        let assigned_type = self.parse_type_expression(type_table, None)?;

        let mut sorted_tvs: Vec<String> = assigned_type
            .get_tvs_set()
            .iter()
            .map(|tv| tv.clone())
            .collect();

        sorted_tvs.sort();

        let assigned_type = Type::fa(sorted_tvs, assigned_type);

        #[cfg(debug_assertions)]
        let _assigned_type_str = assigned_type.to_string();

        Ok(assigned_type)
    }

    pub(super) fn parse_type_assignment(
        &mut self,
        type_map: &HashMap<String, Type>,
    ) -> Result<(), ParserError> {
        let name = self.peek(0)?.value.clone();
        if self.type_assignment_map.contains_key(&name) {
            return Err(self.parse_error(format!("Type already assigned: {}", name)));
        }
        self.advance();
        self.advance();

        let assigned_type = self.parse_type_annotation(type_map)?;

        self.type_assignment_map.insert(name, assigned_type);

        Ok(())
    }

    pub(super) fn get_type_assignment(&self, name: &String) -> Result<Type, ParserError> {
        match self.type_assignment_map.get(name) {
            Some(t) => Ok(t.clone()),
            None => Err(self.parse_error(format!("Type not assigned: {}", name))),
        }
    }
}
