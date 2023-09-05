use crate::err::LexError;

#[derive(Debug, PartialEq)]
pub enum Tok {
    String { body: String, inserts: Vec<Tok> },
    Insert { kw: InsertKw, start: usize },
    Ident(String),
    Char(char),

    SQ,
    DQ,

    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,

    Eq,
    Plus,

    Req,
    Opt,

    Struct,
    Let,
    For,
    Invalid,
}

#[derive(Debug, PartialEq)]
pub enum InsertKw {
    For { name: String, fmt: Box<Tok> },
    Some(String),
    Number(usize),
    None,
}

impl Tok {
    pub fn is_init(&self) -> bool {
        match self {
            Tok::SQ | Tok::DQ | Tok::LBrace | Tok::LBracket | Tok::LParen | Tok::Char(_) => true,
            _ => false,
        }
    }
}
impl From<char> for Tok {
    fn from(value: char) -> Self {
        match value {
            '\'' => Tok::SQ,
            '"' => Tok::DQ,
            '{' => Tok::LBrace,
            '}' => Tok::RBrace,
            '[' => Tok::LBracket,
            ']' => Tok::RBracket,
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            '=' => Tok::Eq,
            '+' => Tok::Plus,
            '?' => Tok::Opt,
            '!' => Tok::Req,
            c if value.is_alphabetic() => Tok::Char(c),
            _ => Tok::Invalid,
        }
    }
}

impl From<String> for Tok {
    fn from(value: String) -> Self {
        match value.as_str() {
            "struct" => Tok::Struct,
            "let" => Tok::Let,
            "for" => Tok::For,
            _ => Tok::Ident(value),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InitTok {
    SQ,
    DQ,
    LBrace,
    LBracket,
    LParen,
    Char,
}

impl TryFrom<Tok> for InitTok {
    type Error = LexError;

    fn try_from(value: Tok) -> Result<Self, Self::Error> {
        match value {
            Tok::SQ => Ok(InitTok::SQ),
            Tok::DQ => Ok(InitTok::DQ),
            Tok::LBrace => Ok(InitTok::LBrace),
            Tok::LBracket => Ok(InitTok::LBracket),
            Tok::LParen => Ok(InitTok::LParen),
            Tok::Char(_) => Ok(InitTok::Char),
            _ => Err(LexError::NotInit),
        }
    }
}
