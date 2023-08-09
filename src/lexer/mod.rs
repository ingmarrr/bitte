use std::{iter::Peekable, str::Chars};

use crate::token::Token;

pub mod error;
#[cfg(test)]
pub mod test;

pub struct Lexer<'a> {
    pub ch: char,
    pub peek: char,
    pub inp: Peekable<Chars<'a>>,
    pub line: usize,
    pub col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        let mut inp = input.chars().peekable();
        let peek = *inp.peek().unwrap_or(&'\0');
        Lexer {
            ch: '\0',
            peek,
            inp,
            line: 0,
            col: 0,
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            if let Token::Eof = tok {
                return tokens;
            }
            tokens.push(tok);
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.next_char();
        self.skip_ws();
        let tok = Token::from(self.ch);
        match tok {
            Token::SQuote => self.parse_str(),
            Token::Alpha(_) => self.parse_ident(),
            _ => tok
        }
    }

    fn parse_ident(&mut self) -> Token {
        let mut buf = String::new();
        loop {
            let tok = Token::from(self.ch);
            if !tok.is_alphanum() {
                return Token::Ident(buf);
            }
            buf.push(self.ch);
            self.next_char();
        }
    }

    fn parse_str(&mut self) -> Token {
        let mut buf = String::new();
        self.next_char();
        loop {
            let tok = Token::from(self.ch);
            if let Token::SQuote | Token::Eof = tok {
                return Token::String(buf);
            }

            buf.push(self.ch);
            self.next_char();
        }
    }

    fn next_char(&mut self) {
        self.ch = self.inp.next().unwrap_or('\0');
        self.peek = *self.inp.peek().unwrap_or(&'\0');
    }

    fn skip_ws(&mut self) {
        loop {
            match Token::from(self.ch) {
                Token::Whitespace => {
                    self.next_char();
                    self.col += 1;
                }
                Token::Newline => {
                    self.next_char();
                    self.line += 1;
                    self.col = 0;
                }
                _ => break,
            }
        }
    }
}
