use crate::ast::Ty;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Ident(Box<str>),
    Lit(Lit),
    Fmt(FmtCall),
    Struct(StructCall),
    Inserted(Inserted),
}

impl Expr {
    pub fn ty(&self) -> Ty {
        match self {
            Expr::Ident(_) => Ty::Unknown,
            Expr::Lit(lit) => lit.ty(),
            Expr::Fmt(_) | Expr::Inserted(_) => Ty::Str,
            Expr::Struct(_) => Ty::Struct,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FmtCall {
    pub name: Box<str>,
    pub fields: Vec<(Box<str>, Expr)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructCall {
    pub name: Box<str>,
    pub fields: Vec<(Box<str>, Expr)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Inserted {
    pub string_parts: Vec<Box<str>>,
    pub inserts: Vec<(usize, Insert)>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Insert {
    For {
        expr: Box<Insert>,
        item: Box<str>,
        iter: List,
    },
    Ident(Box<str>),
    Fmt {
        name: Box<str>,
        args: Vec<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Lit {
    Str(Box<str>),
    Char(char),
    List(List),
}

impl Lit {
    pub fn ty(&self) -> Ty {
        match self {
            Lit::Str(_) => Ty::Str,
            Lit::Char(_) => Ty::Char,
            Lit::List(_) => Ty::List,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum List {
    LitList(Vec<Box<str>>),
    Ident(Box<str>),
}
