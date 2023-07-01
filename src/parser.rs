use crate::ast::{Def, Insertion, ListIdent, Program, Type, Value};
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
        let ty = match self.tok {
            Token::Pound => Type::List,
            Token::Dollar => Type::String,
            _ => {
                self.push_err(vec!["$", "#"]);
                return None;
            }
        };

        self.next_token();
        let ident = if let Token::Ident(name) = &self.tok {
            name.clone()
        } else {
            self.push_err(vec!["identifier"]);
            return None;
        };

        self.next_token();
        if !self.is_token(Token::Assign, vec!["="]) {
            return None;
        }

        self.next_token();
        let value = match self.tok {
            Token::Pound => self.parse_list_str(),
            Token::Bang => self.parse_element(),
            Token::SQuote => self.parse_str(),
            Token::RSquare => todo!(),
            _ => {
                self.push_err(vec!["#", "$", "'"]);
                None
            }
        };

        match value {
            Some(val) => Some(Def::Let { ty, ident, val }),
            None => None,
        }
    }

    fn parse_list_str(&mut self) -> Option<Value> {
        self.next_token();
        let ident = if let Token::Ident(name) = &self.tok {
            ListIdent::Name(name.clone())
        } else {
            self.push_err(vec!["identifier"]);
            return None;
        };

        self.next_token();
        if !self.is_token(Token::LSquirly, vec!["{"]) {
            return None;
        };

        let value_string = self.parse_str();
        let (body, insertions) = if let Some(Value::String { body, insertions }) = value_string {
            (body, insertions)
        } else {
            self.push_err(vec!["string"]);
            return None;
        };

        self.next_token();
        if !self.is_token(Token::RSquirly, vec!["}"]) {
            return None;
        }

        Some(Value::ListStr {
            ident,
            body,
            insertions,
        })
    }

    fn parse_element(&mut self) -> Option<Value> {
        None
    }

    fn parse_str(&mut self) -> Option<Value> {
        self.next_token();
        if let Token::SQuote = self.tok {
            return Some(Value::String {
                body: String::new(),
                insertions: vec![],
            });
        }

        if let Token::Ident(body) = self.tok.clone() {
            return self.parse_str_body(&body);
        }

        self.push_err(vec!["string"]);
        None
    }

    fn parse_str_body(&mut self, body: &str) -> Option<Value> {
        let mut _insertions = vec![];
        let mut chs = body.chars().peekable();

        loop {
            match chs.next() {
                Some(ch) if ch != '\'' => {
                    let peeked = chs.peek();
                    match Token::from(ch) {
                        Token::Dollar => match peeked {
                            Some(peek) if peek.is_alphabetic() => {
                                let mut ident = String::new();
                                while ch.is_alphanumeric() || ch == '_' {
                                    ident.push(ch);
                                    chs.next();
                                }
                                _insertions.push(Insertion::Ident(ident));
                            }
                            _ => _insertions.push(Insertion::Empty),
                        },
                        Token::Bang => {}
                        Token::Pound => match peeked {
                            Some(peek) if peek.is_alphabetic() => {
                                let mut ident = String::new();

                                while let Some(&c) = chs.peek() {
                                    if c.is_alphanumeric() || c == '_' {
                                        ident.push(c);
                                        chs.next();
                                    } else {
                                        break;
                                    }
                                }
                                if let Some('{') = chs.next() {
                                    let body = match chs.next() {
                                        Some('\'') => {
                                            let mut body = String::new();
                                            while let Some(&c) = chs.peek() {
                                                match c {
                                                    '\'' => {
                                                        chs.next().unwrap();
                                                        if let Some(&'}') = chs.peek() {
                                                            break;
                                                        }
                                                        self.push_err(vec!["}"]);
                                                        break;
                                                    }
                                                    '\0' => {
                                                        self.push_err(vec!["'"]);
                                                        break;
                                                    }
                                                    _ => {
                                                        body.push(c);
                                                        chs.next();
                                                    }
                                                }
                                            }
                                            body
                                        }
                                        _ => {
                                            self.push_err(vec!["'"]);
                                            return None;
                                        }
                                    };
                                    let value = self.parse_str_body(&body);

                                    match value {
                                        Some(Value::String { body, insertions }) => _insertions
                                            .push(Insertion::Value(Value::ListStr {
                                                ident: ListIdent::Name(ident),
                                                body,
                                                insertions,
                                            })),
                                        _ => {
                                            self.push_err(vec!["string"]);
                                            return None;
                                        }
                                    }
                                } else {
                                    _insertions.push(Insertion::Ident(ident));
                                    continue;
                                }
                            }
                            Some(peek) if peek == &'[' => {
                                let mut list_buf = String::new();
                                while ch != ']' {
                                    list_buf.push(ch);
                                    chs.next();
                                }
                                let list = list_buf
                                    .split(',')
                                    .map(|s| s.to_owned())
                                    .collect::<Vec<String>>();
                                let value = self.parse_str();
                                let (body, ins) = match value {
                                    Some(val) => match val {
                                        Value::String { body, insertions } => (body, insertions),
                                        Value::ListStr {
                                            body, insertions, ..
                                        } => (body, insertions),
                                        _ => {
                                            self.push_err(vec!["string", "list expansion"]);
                                            return None;
                                        }
                                    },
                                    None => {
                                        self.push_err(vec!["string", "list expansion"]);
                                        return None;
                                    }
                                };
                                _insertions.push(Insertion::Value(Value::ListStr {
                                    ident: ListIdent::Anonymos(list),
                                    body,
                                    insertions: ins,
                                }));
                            }
                            _ => {
                                self.push_err(vec!["identitifer", "["]);
                                return None;
                            }
                        },
                        _ => (),
                    }
                }
                _ => {
                    return Some(Value::String {
                        body: body.into(),
                        insertions: _insertions,
                    })
                }
            }
        }
    }

    fn is_token(&mut self, tok: Token, expected: Vec<&'a str>) -> bool {
        if self.tok != tok {
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
