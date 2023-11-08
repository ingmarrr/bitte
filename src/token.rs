use crate::{
    ast::AstKind,
    charset::{is_all_num, is_ident},
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Clone, Copy)]
pub struct Source<'a> {
    /// Beginning offset of any token
    pub bix: usize,
    /// Column number of any token
    pub col: usize,
    /// Line number of any token
    pub line: usize,
    /// Source string
    pub src: &'a [u8],
    /// Source Length
    pub len: usize,
}

impl<'a> std::fmt::Debug for Source<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Source")
            .field("bix", &self.bix)
            .field("col", &self.col)
            .field("line", &self.line)
            .field("src", &std::str::from_utf8(self.src).unwrap())
            .finish()
    }
}

impl<'a> std::fmt::Display for Source<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src_str = std::str::from_utf8(self.src).map_err(|_| std::fmt::Error)?;
        if src_str.len() <= 1 {
            write!(f, "{}:{}", self.line, self.col)
        } else {
            write!(f, "{}:{} - [{}]", self.line, self.col, src_str)
        }
    }
}

impl<'a> Source<'a> {
    pub fn range(&self) -> std::ops::Range<usize> {
        self.bix..self.src.len()
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub src: Source<'a>,
    pub val: Option<&'a str>,
    pub kind: TokKind,
}

impl<'a> Token<'a> {
    pub fn is_closer(&self) -> bool {
        match self.kind {
            TokKind::Closer(_) => true,
            _ => false,
        }
    }

    pub fn is_opener(&self) -> bool {
        match self.kind {
            TokKind::Opener(_) => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(val) = self.val {
            return write!(
                f,
                "`{}` = `{}` :: [{}]",
                self.kind,
                val,
                self.src.to_string()
            );
        }

        write!(f, "`{}` :: [{}]", self.kind, self.src.to_string())
    }
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

impl TokKind {
    pub fn symbol_to_ast_kind(&self) -> Option<AstKind> {
        match self {
            TokKind::Symbol(sym) => sym.try_into_ast_kind(),
            _ => None,
        }
    }
}

impl std::fmt::Display for TokKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokKind::Keyword(kw) => write!(f, "{}", kw),
            TokKind::Symbol(sym) => write!(f, "{}", sym),
            TokKind::Opener(op) => write!(f, "{}", op),
            TokKind::Closer(cl) => write!(f, "{}", cl),
            TokKind::Literal(lit) => write!(f, "{}", lit),
            TokKind::Ident => write!(f, "Identifier"),
            TokKind::Error => write!(f, "Error"),
            TokKind::SOF => write!(f, "Start of File"),
            TokKind::EOF => write!(f, "End of File"),
            TokKind::Invalid => write!(f, "Invalid Token"),
        }
    }
}

impl From<u8> for TokKind {
    fn from(value: u8) -> Self {
        match value {
            b'@' => TokKind::Symbol(Symbol::At),
            b'!' => TokKind::Symbol(Symbol::Bang),
            b'$' => TokKind::Symbol(Symbol::Dollar),
            b'#' => TokKind::Symbol(Symbol::Pound),
            b'\\' => TokKind::Symbol(Symbol::Backslash),
            b'/' => TokKind::Symbol(Symbol::Slash),
            b':' => TokKind::Symbol(Symbol::Colon),
            b';' => TokKind::Symbol(Symbol::Semi),
            b'=' => TokKind::Symbol(Symbol::Equal),
            b'.' => TokKind::Symbol(Symbol::Dot),
            b',' => TokKind::Symbol(Symbol::Comma),

            b'{' => TokKind::Opener(Opener::LCurly),
            b'[' => TokKind::Opener(Opener::LSquare),
            b'(' => TokKind::Opener(Opener::LParen),

            b'}' => TokKind::Closer(Closer::RCurly),
            b']' => TokKind::Closer(Closer::RSquare),
            b')' => TokKind::Closer(Closer::RParen),

            b'\0' => TokKind::EOF,
            _ => TokKind::Invalid,
        }
    }
}

impl From<&str> for TokKind {
    fn from(kind: &str) -> TokKind {
        match kind {
            "main" => TokKind::Keyword(Keyword::Main),
            "let" => TokKind::Keyword(Keyword::Let),
            "dir" => TokKind::Keyword(Keyword::Dir),
            "file" => TokKind::Keyword(Keyword::File),
            "fmt" => TokKind::Keyword(Keyword::Fmt),
            "req" => TokKind::Keyword(Keyword::Req),
            "opt" => TokKind::Keyword(Keyword::Opt),
            "for" => TokKind::Keyword(Keyword::For),
            "in" => TokKind::Keyword(Keyword::In),

            "@" => TokKind::Symbol(Symbol::At),
            "#" => TokKind::Symbol(Symbol::At),
            "!" => TokKind::Symbol(Symbol::Bang),
            "\\" => TokKind::Symbol(Symbol::Backslash),
            "/" => TokKind::Symbol(Symbol::Slash),
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

            st if is_ident(st) => TokKind::Ident,
            st if is_all_num(st) => TokKind::Literal(Literal::Int),

            _ => TokKind::Invalid,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Main,
    Let,
    Dir,
    File,
    Fmt,
    Req,
    Opt,
    For,
    In,
    String,
    List,
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Main => write!(f, "Main"),
            Keyword::Let => write!(f, "Let"),
            Keyword::Dir => write!(f, "Directory"),
            Keyword::File => write!(f, "File"),
            Keyword::Fmt => write!(f, "Fmt"),
            Keyword::Req => write!(f, "Req"),
            Keyword::Opt => write!(f, "Opt"),
            Keyword::For => write!(f, "For"),
            Keyword::In => write!(f, "In"),
            Keyword::String => write!(f, "String"),
            Keyword::List => write!(f, "List"),
        }
    }
}

