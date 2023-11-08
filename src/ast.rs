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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Ref(Ref),
    Lit(String),
}

impl Expr {
    pub fn dump(&self) -> String {
        match self {
            Expr::Ref(r) => r.dump(),
            Expr::Lit(l) => l.clone(),
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

impl Ast {
    pub fn dump(&self) -> String {
        match self {
            Ast::Let(l) => {
                let mut buf: String = "let".into();
                buf.push_str(&format!(" {} ", l.name));
                for (name, ty) in l.params.iter() {
                    buf.push_str(&format!("{}: {}, ", name, ty));
                }
                buf.push_str(&format!(":{} ", l.ty));
                for expr in l.expr.iter() {
                    match expr {
                        Expr::Lit(lit) => {
                            buf.push_str("{\"");
                            buf.push_str(&lit);
                            buf.push_str("\"}");
                        }
                        Expr::Ref(re) => {
                            buf.push_str("{$");
                            buf.push_str(&re.dump());
                            buf.push_str("$}");
                        }
                    }
                }
                buf
            }
            Ast::Dir(d) => {
                let mut buf: String = "dir".into();
                buf.push_str(&format!(" {}(", d.alias));
                for (name, ty) in d.params.iter() {
                    buf.push_str(&format!("{}: {}, ", name, ty));
                }
                buf.push_str(&format!(
                    "): \"{}\" {{\n",
                    d.path.file_name().unwrap().to_str().unwrap()
                ));
                for child in d.children.iter() {
                    buf.push_str(&child.dump());
                    buf.push_str(",\n")
                }
                buf.push_str("}");
                buf
            }
            Ast::File(f) => {
                let mut buf: String = "file".into();
                buf.push_str(&format!(" {}(", f.alias));
                for (name, ty) in f.params.iter() {
                    buf.push_str(&format!("{}: {}, ", name, ty));
                }
                buf.push_str(&format!("): \"{}\"", f.name));
                for expr in f.content.iter() {
                    match expr {
                        Expr::Lit(lit) => {
                            buf.push_str("{\"");
                            buf.push_str(&lit);
                            buf.push_str("\"}");
                        }
                        Expr::Ref(re) => {
                            buf.push_str("{$");
                            buf.push_str(&re.dump());
                            buf.push_str("$}");
                        }
                    }
                }
                buf
            }
            Ast::Lit(lit) => lit.clone(),
            Ast::Ref(re) => re.dump(),
            Ast::Req(req) => {
                let mut buf: String = "req".into();
                buf.push_str(&format!(" {}:", req.name));
                buf.push_str(&format!("{};", req.ty));
                buf
            }
        }
    }

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

    pub fn name(&self) -> Option<String> {
        match self {
            Ast::Let(l) => Some(l.name.clone()),
            Ast::Req(r) => Some(r.name.clone()),
            Ast::Dir(d) => Some(d.path.to_str().unwrap().to_string()),
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
            Ast::Lit(_) => Ty::String,
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
    pub main: bool,
    pub params: Vec<(String, Ty)>,
    pub path: std::path::PathBuf,
    pub alias: String,
    pub children: Vec<Ast>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct File {
    pub main: bool,
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
pub struct Ref {
    pub name: String,
    pub kind: AstKind,
    pub args: Vec<(String, Expr)>,
    pub ty: Ty,
}

impl Ref {
    pub fn dump(&self) -> String {
        let mut buf: String = match self.kind {
            AstKind::Dir => "@".into(),
            AstKind::File => "#".into(),
            AstKind::Let => "!".into(),
            AstKind::Req => "!".into(),
            AstKind::Lit => unreachable!(),
        };
        buf.push_str(&self.name);
        buf.push_str("(");
        for (name, expr) in self.args.iter() {
            buf.push_str(&format!("{}: {}", name, expr.dump()));
        }
        buf.push_str(")");

        buf
    }
}
