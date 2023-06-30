use crate::ast::{Def, Program, Type};
use crate::lexer::Lexer;
use crate::token::Token;

pub struct Parser<'a> {
    pub lex: &'a mut Lexer<'a>,
    tok: Token,
    peek: Token,
    errors: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(lex: &'a mut Lexer<'a>) -> Parser<'a> {
        let tok = lex.next_token();
        let peek = lex.next_token();
        let parser = Parser {
            lex,
            tok,
            peek,
            errors: vec![],
        };
        parser
    }

    pub fn next_token(&mut self) {
        print!("Tok: {} | ", self.tok);
        println!("Peek: {}", self.peek);
        self.tok = self.peek.clone();
        self.peek = self.lex.next_token();
    }

    pub fn parse(&mut self) -> Program {
        let mut program = Program { defs: vec![] };
        //println!("{:#?}", self.tok);
        //println!("{:#?}", self.peek);

        while self.tok != Token::Eof {
            if let Some(def) = self.parse_def(self.tok.clone()) {
                program.defs.push(def);
            }
            self.next_token();
        }

        program
    }

    fn parse_def(&mut self, tok: Token) -> Option<Def> {
        let parsed = match tok {
            Token::Let => self.parse_let(),
            _ => None,
        };
        parsed
    }

    fn parse_let(&mut self) -> Option<Def> {
        self.next_token();
        if !self.is_token(Token::Dollar, vec!["#", "$"]) {
            return None;
        }

        self.next_token();
        let ident = if let Token::Ident(name) = &self.tok {
            name.as_str()
        } else {
            self.push_err(vec!["identifier"]);
            return None;
        };

        self.next_token();
        if !self.is_token(Token::Assign, vec!["="]) {
            return None;
        }

        self.next_token();
        match self.tok {
            Token::Pound => self.parse_list_str(),
            Token::Bang => self.parse_element(),
            Token::SQuote => self.parse_str(),
            _ => {
                self.push_err(vec!["#", "$", "'"]);
                None
            }
        }
    }

    fn parse_list_str(&mut self) -> Option<Def> {
        self.next_token();
        let ident = if let Token::Ident(name) = &self.tok {
            name.clone()
        } else {
            self.push_err(vec!["identifier"]);
            return None;
        };

        self.next_token();
        if !self.is_token(Token::LSquirly, vec!["{"]) {
            return None;
        };
        self.next_token();
        let val = if let Token::Ident(body) = &self.tok {
            body.clone()
        } else {
            self.push_err(vec!["string"]);
            return None;
        };

        self.next_token();
        if !self.is_token(Token::RSquirly, vec!["}"]) {
            return None;
        }

        Some(Def::Let {
            ty: Type::List,
            ident,
            val,
        })
    }

    fn parse_element(&mut self) -> Option<Def> {
        None
    }

    fn parse_str(&mut self) -> Option<Def> {
        None
    }

    fn is_token(&mut self, tok: Token, expected: Vec<&'a str>) -> bool {
        if self.tok != tok {
            self.push_err(expected);
            return false;
        }
        true
    }

    fn is_peek_token(&mut self, tok: Token, expected: Vec<&'a str>) -> bool {
        if self.peek != tok {
            self.push_err(expected);
            return false;
        }
        true
    }

    fn push_err(&mut self, expected: Vec<&'a str>) {
        self.errors
            .push(format!("Expected {:?}, found: {}", expected, self.tok));
    }
}
