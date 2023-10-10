use crate::charset::{is_all_num, is_ident};

pub struct Source {
    pub bix: usize,
    pub eix: usize,
}

pub struct Token<'a> {
    pub src: Source,
    pub val: Option<&'a str>,
    pub kind: TokKind,
}

pub enum TokKind {
    Keyword(Keyword),
    Symbol(Symbol),
    Group(Opener, Closer),
    Literal(Literal),
    Ident,
    Error,
    SOF,
    EOF,
    Invalid,
}

impl From<char> for TokKind {
    fn from(value: char) -> Self {
        match value {
            '@' => TokKind::Symbol(Symbol::At),
            '\\' => TokKind::Symbol(Symbol::Backslash),
            ':' => TokKind::Symbol(Symbol::Colon),
            ';' => TokKind::Symbol(Symbol::Semi),
            '=' => TokKind::Symbol(Symbol::Equal),
            '.' => TokKind::Symbol(Symbol::Dot),
            ',' => TokKind::Symbol(Symbol::Comma),
            '$' => TokKind::Symbol(Symbol::Dollar),

            '"' => TokKind::Group(Opener::DQuote, Closer::DQuote),
            '{' | '}' => TokKind::Group(Opener::LCurly, Closer::RCurly),
            '[' | ']' => TokKind::Group(Opener::LSquare, Closer::RSquare),
            '(' | ')' => TokKind::Group(Opener::LParen, Closer::RParen),

            _ => TokKind::Invalid,
        }
    }
}

impl From<&str> for TokKind {
    fn from(kind: &str) -> TokKind {
        match kind {
            "let" => TokKind::Keyword(Keyword::Let),
            "struct" => TokKind::Keyword(Keyword::Struct),
            "fmt" => TokKind::Keyword(Keyword::Fmt),
            "req" => TokKind::Keyword(Keyword::Req),
            "opt" => TokKind::Keyword(Keyword::Opt),
            "for" => TokKind::Keyword(Keyword::For),
            "in" => TokKind::Keyword(Keyword::In),

            "@" => TokKind::Symbol(Symbol::At),
            "\\" => TokKind::Symbol(Symbol::Backslash),
            ":" => TokKind::Symbol(Symbol::Colon),
            ";" => TokKind::Symbol(Symbol::Semi),
            "=" => TokKind::Symbol(Symbol::Equal),
            "." => TokKind::Symbol(Symbol::Dot),
            "," => TokKind::Symbol(Symbol::Comma),
            "$" => TokKind::Symbol(Symbol::Dollar),

            "\"" => TokKind::Group(Opener::DQuote, Closer::DQuote),
            "{" | "}" => TokKind::Group(Opener::LCurly, Closer::RCurly),
            "[" | "]" => TokKind::Group(Opener::LSquare, Closer::RSquare),
            "(" | ")" => TokKind::Group(Opener::LParen, Closer::RParen),

            st if is_ident(st) => TokKind::Ident,
            st if is_all_num(st) => TokKind::Literal(Literal::Int),

            _ => TokKind::Invalid,
        }
    }
}

pub enum Keyword {
    Let,
    Struct,
    Fmt,
    Req,
    Opt,
    For,
    In,
}

pub enum Symbol {
    At,
    Backslash,
    Colon,
    Semi,
    Comma,
    Equal,
    Dot,
    Dollar,
}

pub enum Opener {
    DQuote,
    DoubleLCurly,
    LCurly,
    LSquare,
    LParen,
}

impl TryFrom<&str> for Opener {
    type Error = ();

    fn try_from(closer: &str) -> Result<Opener, Self::Error> {
        match closer {
            "\"" => Ok(Opener::DQuote),
            "{" => Ok(Opener::LCurly),
            "{$" => Ok(Opener::DoubleLCurly),
            "[" => Ok(Opener::LSquare),
            "(" => Ok(Opener::LParen),
            _ => Err(()),
        }
    }
}

pub enum Closer {
    DQuote,
    DoubleRCurly,
    RCurly,
    RSquare,
    RParen,
}

impl TryFrom<&str> for Closer {
    type Error = ();

    fn try_from(closer: &str) -> Result<Closer, Self::Error> {
        match closer {
            "\"" => Ok(Closer::DQuote),
            "}" => Ok(Closer::RCurly),
            "$}" => Ok(Closer::DoubleRCurly),
            "]" => Ok(Closer::RSquare),
            ")" => Ok(Closer::RParen),
            _ => Err(()),
        }
    }
}

pub enum Literal {
    String(StringTy),
    List,
    Int,
    Tuple,
}

pub enum StringTy {
    InsertEnded,
    InBetween,
    InsertStarted,
    Literal,
}
