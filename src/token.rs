use crate::charset::{is_all_num, is_ident};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Copy)]
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

        write!(f, "{}:{} - [{}]", self.line, self.col, src_str)
    }
}

impl<'a> Source<'a> {
    pub fn range(&self) -> std::ops::Range<usize> {
        self.bix..self.src.len()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub src: Source<'a>,
    pub val: Option<&'a str>,
    pub kind: TokKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokKind {
    Keyword(Keyword),
    Symbol(Symbol),
    Opener(Opener),
    Closer(Closer),
    Literal(Literal),
    Ident,
    Error,
    SOF,
    EOF,
    Invalid,
}

impl std::fmt::Display for TokKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokKind::Keyword(kw) => write!(f, "{}", kw),
            TokKind::Symbol(sym) => write!(f, "{}", sym),
            // TokKind::Group(op, cl) => write!(f, "{}{}", op, cl),
            TokKind::Opener(op) => write!(f, "{}", op),
            TokKind::Closer(cl) => write!(f, "{}", cl),
            TokKind::Literal(lit) => write!(f, "{}", lit),
            TokKind::Ident => write!(f, "Identifier"),
            TokKind::Error => write!(f, "Error"),
            TokKind::SOF => write!(f, "Start of File"),
            TokKind::EOF => write!(f, "End of File"),
            TokKind::Invalid => write!(f, "Invalid"),
        }
    }
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

            b'{' => TokKind::Opener(Opener::LCurly),
            b'[' => TokKind::Opener(Opener::LSquare),
            b'(' => TokKind::Opener(Opener::LParen),

            b'}' => TokKind::Closer(Closer::RCurly),
            b']' => TokKind::Closer(Closer::RSquare),
            b')' => TokKind::Closer(Closer::RParen),

            // b'"' => TokKind::Group(Opener::DQuote, Closer::DQuote),
            // b'{' | b'}' => TokKind::Group(Opener::LCurly, Closer::RCurly),
            // b'[' | b']' => TokKind::Group(Opener::LSquare, Closer::RSquare),
            // b'(' | b')' => TokKind::Group(Opener::LParen, Closer::RParen),
            b'\0' => TokKind::EOF,
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

            "{" => TokKind::Opener(Opener::LCurly),
            "[" => TokKind::Opener(Opener::LSquare),
            "(" => TokKind::Opener(Opener::LParen),

            "}" => TokKind::Closer(Closer::RCurly),
            "]" => TokKind::Closer(Closer::RSquare),
            ")" => TokKind::Closer(Closer::RParen),

            // "\"" => TokKind::Group(Opener::DQuote, Closer::DQuote),
            // "{" | "}" => TokKind::Group(Opener::LCurly, Closer::RCurly),
            // "[" | "]" => TokKind::Group(Opener::LSquare, Closer::RSquare),
            // "(" | ")" => TokKind::Group(Opener::LParen, Closer::RParen),
            st if is_ident(st) => TokKind::Ident,
            st if is_all_num(st) => TokKind::Literal(Literal::Int),

            _ => TokKind::Invalid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Let,
    Struct,
    Fmt,
    Req,
    Opt,
    For,
    In,
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Let => write!(f, "let"),
            Keyword::Struct => write!(f, "struct"),
            Keyword::Fmt => write!(f, "fmt"),
            Keyword::Req => write!(f, "req"),
            Keyword::Opt => write!(f, "opt"),
            Keyword::For => write!(f, "for"),
            Keyword::In => write!(f, "in"),
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::At => write!(f, "@"),
            Symbol::Backslash => write!(f, "\\"),
            Symbol::Colon => write!(f, ":"),
            Symbol::Semi => write!(f, ";"),
            Symbol::Comma => write!(f, ","),
            Symbol::Equal => write!(f, "="),
            Symbol::Dot => write!(f, "."),
            Symbol::Dollar => write!(f, "$"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opener {
    DQuote,
    DoubleLCurly,
    LCurly,
    LSquare,
    LParen,
}

impl std::fmt::Display for Opener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opener::DQuote => write!(f, "\""),
            Opener::DoubleLCurly => write!(f, "{{"),
            Opener::LCurly => write!(f, "{{"),
            Opener::LSquare => write!(f, "["),
            Opener::LParen => write!(f, "("),
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Closer {
    DQuote,
    DoubleRCurly,
    RCurly,
    RSquare,
    RParen,
}

impl std::fmt::Display for Closer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Closer::DQuote => write!(f, "\""),
            Closer::DoubleRCurly => write!(f, "}}"),
            Closer::RCurly => write!(f, "}}"),
            Closer::RSquare => write!(f, "]"),
            Closer::RParen => write!(f, ")"),
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Literal {
    String(StringTy),
    List,
    Int,
    Tuple,
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(StringTy::InsertEnded) => write!(f, "String Insert Ended"),
            Literal::String(StringTy::InBetween) => write!(f, "String In Between"),
            Literal::String(StringTy::InsertStarted) => write!(f, "String Insert Started"),
            Literal::String(StringTy::Literal) => write!(f, "String Literal"),
            Literal::List => write!(f, "List"),
            Literal::Int => write!(f, "Int"),
            Literal::Tuple => write!(f, "Tuple"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringTy {
    InsertEnded,
    InBetween,
    InsertStarted,
    Literal,
}
