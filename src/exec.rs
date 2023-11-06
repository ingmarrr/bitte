use std::{
    collections::HashMap,
    io::{Error, Write},
};

use crate::{
    ast::{Ast, AstKind, Dir, Expr, File, Let, Lit, Ref, Ty},
    err::ExecErr,
};

pub struct Excecutor;

impl Excecutor {
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

    pub fn dir(syms: &Syms, dir: Dir) -> Result<(), ExecErr> {
        let path = std::path::Path::new(&dir.name);
        println!("Path: {:?}", path);
        if !path.exists() {
            let _ = std::fs::create_dir_all(&dir.name);
        }
        if path.is_file() {
            return Err(Error::new(std::io::ErrorKind::AlreadyExists, "Path is a file").into());
        }
        for child in dir.children {
            match child {
                Ast::File(file) => {
                    let _ = Self::file(syms, dir.name.clone(), file.clone(), Vec::new());
                }
                Ast::Ref(Ref {
                    kind: AstKind::File,
                    name,
                    args,
                }) => {
                    let sym = syms.get(&Key(name.clone(), Scope::Global));
                    if let None = sym {
                        return Err(ExecErr::NotFound(name));
                    }
                    let sym = sym.unwrap();
                    if let Ast::File(file) = &sym.val {
                        let _ = Self::file(syms, dir.name.clone(), file.clone(), args);
                    } else {
                        return Err(ExecErr::InvalidType(
                            sym.kind.to_string(),
                            "file".to_string(),
                        ));
                    }
                }
                Ast::Dir(mut subdir) => {
                    subdir.name = dir.name.join(subdir.name);
                    let _ = Self::dir(&syms, subdir.clone());
                }
                Ast::Ref(Ref {
                    kind: AstKind::Dir,
                    name,
                    ..
                }) => {
                    let sym = syms.get(&Key(name.clone(), Scope::Global));
                    if let None = sym {
                        return Err(ExecErr::NotFound(name));
                    }
                    let sym = sym.cloned().unwrap();
                    if let Ast::Dir(mut subdir) = sym.val {
                        subdir.name = dir.name.join(subdir.name);
                        let _ = Self::dir(&syms, subdir.clone());
                    } else {
                        return Err(ExecErr::InvalidType(
                            sym.kind.to_string(),
                            "dir".to_string(),
                        ));
                    }
                }
                _ => return Err(Error::new(std::io::ErrorKind::InvalidData, "Expected dir").into()),
            }
        }
        Ok(())
    }

    fn stringify(
        syms: &Syms,
        expr: Expr,
        scope_args: Vec<(String, Expr)>,
    ) -> Result<String, ExecErr> {
        let mut buf = String::new();
        match expr {
            Expr::Lit(Lit::String(lit)) => buf.push_str(&lit),
            Expr::Ref(re) => {
                let sym = if let Some(sym) = syms.get(&Key(re.name.clone(), Scope::Global)) {
                    sym
                } else {
                    if let Some((_, expr)) = scope_args
                        .iter()
                        .find(|(name, _)| name == &re.name)
                        .cloned()
                    {
                        return Self::stringify(syms, expr, scope_args);
                    } else {
                        return Err(ExecErr::NotFound(re.name));
                    }
                };

                if re.kind != sym.kind {
                    return Err(ExecErr::InvalidType(re.name, re.kind.to_string()));
                }
                match sym.kind {
                    AstKind::Req => {
                        if let Ast::Req(r) = sym.val.clone() {
                            buf.push_str(&r.expr);
                        } else {
                            return Err(ExecErr::InvalidType(re.name, re.kind.to_string()));
                        }
                    }
                    // AstKind::Lit => {
                    //     if let Ast::Lit(Lit::String(lit)) = sym.val.clone() {
                    //         buf.push_str(&lit);
                    //     } else {
                    //         return Err(ExecErr::InvalidType(re.name, re.kind.to_string()));
                    //     }
                    // }
                    AstKind::Let => {
                        if let Ast::Let(Let { expr, .. }) = sym.val.clone() {
                            buf.push_str(&Self::stringify_vec(syms, expr, scope_args)?);
                        } else {
                            return Err(ExecErr::InvalidType(re.name, re.kind.to_string()));
                        }
                    }
                    _ => return Err(ExecErr::InvalidType(re.name, re.kind.to_string())),
                }
            }
            _ => return Err(ExecErr::InvalidType("".to_string(), "".to_string())),
        }
        Ok(buf)
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
    pub args: Vec<(String, String)>,
    pub symbols: HashMap<Key, Sym>,
}

impl Syms {
    pub fn new(args: Vec<(String, String)>) -> Syms {
        Syms {
            args,
            symbols: HashMap::new(),
        }
    }

    pub fn add(&mut self, sym: Sym) -> Result<(), ExecErr> {
        let key = Key(sym.name.to_owned(), sym.scope.to_owned());
        if self.symbols.contains_key(&key) {
            return Err(ExecErr::AlreadyExists(sym.name));
        }
        self.symbols.insert(key, sym);
        Ok(())
    }

    pub fn get(&self, key: &Key) -> Option<&Sym> {
        self.symbols.get(key)
    }

    pub fn has(&self, key: &Key) -> bool {
        self.symbols.get(key).is_some()
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Key(pub String, pub Scope);

#[derive(Debug, Clone)]
pub struct Sym {
    pub name: String,
    pub ty: Ty,
    pub kind: AstKind,
    pub scope: Scope,
    pub reqs: Vec<(String, Ty)>,
    pub val: Ast,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Scope {
    Global,
    Local(String),
}
