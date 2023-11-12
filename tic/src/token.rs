use crate::charset::is_ident;

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
    pub buf: &'a [u8],
    /// Source Length
    pub len: usize,
}

impl<'a> std::fmt::Debug for Source<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Source")
            .field("bix", &self.bix)
            .field("col", &self.col)
            .field("line", &self.line)
            .field("src", &std::str::from_utf8(self.buf).unwrap())
            .finish()
    }
}

impl<'a> std::fmt::Display for Source<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src_str = std::str::from_utf8(self.buf).map_err(|_| std::fmt::Error)?;
        if src_str.len() <= 1 {
            write!(f, "{}:{}", self.line, self.col)
        } else {
            write!(f, "{}:{} - [{}]", self.line, self.col, src_str)
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub src: Source<'a>,
    pub kind: TokKind,
}

impl<'a> Token<'a> {
    #[rustfmt::skip]
    pub fn is_closer(&self) -> bool {
        use TokKind::*;
        match self.kind {
            CloserDQuote 
            | RCurlyDollar 
            | RCurlyDQuote 
            | RCurly 
            | RSquare 
            | RParen => true,
            _ => false,
        }
    }

    pub fn is_opener(&self) -> bool {
        use TokKind::*;
        match self.kind {
            OpenerDQuote 
            | LCurly 
            | LCurlyDollar 
            | LCurlyDQuote 
            | LSquare 
            | LParen => true,
            _ => false,
        }
    }

    /// Converts the src to a string slice
    /// via `std::str::from_utf8`
    /// and returns the result.
    ///
    /// This should not fail as we do the utf-8
    /// checking in the lexer.
    pub fn val(&self) -> &'a str {
        std::str::from_utf8(self.src.buf).ok().unwrap()
    }

    pub fn val_owned(&self) -> String {
        self.val().to_string()
    }
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "`{}` = `{}` :: [{}]",
            self.kind,
            self.val(),
            self.src.to_string()
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokKind {
    // Keywords
    Main,
    Let,
    Dir,
    File,
    Req,
    For,
    In,
    StringKw,
    ListKw,

    // Symbols
    At,
    Bang,
    Dollar,
    Pound,
    Slash,
    Backslash,
    Colon,
    Semi,
    Comma,
    Equal,
    Dot,

    // Openers
    OpenerDQuote,
    LCurly,
    LCurlyDouble,
    LCurlyDollar,
    LCurlyDQuote,
    LSquare,
    LParen,

    // Closers
    CloserDQuote,
    RCurly,
    RCurlyDouble,
    RCurlyDollar,
    RCurlyDQuote,
    RSquare,
    RParen,

    StringLit,
    Int,

    Ident,
    Invalid,
    EOF,
}

impl std::fmt::Display for TokKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokKind::Main => write!(f, "Main"),
            TokKind::Let => write!(f, "Let"),
            TokKind::Dir => write!(f, "Directory"),
            TokKind::File => write!(f, "File"),
            TokKind::Req => write!(f, "Req"),
            TokKind::For => write!(f, "For"),
            TokKind::In => write!(f, "In"),
            TokKind::StringKw => write!(f, "String (Keyword)"),
            TokKind::ListKw => write!(f, "List (Keyword)"),

            TokKind::At => write!(f, "@"),
            TokKind::Bang => write!(f, "!"),
            TokKind::Backslash => write!(f, "\\"),
            TokKind::Slash => write!(f, "/"),
            TokKind::Colon => write!(f, ":"),
            TokKind::Semi => write!(f, ";"),
            TokKind::Comma => write!(f, ","),
            TokKind::Equal => write!(f, "="),
            TokKind::Dot => write!(f, "."),
            TokKind::Dollar => write!(f, "$"),
            TokKind::Pound => write!(f, "#"),

            TokKind::LCurly => write!(f, "{{"),
            TokKind::LCurlyDouble => write!(f, "{{{{"),
            TokKind::LCurlyDollar => write!(f, "{{$"),
            TokKind::LCurlyDQuote => write!(f, "{{\""),
            TokKind::LSquare => write!(f, "["),
            TokKind::LParen => write!(f, "("),
            TokKind::OpenerDQuote => write!(f, "\""),

            TokKind::RCurly => write!(f, "}}"),
            TokKind::RCurlyDouble => write!(f, "}}}}"),
            TokKind::RCurlyDollar => write!(f, "$}}"),
            TokKind::RCurlyDQuote => write!(f, "\"}}"),
            TokKind::RSquare => write!(f, "]"),
            TokKind::RParen => write!(f, ")"),
            TokKind::CloserDQuote => write!(f, "\""),

            TokKind::StringLit => write!(f, "String (Literal)"),
            TokKind::Int => write!(f, "Int (Literal)"),

            TokKind::Ident => write!(f, "Identifier"),
            TokKind::EOF => write!(f, "End of File"),
            TokKind::Invalid => write!(f, "Invalid Token"),
        }
    }
}

impl From<u8> for TokKind {
    fn from(value: u8) -> Self {
        match value {
            b'@' => TokKind::At,
            b'!' => TokKind::Bang,
            b'$' => TokKind::Dollar,
            b'#' => TokKind::Pound,
            b'/' => TokKind::Slash,
            b'\\' => TokKind::Backslash,
            b':' => TokKind::Colon,
            b';' => TokKind::Semi,
            b',' => TokKind::Comma,
            b'.' => TokKind::Dot,
            b'=' => TokKind::Equal,

            b'{' => TokKind::LCurly,
            b'[' => TokKind::LSquare,
            b'(' => TokKind::LParen,

            b'}' => TokKind::RCurly,
            b']' => TokKind::RSquare,
            b')' => TokKind::RParen,

            b'\0' => TokKind::EOF,
            _ => TokKind::Invalid,
        }
    }
}

