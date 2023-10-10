use crate::ast::Ty;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Ident(String),
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
    pub name: String,
    pub fields: Vec<(String, Expr)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructCall {
    pub name: String,
    pub fields: Vec<(String, Expr)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Inserted {
    pub string_parts: Vec<String>,
    pub inserts: Vec<(usize, Insert)>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Insert {
    For {
        expr: Box<Insert>,
        item: String,
        iter: List,
    },
    Ident(String),
    Fmt {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Lit {
    Str(String),
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
    LitList(Vec<String>),
    Ident(String),
}
