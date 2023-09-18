use crate::token::Tok;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Decl(Decl),
    Expr(Expr),
}

#[derive(Debug, PartialEq)]
pub enum Decl {
    Struct(Struct),
    Let {
        name: String,
        ty: Option<Ty>,
        val: Expr,
    },
    Fmt {
        name: String,
        params: Vec<Param>,
        body: LitStr,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Struct {
    pub name: Box<str>,
    pub children: Vec<Struct>,
    pub files: Vec<Box<str>>,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Ident(String),
    Lit(Lit),
    StructInit { name: String, fields: Vec<Field> },
    Fmt { name: String, args: Vec<Expr> },
}

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub val: Expr,
}

#[derive(Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Ty,
}

#[derive(Debug, PartialEq)]
pub enum Lit {
    // TODO ! :: Add `Insert` to string
    Str(LitStr),
    Char(char),
    Num(String),
    List(Vec<Expr>),
}

#[derive(Debug, PartialEq)]
pub struct LitStr {
    pub body: String,
    pub inserts: Vec<Tok>,
}

#[derive(Debug, PartialEq)]
pub enum Ty {
    Str,
    Char,
    Int,
    List,
    Struct(String),
}
