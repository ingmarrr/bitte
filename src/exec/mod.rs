use std::{
    collections::HashMap,
    io::{Error, Write},
};

use crate::{
    ast::{Ast, AstKind, Dir, Expr, File, Ty},
    err::ExecErr,
};

use self::{executable::Executable, resolve::Resolve};

pub mod executable;
pub mod resolve;

pub struct Exec;

impl Exec {
    pub fn run(syms: &Syms, ast: Ast, args: Vec<(String, Expr)>) -> Result<(), ExecErr> {
        match ast {
            Ast::File(file) => {
                let _ = Self::file(syms, std::path::PathBuf::new(), file, args);
            }
            Ast::Dir(dir) => {
                let _ = Self::dir(syms, dir, args);
            }
            _ => return Err(Error::new(std::io::ErrorKind::InvalidData, "Expected dir").into()),
        }
        Ok(())
    }

    pub fn file(
        syms: &Syms,
        parent: std::path::PathBuf,
        file: File,
        args: Vec<(String, Expr)>,
    ) -> Result<(), ExecErr> {
        let path = file.path(parent.clone());
        if !path.exists() {
            let _ = std::fs::create_dir_all(parent.clone());
        }
        let mut fi = std::fs::File::create(path)?;
        let body = Self::stringify_vec(syms, file.content, args)?;
        let _ = fi.write_all(body.as_bytes());
        Ok(())
    }

    pub fn dir(syms: &Syms, dir: Dir, args: Vec<(String, Expr)>) -> Result<(), ExecErr> {
        dir.execute(syms, "./".into(), args)
    }

    fn stringify(
        syms: &Syms,
        expr: Expr,
        scope_args: Vec<(String, Expr)>,
    ) -> Result<String, ExecErr> {
        expr.resolve::<String>(syms, &Ty::String, scope_args)
    }

    fn stringify_vec(
        syms: &Syms,
        exprs: Vec<Expr>,
        args: Vec<(String, Expr)>,
    ) -> Result<String, ExecErr> {
        let mut buf = String::new();
        for expr in exprs.into_iter() {
            buf.push_str(&Self::stringify(syms, expr, args.clone())?);
        }
        Ok(buf)
    }
}

#[derive(Debug)]
pub struct Syms {
    pub args: Vec<(String, Expr)>,
    pub symbols: HashMap<Key, Sym>,
    pub main: Option<Sym>,
}

impl Syms {
    pub fn new(args: Vec<(String, Expr)>) -> Syms {
        Syms {
            args,
            symbols: HashMap::new(),
            main: None,
        }
    }

    pub fn add(&mut self, sym: Sym) -> Result<(), ExecErr> {
        let key = Key(sym.name(), sym.scope.to_owned());
        if sym.main() {
            if self.main.is_some() {
                return Err(ExecErr::MultipleMain);
            }
            self.main = Some(sym.clone());
        }

        if self.symbols.contains_key(&key) {
            return Err(ExecErr::AlreadyExists(sym.name()));
        }
        self.symbols.insert(key, sym);
        Ok(())
    }

    pub fn add_ast(&mut self, ast: Ast) -> Result<(), ExecErr> {
        self.add(Sym {
            scope: Scope::Global,
            val: ast,
        })
    }

    pub fn add_all_ast(&mut self, asts: Vec<Ast>) -> Result<(), ExecErr> {
        for ast in asts.into_iter() {
            self.add_ast(ast)?;
        }
        Ok(())
    }

    pub fn get(&self, key: &Key) -> Option<&Sym> {
        self.symbols.get(key)
    }

    pub fn has(&self, key: &Key) -> bool {
        self.symbols.get(key).is_some()
    }

    pub fn main(&self) -> Option<Ast> {
        self.main.clone().map(|sym| sym.val)
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Key(pub String, pub Scope);

#[derive(Debug, Clone)]
pub struct Sym {
    pub scope: Scope,
    pub val: Ast,
}

impl Sym {
    pub fn main(&self) -> bool {
        match self.val {
            Ast::Let(ref l) => l.main,
            Ast::Dir(ref d) => d.main,
            Ast::File(ref f) => f.main,
            Ast::Req(_) => false,
            Ast::Ref(_) => false,
            Ast::Lit(_) => false,
        }
    }

    pub fn alias(&self) -> Option<String> {
        match self.val {
            Ast::Dir(ref d) => Some(d.alias.clone()),
            Ast::File(ref f) => Some(f.alias.clone()),
            _ => None,
        }
    }

    pub fn name(&self) -> String {
        match self.val {
            Ast::Let(ref l) => l.name.clone(),
            Ast::Req(ref r) => r.name.clone(),
            Ast::Dir(ref d) => d.path.to_str().unwrap().to_string(),
            Ast::File(ref f) => f.path.clone(),
            Ast::Ref(ref r) => r.name.clone(),
            _ => unreachable!("A Literal cannot be inserted as a symbol."),
        }
    }

    pub fn params(&self) -> Vec<(String, Ty)> {
        match self.val {
            Ast::Let(ref l) => l.params.clone(),
            Ast::Dir(ref d) => d.params.clone(),
            Ast::File(ref f) => f.params.clone(),
            Ast::Req(_) | Ast::Ref(_) | Ast::Lit(_) => Vec::new(),
        }
    }

    pub fn ty(&self) -> Ty {
        match self.val {
            Ast::Let(ref l) => l.ty,
            Ast::Req(ref r) => r.ty,
            Ast::Dir(_) => Ty::Dir,
            Ast::File(_) => Ty::File,
            Ast::Ref(_) => Ty::Unknown,
            Ast::Lit(_) => Ty::String,
        }
    }

    pub fn kind(&self) -> AstKind {
        match self.val {
            Ast::Let(_) => AstKind::Let,
            Ast::Req(_) => AstKind::Req,
            Ast::Dir(_) => AstKind::Dir,
            Ast::File(_) => AstKind::File,
            Ast::Ref(_) => AstKind::Req,
            Ast::Lit(_) => AstKind::Lit,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Scope {
    Global,
    Local(String),
}
