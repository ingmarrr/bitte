use crate::err::ExecErr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Ty {
    String,
    Int,
    List,
    Dir,
    File,
    Unknown,
}

impl From<&str> for Ty {
    fn from(s: &str) -> Self {
        match s {
            "str" => Ty::String,
            "int" => Ty::Int,
            "list" => Ty::List,
            "dir" => Ty::Dir,
            "file" => Ty::File,
            _ => Ty::Unknown,
        }
    }
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::String => f.write_str("str"),
            Ty::Int => f.write_str("int"),
            Ty::List => f.write_str("list"),
            Ty::Dir => f.write_str("dir"),
            Ty::File => f.write_str("file"),
            Ty::Unknown => f.write_str("unknown"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Ast {
    Ref(Ref),
    Req(Req),
    Let(Let),
    Dir(Dir),
    File(File),
    Lit(String),
}

impl Ast {
    pub fn set_main(&mut self) {
        match self {
            Ast::Let(l) => l.main = true,
            Ast::Dir(d) => d.main = true,
            Ast::File(f) => f.main = true,
            _ => {}
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AstKind {
    Let,
    Req,
    Dir,
    File,
    Lit,
}

impl std::fmt::Display for AstKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstKind::Let => f.write_str("let"),
            AstKind::Req => f.write_str("req"),
            AstKind::Dir => f.write_str("dir"),
            AstKind::File => f.write_str("file"),
            AstKind::Lit => f.write_str("lit"),
        }
    }
}

#[cfg(test)]
impl Ast {
    pub fn main(&self) -> bool {
        match self {
            Ast::Let(l) => l.main,
            Ast::Dir(d) => d.main,
            Ast::File(f) => f.main,
            Ast::Req(_) => false,
            Ast::Ref(_) => false,
            Ast::Lit(_) => false,
        }
    }

    pub fn alias(&self) -> Option<String> {
        match self {
            Ast::Dir(d) => Some(d.alias.clone()),
            Ast::File(f) => Some(f.alias.clone()),
            _ => None,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Ast::Let(l) => l.name.clone(),
            Ast::Req(r) => r.name.clone(),
            Ast::Dir(d) => d.path.to_str().unwrap().to_string(),
            Ast::File(f) => f.path.clone(),
            Ast::Ref(r) => r.name.clone(),
            _ => unreachable!("A Literal cannot be inserted as a symbol."),
        }
    }

    pub fn params(&self) -> Vec<(String, Ty)> {
        match self {
            Ast::Let(l) => l.params.clone(),
            Ast::Dir(d) => d.params.clone(),
            Ast::File(f) => f.params.clone(),
            Ast::Req(_) | Ast::Ref(_) | Ast::Lit(_) => Vec::new(),
        }
    }

    pub fn ty(&self) -> Ty {
        match self {
            Ast::Let(l) => l.ty,
            Ast::Req(r) => r.ty,
            Ast::Dir(_) => Ty::Dir,
            Ast::File(_) => Ty::File,
            Ast::Ref(_) => Ty::Unknown,
            Ast::Lit(_) => Ty::String,
        }
    }

    pub fn kind(&self) -> AstKind {
        match self {
            Ast::Let(_) => AstKind::Let,
            Ast::Req(_) => AstKind::Req,
            Ast::Dir(_) => AstKind::Dir,
            Ast::File(_) => AstKind::File,
            Ast::Ref(_) => AstKind::Req,
            Ast::Lit(_) => AstKind::Lit,
        }
    }
}

impl From<&AstKind> for AstKind {
    fn from(k: &AstKind) -> Self {
        match k {
            AstKind::Let => AstKind::Let,
            AstKind::Req => AstKind::Req,
            AstKind::Dir => AstKind::Dir,
            AstKind::File => AstKind::File,
            AstKind::Lit => AstKind::Lit,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Dir {
    pub main: bool,
    pub params: Vec<(String, Ty)>,
    pub path: std::path::PathBuf,
    pub alias: String,
    pub children: Vec<Expr>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct File {
    pub main: bool,
    pub params: Vec<(String, Ty)>,
    pub path: String,
    pub alias: String,
    pub content: Vec<Expr>,
}

impl File {
    pub fn path(&self, parent: std::path::PathBuf) -> std::path::PathBuf {
        parent.join(&self.path)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Let {
    pub main: bool,
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub ty: Ty,
    pub expr: Vec<Expr>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Req {
    pub name: String,
    pub ty: Ty,
    pub expr: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Ref(Ref),
    Lit(Lit),
    If(If),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ref {
    pub name: String,
    pub args: Vec<(String, Expr)>,
    pub ty: Ty,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct If {
    pub cond: BinOp,
    pub then: Vec<Expr>,
    pub els: Vec<Expr>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BinOp {
    pub op: Op,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Op {
    Add,
    Mul,
    Eq,
    Neq,
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Op::*;
        match self {
            Add => f.write_str("+"),
            Mul => f.write_str("*"),
            Eq => f.write_str("=="),
            Neq => f.write_str("!="),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Lit {
    Int(String),
    String(String),
    Dir(Dir),
    File(File),
    BinOp(BinOp),
}

impl TryFrom<Lit> for String {
    type Error = ExecErr;

    fn try_from(lit: Lit) -> Result<Self, Self::Error> {
        match lit {
            Lit::String(s) => Ok(s),
            _ => Err(ExecErr::InvalidType("".into(), "".into())),
        }
    }
}

impl TryFrom<Lit> for File {
    type Error = ExecErr;

    fn try_from(lit: Lit) -> Result<Self, Self::Error> {
        match lit {
            Lit::File(f) => Ok(f),
            _ => Err(ExecErr::InvalidType("".into(), "".into())),
        }
    }
}

impl TryFrom<Lit> for Dir {
    type Error = ExecErr;

    fn try_from(lit: Lit) -> Result<Self, Self::Error> {
        match lit {
            Lit::Dir(d) => Ok(d),
            _ => Err(ExecErr::InvalidType("".into(), "".into())),
        }
    }
}

impl TryFrom<Lit> for bool {
    type Error = ExecErr;

    fn try_from(lit: Lit) -> Result<Self, Self::Error> {
        match lit {
            Lit::BinOp(bin) => match bin.op {
                Op::Eq => Ok(bin.lhs == bin.rhs),
                Op::Neq => Ok(bin.lhs != bin.rhs),
                _ => Err(ExecErr::InvalidType("".into(), "".into())),
            },
            _ => Err(ExecErr::InvalidType("".into(), "".into())),
        }
    }
}

pub enum LitExecutable {
    String(String),
    File(File),
    Dir(Dir),
}

impl TryFrom<Lit> for LitExecutable {
    type Error = ExecErr;

    fn try_from(lit: Lit) -> Result<Self, Self::Error> {
        match lit {
            Lit::String(s) => Ok(LitExecutable::String(s)),
            Lit::File(f) => Ok(LitExecutable::File(f)),
            Lit::Dir(d) => Ok(LitExecutable::Dir(d)),
            l => Err(ExecErr::InvalidExecutable),
        }
    }
}
