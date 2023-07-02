#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Cons(String),

    Dollar,
    Bang,
    Pound,
    At,

    LBrace,
    RBrace,
    LSquare,
    RSquare,
    StartString,
    EndString,

    Required,
    Optional,
    Let,
    Struct,
    Partial,
    Element,

    Assign,
    Colon,
    Eof,
    SQuote,
    Ws,
    Illegal,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Cons(s) => f.write_fmt(format_args!("Cons({})", s)),
            Token::Dollar => f.write_str("$"),
            Token::Bang => f.write_str("!"),
            Token::Pound => f.write_str("#"),
            Token::At => f.write_str("@"),
            Token::LBrace => f.write_str("{"),
            Token::RBrace => f.write_str("}"),
            Token::LSquare => f.write_str("["),
            Token::RSquare => f.write_str("]"),
            Token::Assign => f.write_str("="),
            Token::Colon => f.write_str(":"),
            Token::SQuote => f.write_str("'"),
            Token::StartString => f.write_str("'"),
            Token::EndString => f.write_str("'"),
            Token::Eof => f.write_str("EOF"),
            Token::Illegal => f.write_str("Illegal"),
            Token::Ws => f.write_str("'\\n', '\\r', '\\t', ' '"),
            Token::Required => f.write_str("req"),
            Token::Optional => f.write_str("opt"),
            Token::Let => f.write_str("let"),
            Token::Struct => f.write_str("struct"),
            Token::Partial => f.write_str("partial"),
            Token::Element => f.write_str("elem"),
        }
    }
}

impl From<char> for Token {
    fn from(value: char) -> Self {
        match value {
            '!' => Self::Bang,
            '$' => Self::Dollar,
            '#' => Self::Pound,
            '@' => Self::At,
            '{' => Self::LBrace,
            '}' => Self::RBrace,
            '[' => Self::LSquare,
            ']' => Self::RSquare,
            ':' => Self::Colon,
            '=' => Self::Assign,
            '\'' => Self::SQuote,
            '\0' => Self::Eof,
            ' ' | '\t' | '\n' | '\r' => Self::Ws,
            _ => Self::Illegal,
        }
    }
}

impl From<String> for Token {
    fn from(value: String) -> Self {
        match value.as_str() {
            "req" => Self::Required,
            "opt" => Self::Optional,
            "let" => Self::Let,
            "struct" => Self::Struct,
            "partial" => Self::Partial,
            "elem" => Self::Element,
            _ => Self::Cons(value),
        }
    }
}