impl TryFrom<&str> for Keyword {
    type Error = ();

    fn try_from(kind: &str) -> Result<Keyword, Self::Error> {
        match kind {
            "main" => Ok(Keyword::Main),
            "let" => Ok(Keyword::Let),
            "dir" => Ok(Keyword::Dir),
            "file" => Ok(Keyword::File),
            "fmt" => Ok(Keyword::Fmt),
            "req" => Ok(Keyword::Req),
            "opt" => Ok(Keyword::Opt),
            "for" => Ok(Keyword::For),
            "in" => Ok(Keyword::In),
            "string" => Ok(Keyword::String),
            "list" => Ok(Keyword::List),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    At,
    Backslash,
    Slash,
    Colon,
    Semi,
    Comma,
    Equal,
    Dot,
    Dollar,
    Bang,
    Pound,
}

impl Symbol {
    pub fn try_into_ast_kind(&self) -> Option<AstKind> {
        match self {
            Self::At => Some(AstKind::Dir),
            Self::Bang => Some(AstKind::Let),
            Self::Pound => Some(AstKind::File),
            _ => None,
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::At => write!(f, "@"),
            Symbol::Bang => write!(f, "!"),
            Symbol::Backslash => write!(f, "\\"),
            Symbol::Slash => write!(f, "/"),
            Symbol::Colon => write!(f, ":"),
            Symbol::Semi => write!(f, ";"),
            Symbol::Comma => write!(f, ","),
            Symbol::Equal => write!(f, "="),
            Symbol::Dot => write!(f, "."),
            Symbol::Dollar => write!(f, "$"),
            Symbol::Pound => write!(f, "#"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opener {
    DQuote,
    LCurly,
    LCurlyDollar,
    LCurlyDQuote,
    LSquare,
    LParen,
}

impl<'a> TryFrom<Token<'a>> for Opener {
    type Error = ();

    fn try_from(token: Token<'a>) -> Result<Opener, Self::Error> {
        match token.kind {
            TokKind::Opener(op) => Ok(op),
            _ => Err(()),
        }
    }
}

impl Opener {
    pub fn closer(&self) -> Closer {
        match self {
            Opener::DQuote => Closer::DQuote,
            Opener::LCurly => Closer::RCurly,
            Opener::LCurlyDollar => Closer::RCurlyDollar,
            Opener::LCurlyDQuote => Closer::RCurlyDQuote,
            Opener::LSquare => Closer::RSquare,
            Opener::LParen => Closer::RParen,
        }
    }
}

impl std::fmt::Display for Opener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opener::DQuote => write!(f, "\""),
            Opener::LCurly => write!(f, "{{"),
            Opener::LCurlyDollar => write!(f, "{{$"),
            Opener::LCurlyDQuote => write!(f, "{{\""),
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
            "{$" => Ok(Opener::LCurlyDollar),
            "{\"" => Ok(Opener::LCurlyDQuote),
            "[" => Ok(Opener::LSquare),
            "(" => Ok(Opener::LParen),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Closer {
    DQuote,
    RCurlyDollar,
    RCurlyDQuote,
    RCurly,
    RSquare,
    RParen,
}

impl<'a> TryFrom<Token<'a>> for Closer {
    type Error = ();

    fn try_from(token: Token<'a>) -> Result<Closer, Self::Error> {
        match token.kind {
            TokKind::Closer(cl) => Ok(cl),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Closer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Closer::DQuote => write!(f, "\""),
            Closer::RCurlyDollar => write!(f, "$}}"),
            Closer::RCurlyDQuote => write!(f, "\"}}"),
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
            "$}" => Ok(Closer::RCurlyDollar),
            "\"}" => Ok(Closer::RCurlyDQuote),
            "]" => Ok(Closer::RSquare),
            ")" => Ok(Closer::RParen),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Literal {
    String,
    List,
    Int,
    Tuple,
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Literal::String(StringTy::InsertEnded) => write!(f, "String Insert Ended"),
            // Literal::String(StringTy::InBetween) => write!(f, "String In Between"),
            // Literal::String(StringTy::InsertStarted) => write!(f, "String Insert Started"),
            // Literal::String(StringTy::Literal) => write!(f, "String Literal"),
            Literal::String => write!(f, "String"),
            Literal::List => write!(f, "List"),
            Literal::Int => write!(f, "Int"),
            Literal::Tuple => write!(f, "Tuple"),
        }
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum StringTy {
//     InsertEnded,
//     InBetween,
//     InsertStarted,
//     Literal,
// }
