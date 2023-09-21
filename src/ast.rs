#[derive(Debug, PartialEq)]
pub enum Stmt {
    Decl(Decl),
    Expr(Expr),
}

#[derive(Debug, PartialEq)]
pub enum Decl {
    Struct(Struct),
    Let { name: Box<str>, ty: Ty, expr: Expr },
    Fmt(Fmt),
    Required { name: Box<str>, ty: Ty },
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Ident(Box<str>),
    Lit(Lit),
    Fmt {
        name: Box<str>,
        args: Vec<Expr>,
    },
    Inserted {
        string_parts: Vec<Box<str>>,
        inserts: Vec<(usize, Insert)>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Fmt {
    pub name: Box<str>,
    pub params: Vec<(Box<str>, Ty)>,
    pub string_parts: Vec<Box<str>>,
    pub inserts: Vec<(usize, Insert)>,
}

#[derive(Debug, PartialEq)]
pub struct Struct {
    pub name: Box<str>,
    pub params: Vec<(Box<str>, Ty)>,
    pub children: Vec<Struct>,
    pub files: Vec<(Box<str>, Option<Expr>)>,
}

#[derive(Debug, PartialEq)]
pub enum Lit {
    Str(Box<str>),
    Char(char),
    Num(String),
    List(List),
}

#[derive(Debug, PartialEq, Eq)]
pub enum List {
    LitList(Vec<Box<str>>),
    Ident(Box<str>),
}

#[derive(Debug, PartialEq)]
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
pub enum Ty {
    Str,
    Char,
    Int,
    List,
    Struct,
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Str => f.write_str("str"),
            Ty::Char => f.write_str("char"),
            Ty::Int => f.write_str("int"),
            Ty::List => f.write_str("list"),
            Ty::Struct => f.write_str("struct"),
        }
    }
}
