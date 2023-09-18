use crate::err::LexError;

#[derive(Debug, PartialEq)]
pub enum TokKind {
    String,
    Insert,
    Ident,
    Char,
    Num,
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
    Colon,
    Comma,
    Dot,
    Req,
    Opt,
    Struct,
    Fmt,
    Let,
    For,
    Invalid,
}

impl std::fmt::Display for TokKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokKind::String => f.write_str("string"),
            TokKind::Insert => f.write_str("insert"),
            TokKind::Ident => f.write_str("ident"),
            TokKind::Char => f.write_str("char"),
            TokKind::Num => f.write_str("num"),
            TokKind::SQ => f.write_str("'"),
            TokKind::DQ => f.write_str("\""),
            TokKind::LBrace => f.write_str("{"),
            TokKind::RBrace => f.write_str("}"),
            TokKind::LBracket => f.write_str("["),
            TokKind::RBracket => f.write_str("]"),
            TokKind::LParen => f.write_str("("),
            TokKind::RParen => f.write_str(")"),
            TokKind::Eq => f.write_str("="),
            TokKind::Plus => f.write_str("+"),
            TokKind::Colon => f.write_str(":"),
            TokKind::Comma => f.write_str(","),
            TokKind::Dot => f.write_str("."),
            TokKind::Req => f.write_str("req"),
            TokKind::Opt => f.write_str("opt"),
            TokKind::Struct => f.write_str("struct"),
            TokKind::Fmt => f.write_str("fmt"),
            TokKind::Let => f.write_str("let"),
            TokKind::For => f.write_str("for"),
            TokKind::Invalid => f.write_str("invalid token"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Tok {
    String { body: String, inserts: Vec<Tok> },
    Insert { kw: InsertKw, start: usize },
    Ident(Box<str>),
    Char(char),
    Num(String),

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
    Colon,
    Comma,
    Dot,

    Req,
    Opt,

    Struct,
    Fmt,
    Let,
    For,
    Invalid,
}

impl PartialEq<TokKind> for Tok {
    fn eq(&self, other: &TokKind) -> bool {
        match self {
            Tok::String { body, inserts } if other == &TokKind::String => true,
            Tok::Insert { kw, start } if other == &TokKind::Insert => true,
            Tok::Ident(i) if other == &TokKind::Ident => true,
            Tok::Char(c) if other == &TokKind::Char => true,
            Tok::Num(n) if other == &TokKind::Num => true,
            Tok::SQ if other == &TokKind::SQ => true,
            Tok::DQ if other == &TokKind::DQ => true,
            Tok::LBrace if other == &TokKind::LBrace => true,
            Tok::RBrace if other == &TokKind::RBrace => true,
            Tok::LBracket if other == &TokKind::LBracket => true,
            Tok::RBracket if other == &TokKind::RBracket => true,
            Tok::LParen if other == &TokKind::LParen => true,
            Tok::RParen if other == &TokKind::RParen => true,
            Tok::Eq if other == &TokKind::Eq => true,
            Tok::Plus if other == &TokKind::Plus => true,
            Tok::Colon if other == &TokKind::Colon => true,
            Tok::Comma if other == &TokKind::Comma => true,
            Tok::Dot if other == &TokKind::Dot => true,
            Tok::Req if other == &TokKind::Req => true,
            Tok::Opt if other == &TokKind::Opt => true,
            Tok::Struct if other == &TokKind::Struct => true,
            Tok::Fmt if other == &TokKind::Fmt => true,
            Tok::Let if other == &TokKind::Let => true,
            Tok::For if other == &TokKind::For => true,
            Tok::Invalid if other == &TokKind::Invalid => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for Tok {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tok::String { body, inserts } => f.write_str(&body),
            Tok::Insert { kw, start } => todo!(),
            Tok::Ident(i) => f.write_str(&i),
            Tok::Char(c) => f.write_str(&c.to_string()),
            Tok::Num(n) => f.write_str(&n),
            Tok::SQ => f.write_str("'"),
            Tok::DQ => f.write_str("\""),
            Tok::LBrace => f.write_str("{"),
            Tok::RBrace => f.write_str("}"),
            Tok::LBracket => f.write_str("["),
            Tok::RBracket => f.write_str("]"),
            Tok::LParen => f.write_str("("),
            Tok::RParen => f.write_str(")"),
            Tok::Eq => f.write_str("="),
            Tok::Plus => f.write_str("+"),
            Tok::Colon => f.write_str(":"),
            Tok::Dot => f.write_str("."),
            Tok::Comma => f.write_str(","),
            Tok::Req => f.write_str("req"),
            Tok::Opt => f.write_str("opt"),
            Tok::Struct => f.write_str("struct"),
            Tok::Fmt => f.write_str("fmt"),
            Tok::Let => f.write_str("let"),
            Tok::For => f.write_str("for"),
            Tok::Invalid => f.write_str("invalid token"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InsertKw {
    For { name: Box<str>, fmt: Box<Tok> },
    Some(Box<str>),
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
            ':' => Tok::Colon,
            ',' => Tok::Comma,
            '.' => Tok::Dot,
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
            "fmt" => Tok::Fmt,
            _ => Tok::Ident(value.into_boxed_str()),
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
