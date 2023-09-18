#[derive(Debug, PartialEq)]
pub enum Stmt {
    Decl(Decl),
    Expr(Expr),
}

#[derive(Debug, PartialEq)]
pub enum Decl {
    Struct(Struct),
    Let {
        name: Box<str>,
        ty: Option<Ty>,
        expr: Expr,
    },
    Fmt {
        name: String,
        params: Vec<Param>,
        body: Box<str>,
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
    StructInit {
        name: String,
        fields: Vec<Field>,
    },
    Fmt {
        name: String,
        args: Vec<Expr>,
    },
    Inserted {
        string_parts: Vec<Box<str>>,
        inserts: Vec<(usize, Insert)>,
    },
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

#[derive(Debug, PartialEq)]
pub enum Ty {
    Str,
    Char,
    Int,
    List,
    Struct(String),
}
