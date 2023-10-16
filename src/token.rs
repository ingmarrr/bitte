use crate::charset::{is_all_num, is_ident};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct Source<'a> {
    /// Beginning offset of any token
    pub bix: usize,
    /// Column number of any token
    pub col: usize,
    /// Line number of any token
    pub line: usize,
    /// Source string
    pub src: &'a [u8],
}

impl<'a> std::fmt::Display for Source<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src_str = std::str::from_utf8(self.src).map_err(|_| std::fmt::Error)?;

        write!(f, "{}:{} - {}", self.line, self.col, src_str)
    }
}

impl<'a> Source<'a> {
    pub fn range(&self) -> std::ops::Range<usize> {
        self.bix..self.src.len()
    }
}

pub struct Token<'a> {
    pub src: Source<'a>,
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

impl From<u8> for TokKind {
    fn from(value: u8) -> Self {
        match value {
            b'@' => TokKind::Symbol(Symbol::At),
            b'\\' => TokKind::Symbol(Symbol::Backslash),
            b':' => TokKind::Symbol(Symbol::Colon),
            b';' => TokKind::Symbol(Symbol::Semi),
            b'=' => TokKind::Symbol(Symbol::Equal),
            b'.' => TokKind::Symbol(Symbol::Dot),
            b',' => TokKind::Symbol(Symbol::Comma),
            b'$' => TokKind::Symbol(Symbol::Dollar),

            b'"' => TokKind::Group(Opener::DQuote, Closer::DQuote),
            b'{' | b'}' => TokKind::Group(Opener::LCurly, Closer::RCurly),
            b'[' | b']' => TokKind::Group(Opener::LSquare, Closer::RSquare),
            b'(' | b')' => TokKind::Group(Opener::LParen, Closer::RParen),

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

impl TryFrom<&str> for Keyword {
    type Error = ();

    fn try_from(kind: &str) -> Result<Keyword, Self::Error> {
        match kind {
            "let" => Ok(Keyword::Let),
            "struct" => Ok(Keyword::Struct),
            "fmt" => Ok(Keyword::Fmt),
            "req" => Ok(Keyword::Req),
            "opt" => Ok(Keyword::Opt),
            "for" => Ok(Keyword::For),
            "in" => Ok(Keyword::In),
            _ => Err(()),
        }
    }
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
