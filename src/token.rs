

#[derive(Debug, PartialEq)]
pub enum Token {
    String(String),
    Ident(String),
    Alpha(char),
    Num(char),
    NonAlphaNum(char),

    Let,
    Struct,
    Elem,
    Optional,
    Required,

    Eq,
    LCurly,
    RCurly,
    LSquare,
    RSquare,
    Colon,
    Comma,

    SQuote,
    Bang,
    At,
    Pound,
    Dollar,

    Escape,
    Newline,
    Whitespace,
    Eof,
    Illegal
}

#[derive(Debug, PartialEq)]
pub enum CharToken {
    Eq,
    LCurly,
    RCurly,
    LSquare,
    RSquare,
    Colon,
    Comma,

    SQuote,
    Bang,
    At,
    Pound,
    Dollar,

    Illegal,
}


impl From<char> for Token {
    fn from(value: char) -> Self {
        match value {
            '\'' => Token::SQuote,
            '!' => Token::Bang,
            '@' => Token::At,
            '#' => Token::Pound,
            '$' => Token::Dollar,
            '=' => Token::Eq,
            '{' => Token::LCurly,
            '}' => Token::RCurly,
            '[' => Token::LSquare,
            ']' => Token::RSquare,
            ':' => Token::Colon,
            ',' => Token::Comma,
            '\\' => Token::Escape,
            '\n' => Token::Newline,
            ' ' | '\t' | '\r' => Token::Whitespace,
            '\0' => Token::Eof,
            _ if value.to_string().parse::<usize>().is_ok() => Token::Num(value),
            _ if value.is_alphabetic() => Token::Alpha(value),
            _ => Token::NonAlphaNum(value),
        }
    }
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        match value {
            "let" => Token::Let,
            "struct" => Token::Struct,
            "elem" => Token::Elem,
            "opt" => Token::Optional,
            "req" => Token::Required,
            _ => Token::Ident(value.to_owned())
        }
    }
}


impl Token {
    pub fn is_injector(&self) -> bool {
        match self {
            Self::Bang | Self::Pound | Self::Dollar => true,
            _ => false
        }
    }

    pub fn is_alphanum(&self) -> bool {
        match self {
            Self::Alpha(_) | Self::Num(_) => true,
            _ => false
        }
    }

    pub fn is_whitespace(&self) -> bool {
        match self {
            Self::Whitespace | Self::Newline => true,
            _ => false
        }
    }
}

pub enum Injector {
    Bang,
    Dollar,
    Pound,
}

impl Into<Token> for Injector {
    fn into(self) -> Token {
        match self {
            Self::Bang => Token::Bang,
            Self::Dollar => Token::Dollar,
            Self::Pound => Token::Pound,
        }
    }
}









