// #[derive(Debug, PartialEq)]
// pub enum Stmt {
//     Decl(Decl),
//     Expr(Expr),
// }

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Ty {
    Str,
    Int,
    List,
    Dir,
    File,
    Unknown,
}

impl From<&str> for Ty {
    fn from(s: &str) -> Self {
        match s {
            "str" => Ty::Str,
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
            Ty::Str => f.write_str("str"),
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
    Lit(Lit),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Ref(Ref),
    Lit(Lit),
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

impl Ast {
    pub fn alias(&self) -> Option<String> {
        match self {
            Ast::Dir(d) => Some(d.alias.clone()),
            Ast::File(f) => Some(f.alias.clone()),
            _ => None,
        }
    }

    pub fn name(&self) -> Option<String> {
        match self {
            Ast::Let(l) => Some(l.name.clone()),
            Ast::Req(r) => Some(r.name.clone()),
            Ast::Dir(d) => Some(d.name.to_str().unwrap().to_string()),
            Ast::File(f) => Some(f.name.clone()),
            Ast::Ref(r) => Some(r.name.clone()),
            Ast::Lit(_) => None,
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
            Ast::Lit(_) => Ty::Str,
        }
    }

    pub fn kind(&self) -> AstKind {
        match self {
            Ast::Let(_) => AstKind::Let,
            Ast::Req(_) => AstKind::Req,
            Ast::Dir(_) => AstKind::Dir,
            Ast::File(_) => AstKind::File,
            Ast::Ref(r) => r.kind.clone(),
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
    pub params: Vec<(String, Ty)>,
    pub name: std::path::PathBuf,
    pub alias: String,
    pub children: Vec<Ast>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct File {
    pub params: Vec<(String, Ty)>,
    pub name: String,
    pub alias: String,
    pub content: Vec<Expr>,
}

impl File {
    pub fn path(&self, parent: std::path::PathBuf) -> std::path::PathBuf {
        parent.join(&self.name)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Let {
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
pub enum Lit {
    String(String),
    File(String, Box<Ast>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ref {
    pub name: String,
    pub kind: AstKind,
    pub args: Vec<(String, Expr)>,
}
