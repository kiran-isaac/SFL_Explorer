use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenType {
    EOF,
    Newline,

    Id,
    UppercaseId,

    Match,
    LBrace,
    RBrace,

    IntLit,
    FloatLit,
    StringLit,
    CharLit,
    BoolLit,

    DoubleColon,
    RArrow,
    Forall,
    KWType,
    KWData,

    LParen,
    RParen,

    Lambda,

    Dollar,
    Dot,
    Comma,
    Bar,

    Assignment,
}

#[derive(Clone)]
pub struct Token {
    pub tt: TokenType,
    pub value: String,
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tt {
            TokenType::IntLit
            | TokenType::CharLit
            | TokenType::StringLit
            | TokenType::FloatLit
            | TokenType::Id => write!(f, "{:?}: {}", self.tt, self.value),
            _ => write!(f, "{:?}", self.tt),
        }
    }
}

impl Token {
    pub fn is_infix_id(&self) -> bool {
        if self.tt != TokenType::Id {
            return false;
        }
        match self.value.chars().next().unwrap() {
            'a'..='z' | 'A'..='Z' | '_' => false,
            _ => true,
        }
    }
}
