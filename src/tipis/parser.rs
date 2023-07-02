use crate::ast::{Def, Program};

use super::{
    errors::{ParseError, ParseResult, TraceError},
    lexer::Lexer,
    token::Token,
};

pub struct Parser<'a> {
    pub lex: &'a mut Lexer<'a>,
    tok: Token,
    peek: Token,
    errors: Vec<TraceError>,
}

impl<'a> Parser<'a> {
    pub fn new(lex: &'a mut Lexer<'a>) -> Parser<'a> {
        let tok = lex.next_token();
        let peek = lex.next_token();
        Parser {
            lex,
            tok,
            peek,
            errors: Vec::new(),
        }
    }

    pub fn next_token(&mut self) {
        self.tok = self.peek.clone();
        self.peek = self.lex.next_token();
    }

    pub fn parse(&mut self) -> Result<Program, Vec<String>> {
        let mut program = Program { defs: Vec::new() };
        while self.tok != Token::Eof {
            match self.parse_def() {
                Ok(def) => program.defs.push(def),
                Err(err) => self.errors.push(err),
            }
            self.next_token();
        }
        if self.errors.len() > 0 {
            Err(self.errors.clone())
        } else {
            Ok(program)
        }
    }

    fn parse_def(&mut self) -> ParseResult<Def> {
        Err(ParseError::Expected {
            expected: "def".to_string(),
            got: self.tok.to_string(),
        })
    }

    fn push_err(&mut self, err: ParseError) {
        self.errors.push(TraceError {
            error: err,
            line: self.lex.line,
            column: self.lex.column,
        });
    }
}