impl From<&str> for TokKind {
    fn from(kind: &str) -> TokKind {
        match kind {
            "main" => TokKind::Main,
            "let" => TokKind::Let,
            "dir" => TokKind::Dir,
            "file" => TokKind::File,
            "req" => TokKind::Req,
            "for" => TokKind::For,
            "in" => TokKind::In,
            "string" => TokKind::StringKw,
            "list" => TokKind::ListKw,

            "@" => TokKind::At,
            "!" => TokKind::Bang,
            "$" => TokKind::Dollar,
            "#" => TokKind::Pound,
            "/" => TokKind::Slash,
            "\\" => TokKind::Backslash,
            ":" => TokKind::Colon,
            ";" => TokKind::Semi,
            "," => TokKind::Comma,
            "." => TokKind::Dot,
            "=" => TokKind::Equal,

            "{" => TokKind::LCurly,
            "{{" => TokKind::LCurlyDouble,
            "{$" => TokKind::LCurlyDollar,
            "{\"" => TokKind::LCurlyDQuote,
            "[" => TokKind::LSquare,
            "(" => TokKind::LParen,

            "}" => TokKind::RCurly,
            "}}" => TokKind::RCurlyDouble,
            "$}" => TokKind::RCurlyDollar,
            "\"}" => TokKind::RCurlyDQuote,
            "]" => TokKind::RSquare,
            ")" => TokKind::RParen,

            st if is_ident(st) => TokKind::Ident,
            _ => TokKind::Invalid,
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

impl From<TokKind> for Opener {
    fn from(kind: TokKind) -> Self {
        match kind {
            TokKind::OpenerDQuote => Opener::DQuote,
            TokKind::LCurly => Opener::LCurly,
            TokKind::LCurlyDollar => Opener::LCurlyDollar,
            TokKind::LCurlyDQuote => Opener::LCurlyDQuote,
            TokKind::LSquare => Opener::LSquare,
            TokKind::LParen => Opener::LParen,
            _ => panic!("Invalid opener: {}", kind),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Closer {
    DQuote,
    RCurlyDollar,
    RCurlyDQuote,
    RCurly,
    RSquare,
    RParen,
}

impl From<TokKind> for Closer {
    fn from(kind: TokKind) -> Self {
        match kind {
            TokKind::CloserDQuote => Closer::DQuote,
            TokKind::RCurlyDollar => Closer::RCurlyDollar,
            TokKind::RCurlyDQuote => Closer::RCurlyDQuote,
            TokKind::RCurly => Closer::RCurly,
            TokKind::RSquare => Closer::RSquare,
            TokKind::RParen => Closer::RParen,
            _ => panic!("Invalid closer: {}", kind),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        assert_eq!(TokKind::from("main"), TokKind::Main);
        assert_eq!(TokKind::from("let"), TokKind::Let);
        assert_eq!(TokKind::from("dir"), TokKind::Dir);
        assert_eq!(TokKind::from("file"), TokKind::File);
        assert_eq!(TokKind::from("req"), TokKind::Req);
        assert_eq!(TokKind::from("for"), TokKind::For);
        assert_eq!(TokKind::from("in"), TokKind::In);
        assert_eq!(TokKind::from("string"), TokKind::StringKw);
        assert_eq!(TokKind::from("list"), TokKind::ListKw);
    }

    #[test]
    fn test_symbols() {
        assert_eq!(TokKind::from("@"), TokKind::At);
        assert_eq!(TokKind::from("!"), TokKind::Bang);
        assert_eq!(TokKind::from("$"), TokKind::Dollar);
        assert_eq!(TokKind::from("#"), TokKind::Pound);
        assert_eq!(TokKind::from("/"), TokKind::Slash);
        assert_eq!(TokKind::from("\\"), TokKind::Backslash);
        assert_eq!(TokKind::from(":"), TokKind::Colon);
        assert_eq!(TokKind::from(";"), TokKind::Semi);
        assert_eq!(TokKind::from(","), TokKind::Comma);
        assert_eq!(TokKind::from("."), TokKind::Dot);
        assert_eq!(TokKind::from("="), TokKind::Equal);
    }

    #[test]
    fn test_openers() {
        assert_eq!(TokKind::from("{"), TokKind::LCurly);
        assert_eq!(TokKind::from("{{"), TokKind::LCurlyDouble);
        assert_eq!(TokKind::from("{$"), TokKind::LCurlyDollar);
        assert_eq!(TokKind::from("{\""), TokKind::LCurlyDQuote);
        assert_eq!(TokKind::from("["), TokKind::LSquare);
        assert_eq!(TokKind::from("("), TokKind::LParen);
    }

    #[test]
    fn test_closers() {
        assert_eq!(TokKind::from("}"), TokKind::RCurly);
        assert_eq!(TokKind::from("}}"), TokKind::RCurlyDouble);
        assert_eq!(TokKind::from("$}"), TokKind::RCurlyDollar);
        assert_eq!(TokKind::from("\"}"), TokKind::RCurlyDQuote);
        assert_eq!(TokKind::from("]"), TokKind::RSquare);
        assert_eq!(TokKind::from(")"), TokKind::RParen);
    }

    #[test]
    fn test_literals() {
        // assert_eq!(TokKind::from("123"), TokKind::Int);
        assert_eq!(TokKind::from("abc"), TokKind::Ident);
    }
}
