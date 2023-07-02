#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    Ident(String),
    String(String),

    Dollar,
    Bang,
    Pound,
    At,

    LSquirly,
    RSquirly,
    LSquare,
    RSquare,

    Assign,
    Colon,
    SQuote,
    Eof,
    Illegal,

    Required,
    Optional,
    Let,
    Struct,
    Partial,
    Element,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(ident) => f.write_fmt(format_args!("Ident({})", ident)),
            Token::String(s) => f.write_fmt(format_args!("String({})", s)),
            Token::Dollar => f.write_str("Dollar"),
            Token::Bang => f.write_str("Bang"),
            Token::Pound => f.write_str("Pound"),
            Token::At => f.write_str("At"),
            Token::LSquirly => f.write_str("LeftSquirly"),
            Token::RSquirly => f.write_str("RightSquirly"),
            Token::LSquare => f.write_str("LeftSquare"),
            Token::RSquare => f.write_str("RightSquare"),
            Token::Assign => f.write_str("Assign"),
            Token::Colon => f.write_str("Colon"),
            Token::SQuote => f.write_str("Single Quote"),
            Token::Eof => f.write_str("EOF"),
            Token::Illegal => f.write_str("Illegal"),
            Token::Required => f.write_str("Required"),
            Token::Optional => f.write_str("Optional"),
            Token::Let => f.write_str("Let"),
            Token::Struct => f.write_str("Struct"),
            Token::Partial => f.write_str("Partial"),
            Token::Element => f.write_str("Element"),
        }
    }
}

impl From<char> for Token {
    fn from(ch: char) -> Self {
        match ch {
            '!' => Self::Bang,
            '@' => Self::At,
            '#' => Self::Pound,
            '$' => Self::Dollar,
            '=' => Self::Assign,
            '{' => Self::LSquirly,
            '}' => Self::RSquirly,
            '[' => Self::LSquare,
            ']' => Self::RSquare,
            ':' => Self::Colon,
            '\'' => Self::SQuote,
            '\0' => Self::Eof,
            _ => Self::Illegal,
        }
    }
}

impl From<String> for Token {
    fn from(st: String) -> Self {
        match st.as_str() {
            "required" => Self::Required,
            "optional" => Self::Optional,
            "let" => Self::Let,
            "struct" => Self::Struct,
            "partial" => Self::Partial,
            "elem" => Self::Element,
            _ => Self::Ident(st),
        }
    }
}
