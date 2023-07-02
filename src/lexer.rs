use std::str::Chars;

use crate::token::Token;

pub type LexResult<T> = std::result::Result<T, LexError>;

#[derive(Debug, thiserror::Error)]
pub enum LexError {
    #[error("Illegal token.")]
    Illegal,
}

pub struct Lexer<'a> {
    input: std::iter::Peekable<Chars<'a>>,
    char: char,
    inserting: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        let mut lex = Lexer {
            input: input.chars().peekable(),
            char: '\0',
            inserting: 0,
        };
        lex.read_char();
        lex
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_ws();
        let tok = match self.char {
            'a'..='z' | 'A'..='Z' | '_' => {
                let ident = self.read_ident();
                return Token::from(ident);
            }
            '\'' => Token::String(self.read_str()),
            _ => Token::from(self.char),
        };
        self.read_char();
        tok
    }

    fn read_char(&mut self) {
        self.char = match self.input.peek() {
            Some(ch) => *ch,
            None => '\0',
        };
        self.input.next();
    }

    fn __peek_char(&mut self) -> char {
        match self.input.peek() {
            Some(ch) => *ch,
            None => '\0',
        }
    }

    fn read_ident(&mut self) -> String {
        let mut buf = String::new();
        while let 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' = self.char {
            buf.push(self.char);
            self.read_char();
        }
        buf
    }

    fn read_str(&mut self) -> String {
        let mut buf = String::new();
        self.read_char();
        self.inserting += 1;
        loop {
            match self.char {
                '\'' => match self.inserting {
                    1 => break,
                    _ => self.inserting -= 1,
                },
                '#' | '!' => {
                    self.inserting += 1;
                }
                _ => {}
            };
            buf.push(self.char);
            self.read_char();
        }
        buf
    }

    fn skip_ws(&mut self) {
        while self.char.is_whitespace() {
            self.read_char();
        }
    }
}
