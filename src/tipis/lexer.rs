use super::token::Token;

pub struct Lexer<'a> {
    input: std::iter::Peekable<std::str::Chars<'a>>,
    ch: char,
    next_squote_opens_string: bool,
    required_string_closings: usize,
    in_string: bool,
    pub line: usize,
    pub col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            input: input.chars().peekable(),
            ch: '\0',
            next_squote_opens_string: false,
            required_string_closings: 0,
            in_string: false,
            line: 0,
            col: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_ws();

        match self.ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => Token::Cons(self.read_cons()),
            '\'' => self.handle_squote(),
            '{' => self.handle_lbrace(),
            '}' => self.handle_rbrace(),
            _ => Token::from(self.ch),
        }
    }

    fn handle_lbrace(&mut self) -> Token {
        if self.in_string && self.next_squote_opens_string {
            self.in_string = false;
        }
        Token::LBrace
    }

    fn handle_rbrace(&mut self) -> Token {
        if !self.in_string && self.required_string_closings > 0 {
            self.in_string = true;
        }
        Token::RBrace
    }

    fn handle_squote(&mut self) -> Token {
        match self.next_squote_opens_string {
            true => {
                self.next_squote_opens_string = false;
                self.in_string = true;
                Token::StartString
            }
            false => match self.required_string_closings {
                0 => Token::SQuote,
                _ => {
                    self.required_string_closings -= 1;
                    self.in_string = false;
                    Token::EndString
                }
            },
        }
    }

    fn read_cons(&mut self) -> String {
        let mut buf = String::new();
        match self.in_string {
            true => {
                while let Some(ch) = self.input.peek() {
                    match ch {
                        '#' | '!' => {
                            self.next_squote_opens_string = true;
                            break;
                        }
                        '$' | '\0' | '\'' => break,
                        '{' if !self.next_squote_opens_string => break,
                        _ => buf.push(self.ch),
                    };
                    self.read_char();
                }
            }
            false => {
                while let 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' = self.ch {
                    buf.push(self.ch);
                    self.read_char();
                }
            }
        };
        buf
    }

    fn read_char(&mut self) {
        self.ch = match self.input.peek() {
            Some(ch) => *ch,
            None => '\0',
        };
        self.input.next();
        self.col += 1;
    }

    fn skip_ws(&mut self) {
        if self.in_string {
            return;
        }

        while self.ch.is_whitespace() {
            if self.ch == '\n' {
                self.line += 1;
                self.col = 0;
            }
            self.read_char();
        }
    }
}
