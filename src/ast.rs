use crate::{decl::Decl, expr::Expr};

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Decl(Decl),
    Expr(Expr),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Ty {
    Str,
    Char,
    Int,
    List,
    Struct,
    Unknown,
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Str => f.write_str("str"),
            Ty::Char => f.write_str("char"),
            Ty::Int => f.write_str("int"),
            Ty::List => f.write_str("list"),
            Ty::Struct => f.write_str("struct"),
            Ty::Unknown => f.write_str("unknown"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Ast {
    Ref(AstKind, String),
    Let(Let),
    Dir(Dir),
    File(File),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AstKind {
    Let,
    Dir,
    File,
}

impl Ast {
    pub fn name(&self) -> String {
        match self {
            Ast::Let(l) => l.name.clone(),
            Ast::Dir(d) => d.name.to_str().unwrap().to_string(),
            Ast::File(f) => f.name.clone(),
            Ast::Ref(_, s) => s.clone(),
        }
    }

    pub fn ty(&self) -> Ty {
        match self {
            Ast::Let(l) => l.ty.clone(),
            Ast::Dir(_) => Ty::Struct,
            Ast::File(_) => Ty::Str,
            Ast::Ref(_, _) => Ty::Unknown,
        }
    }

    pub fn kind(&self) -> AstKind {
        match self {
            Ast::Let(_) => AstKind::Let,
            Ast::Dir(_) => AstKind::Dir,
            Ast::File(_) => AstKind::File,
            Ast::Ref(k, _) => AstKind::from(k),
        }
    }
}

impl From<&AstKind> for AstKind {
    fn from(k: &AstKind) -> Self {
        match k {
            AstKind::Let => AstKind::Let,
            AstKind::Dir => AstKind::Dir,
            AstKind::File => AstKind::File,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Dir {
    pub name: std::path::PathBuf,
    pub children: Vec<Ast>,
    pub files: Vec<Ast>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct File {
    pub name: String,
    pub content: String,
}

impl File {
    pub fn path(&self, parent: std::path::PathBuf) -> std::path::PathBuf {
        parent.join(&self.name)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Let {
    pub name: String,
    pub ty: Ty,
    pub expr: String,
}
