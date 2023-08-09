
use crate::lexer::Lexer;
use crate::token::Token;

#[test]
fn test_injectors() {
    let ijs = "!#$";
    let mut lx = Lexer::new(ijs);

    let toks = vec![
        Token::Bang,
        Token::Pound,
        Token::Dollar,
    ];

    for tok in toks {
        let tk = lx.next_token();
        assert_eq!(tok, tk);
    }
}

#[test]
fn test_ident_alpha() {
    let cons = "abc";
    let toks = Lexer::new(cons).lex();
    assert_eq!(toks, vec![Token::Ident("abc".into())]);
}

#[test]
fn test_cons_alphanum() {
    let cons = "abc123";
    let toks = Lexer::new(cons).lex();
    assert_eq!(toks, vec![Token::Ident("abc123".into())]);
    
}

#[test]
fn test_string() {
    let cons = "'hello'";
    let toks = Lexer::new(cons).lex();
    assert_eq!(toks, vec![Token::String("hello".into())]);
}

#[test]
fn test_empty_string() {
    let cons = "''";
    let toks = Lexer::new(cons).lex();
    assert_eq!(toks, vec![Token::String("".into())]);
}


#[test]
fn test_string_with_injector() {
    let cons = "'hello $ world'";
    let toks = Lexer::new(cons).lex();
    assert_eq!(toks, vec![Token::String("hello $ world".into())])
}

#[test]
fn test_lex_let() {
    let cons = "let hello = 'hello world'";
    let toks = Lexer::new(cons).lex();
    assert_eq!(toks, vec![
        Token::Ident("let".into()),
        Token::Ident("hello".into()),
        Token::Eq,
        Token::String("hello world".into()),
    ]);
}


#[test]
fn test_lex_chars() {
    let cons = "={}[]:,''!@#$\\";
    let toks = Lexer::new(cons).lex();
    assert_eq!(toks, vec![
        Token::Eq,
        Token::LCurly,
        Token::RCurly,
        Token::LSquare,
        Token::RSquare,
        Token::Colon,
        Token::Comma,
        Token::String("".into()),
        Token::Bang,
        Token::At,
        Token::Pound,
        Token::Dollar,
        Token::Escape,
    ])
}


