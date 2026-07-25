use super::*;

impl Parser {
    fn parse_pattern<'a>(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
        unpack: bool,
        bound_set: &'a mut HashSet<String>,
    ) -> Result<(usize, &'a mut HashSet<String>), ParserError> {
        let mut left = self
            .parse_pattern_primary(ast, type_table, unpack, bound_set)?
            .0;

        #[cfg(debug_assertions)]
        let _t_queue = format!("{:?}", self.t_queue);
        loop {
            let line = self.lexer.line;
            let col = self.lexer.col;
            match &self.peek(0)?.tt {
                // If paren, apply to paren
                TokenType::LParen => {
                    self.advance();
                    let right = self.parse_pattern(ast, type_table, unpack, bound_set)?.0;
                    self.advance();
                    left = ast.add_app(left, right, line, col, false);
                }

                TokenType::RParen
                | TokenType::RArrow
                | TokenType::LBrace
                | TokenType::EOF
                | TokenType::DoubleColon
                | TokenType::Newline => {
                    return Ok((left, bound_set));
                }

                TokenType::Comma => {
                    self.advance();
                    let right = self.parse_pattern(ast, type_table, unpack, bound_set)?.0;
                    left = ast.add_pair(left, right, line, col);
                }

                TokenType::FloatLit
                | TokenType::CharLit
                | TokenType::IntLit
                | TokenType::BoolLit => {
                    let right = self
                        .parse_pattern_primary(ast, type_table, unpack, bound_set)?
                        .0;
                    left = ast.add_app(left, right, line, col, false);
                }

                TokenType::Id | TokenType::UppercaseId => {
                    // Will throw if lowercase ID is found
                    let id_node = self
                        .parse_pattern_primary(ast, type_table, unpack, bound_set)?
                        .0;
                    left = ast.add_app(left, id_node, line, col, false);
                }

                _ => {
                    let e = format!("Unexpected token in pattern: {:?}", self.peek(0)?);
                    return Err(self.parse_error(e));
                }
            }
        }
    }

    // Parse a primary expression
    fn parse_pattern_primary<'a>(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
        unpack: bool,
        bound_set: &'a mut HashSet<String>,
    ) -> Result<(usize, &'a mut HashSet<String>), ParserError> {
        let line = self.lexer.line;
        let col = self.lexer.col;
        let t = self.consume()?;
        match t.tt {
            TokenType::Id | TokenType::UppercaseId => {
                let id_name = t.value.clone();

                match id_name.chars().next().unwrap() {
                    'A'..='Z' => {
                        if !self.bound.contains(&id_name) {
                            Err(self.parse_error(format!(
                                "Unbound constructor identifier: {}",
                                id_name
                            )))
                        } else {
                            Ok((ast.add_id(t, line, col), bound_set))
                        }
                    }
                    '_' => Ok((ast.add_id(t, line, col), bound_set)),
                    'a'..='z' => {
                        if unpack {
                            if self.bound.contains(&id_name) {
                                return Err(self.parse_error(format!(
                                    "Cannot rebind already bound identifier: {}",
                                    id_name
                                )));
                            } else {
                                bound_set.insert(id_name.clone());
                            }
                        } else if !unpack && !self.bound.contains(&id_name) {
                            return Err(
                                self.parse_error(format!("Unbound Identifier: {}", id_name))
                            );
                        }
                        Ok((ast.add_id(t, line, col), bound_set))
                    }
                    _ => Err(self.parse_error(format!("unexpected char in id: {}", t.value))),
                }
            }
            TokenType::IntLit | TokenType::FloatLit | TokenType::BoolLit | TokenType::CharLit => {
                Ok((ast.add_lit(t, line, col), bound_set))
            }
            TokenType::LParen => {
                let exp = self.parse_pattern(ast, type_table, unpack, bound_set)?.0;
                self.advance();
                Ok((exp, bound_set))
            }

            _ => Err(self.parse_error(format!("Unexpected Token in pattern primary: {:?}", t))),
        }
    }

    pub(super) fn parse_match(
        &mut self,
        ast: &mut AST,
        type_table: &HashMap<String, Type>,
    ) -> Result<usize, ParserError> {
        // Parse the expression to match on, this does not allow literals
        let match_unpack = self.parse_expression(ast, type_table)?;

        match self.consume()?.tt {
            TokenType::DoubleColon => {
                ast.set_type(match_unpack, self.parse_type_expression(type_table, None)?);
                match self.consume()?.tt {
                    TokenType::LBrace => {}
                    _ => {
                        return Err(self.parse_error(
                            "Expected \"{\" after match type assignment before cases".to_string(),
                        ));
                    }
                };
            }
            TokenType::LBrace => {}
            _ => {
                return Err(self.parse_error(
                    "Expected type assignment of match subject, or lbrace".to_string(),
                ));
            }
        };

        let mut children = vec![match_unpack];

        loop {
            let t = self.peek(0)?;

            match self.peek(0)?.tt {
                TokenType::RBrace => {
                    self.advance();
                    break;
                }
                TokenType::Newline => {
                    self.advance();
                }
                TokenType::Bar => {
                    let mut bound_set = HashSet::new();
                    let bar = self.consume()?;
                    match bar.tt {
                        TokenType::Bar => {}
                        _ => {
                            return Err(
                                self.parse_error("Expected \"|\" before case pattern".to_string())
                            );
                        }
                    };

                    let case = self.parse_pattern(ast, type_table, true, &mut bound_set)?.0;

                    let arrow = self.consume()?;
                    match arrow.tt {
                        TokenType::RArrow => {}
                        _ => {
                            return Err(
                                self.parse_error("Expected \"->\" after case pattern".to_string())
                            );
                        }
                    };

                    for item in bound_set.iter() {
                        self.bind(item.clone())
                    }
                    let expr = self.parse_expression(ast, type_table)?;
                    for item in bound_set.iter() {
                        self.unbind(item)
                    }
                    children.push(case);
                    children.push(expr);
                }
                _ => {
                    return Err(self.parse_error(format!(
                        "Unexpected Token in match: expected \"|\", got: {:?}",
                        t
                    )))
                }
            }
        }

        Ok(ast.add_match(children, self.lexer.line, self.lexer.col))
    }
}
