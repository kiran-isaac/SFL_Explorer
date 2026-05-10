use crate::parsing::*;

fn test_lex(str: String) -> Result<Vec<Token>, LexerError> {
    let mut lexer = Lexer::new(str, None);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.get_token()?;
        if token.tt == TokenType::Newline {
            continue;
        }
        tokens.push(token.clone());
        if token.tt == TokenType::EOF {
            break;
        }
    }
    Ok(tokens)
}

fn test_lex_should_err(str: String) -> Result<LexerError, Token> {
    let mut lexer = Lexer::new(str, None);
    let token = lexer.get_token();
    match token {
        Ok(t) => Err(t),
        Err(e) => Ok(e),
    }
}

macro_rules! lexer_tokentype_test {
    ($x:expr, $y:expr) => {
        let desired: Vec<TokenType> = $y;
        let tokens = test_lex($x.to_string()).unwrap();

        assert_eq!(tokens.len(), desired.len());
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(token.tt, desired[i]);
        }
    };
}

#[test]
fn int_lit() {
    lexer_tokentype_test!(
        "123 12 23 0 -10 -10",
        vec![
            TokenType::IntLit,
            TokenType::IntLit,
            TokenType::IntLit,
            TokenType::IntLit,
            TokenType::IntLit,
            TokenType::IntLit,
            TokenType::EOF
        ]
    );
}

#[test]
fn float_lit() {
    lexer_tokentype_test!(
        "123.0 12.0 23.0 0.0 0. .0 -123.0 -12.0 -23.0 -0.0 -0. -.0",
        vec![
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::FloatLit,
            TokenType::EOF
        ]
    );
}

#[test]
fn char_lit() {
    let string = "'\\n' '\\r' 'a'";
    let tokens = test_lex(string.to_string()).unwrap();

    assert!(tokens[0].tt == TokenType::CharLit);
    assert!(tokens[0].value == "\n");

    assert!(tokens[1].tt == TokenType::CharLit);
    assert!(tokens[1].value == "\r");

    assert!(tokens[2].tt == TokenType::CharLit);
    assert!(tokens[2].value == "a");

    let invalid_char_lits = vec!["'\t'", "''", "'aa'", "'aaa'", "'\\a'"];
    for lit in invalid_char_lits {
        test_lex_should_err(lit.to_string()).unwrap();
    }
}

#[test]
fn bool_lit() {
    lexer_tokentype_test!(
        "true false",
        vec![TokenType::BoolLit, TokenType::BoolLit, TokenType::EOF]
    );
}

#[test]
fn comment() {
    lexer_tokentype_test!("//Hello\n\nx\n", vec![TokenType::Id, TokenType::EOF]);
}

#[test]
fn multi_line_comment() {
    lexer_tokentype_test!(
        "/*Hello\nWorld*/\n\nx\n",
        vec![TokenType::Id, TokenType::EOF]
    );
}

#[test]
fn lex_abstraction() {
    lexer_tokentype_test!(
        "\\x . x",
        vec![
            TokenType::Lambda,
            TokenType::Id,
            TokenType::Dot,
            TokenType::Id,
            TokenType::EOF
        ]
    );
}

#[test]
fn lex_id() {
    lexer_tokentype_test!(
        "x y x_y + - / * == >= <= > <",
        vec![
            TokenType::Id,
            TokenType::Id,
            TokenType::Id,
            TokenType::Id,
            TokenType::Id,
            TokenType::Id,
            TokenType::Id,
            TokenType::Id,
            TokenType::Id,
            TokenType::Id,
            TokenType::Id,
            TokenType::Id,
            TokenType::EOF
        ]
    );
}

#[test]
fn lex_type_stuff() {
    lexer_tokentype_test!(
        "add :: Int -> Int -> Int",
        vec![
            TokenType::Id,
            TokenType::DoubleColon,
            TokenType::UppercaseId,
            TokenType::RArrow,
            TokenType::UppercaseId,
            TokenType::RArrow,
            TokenType::UppercaseId,
            TokenType::EOF
        ]
    );
}
