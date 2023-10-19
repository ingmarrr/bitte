use std::{
    collections::HashMap,
    io::{Error, Write},
    ops::Deref,
};

use crate::{
    ast::{Ast, AstKind, Dir, File, Ty},
    err::ExecErr,
    sym::Req,
};

pub struct Excecuter;

impl Excecuter {
    pub fn file(syms: &Syms, parent: std::path::PathBuf, file: File) -> Result<(), ExecErr> {
        let path = file.path(parent.clone());
        if !path.exists() {
            let _ = std::fs::create_dir_all(parent.clone());
        }
        let fi = std::fs::File::create(path);
        match fi {
            Ok(mut f) => match *file.content {
                Ast::Lit(body) => {
                    let _ = f.write_all(body.as_bytes());
                    return Ok(());
                }
                Ast::Ref(AstKind::Let, name) => {
                    let sym = syms.get(&Key(name.clone(), Scope::Global));
                    if let None = sym {
                        return Err(ExecErr::NotFound(name));
                    }
                    let sym = sym.unwrap();
                    if let Ast::Let(l) = &sym.val {
                        let _ = f.write_all(l.expr.as_bytes());
                        return Ok(());
                    } else {
                        return Err(ExecErr::InvalidType(name, Ty::Str.to_string()));
                    }
                }
                _ => return Err(Error::new(std::io::ErrorKind::InvalidData, "Expected lit").into()),
            },
            Err(err) => Err(err.into()),
        }
    }

    pub fn dir(syms: &Syms, dir: Dir) -> Result<(), ExecErr> {
        let path = std::path::Path::new(&dir.name);
        println!("{:?}", path);
        if !path.exists() {
            let _ = std::fs::create_dir_all(&dir.name);
        }
        if path.is_file() {
            return Err(Error::new(std::io::ErrorKind::AlreadyExists, "Path is a file").into());
        }
        for file in dir.files {
            println!("{:?}", file);
            match file {
                Ast::File(file) => {
                    let _ = Self::file(syms, dir.name.clone(), file.clone());
                }
                Ast::Ref(AstKind::File, name) => {
                    let sym = syms.get(&Key(name.clone(), Scope::Global));
                    if let None = sym {
                        return Err(ExecErr::NotFound(name));
                    }
                    let sym = sym.unwrap();
                    if let Ast::File(file) = &sym.val {
                        let _ = Self::file(syms, dir.name.clone(), file.clone());
                    } else {
                        return Err(ExecErr::NotFound(name));
                    }
                }
                _ => {
                    return Err(Error::new(std::io::ErrorKind::InvalidData, "Expected file").into())
                }
            }
        }
        for child in dir.children {
            match child {
                Ast::Dir(mut dir) => {
                    dir.name = dir.name.strip_prefix(&dir.name).unwrap().to_path_buf();
                    let _ = Self::dir(&syms, dir.clone());
                }
                Ast::Ref(AstKind::Dir, name) => {
                    let sym = syms.get(&Key(name.clone(), Scope::Global));
                    if let None = sym {
                        return Err(ExecErr::NotFound(name));
                    }
                    let sym = sym.unwrap();
                    if let Ast::Dir(dir) = &sym.val {
                        let _ = Self::dir(&syms, dir.clone());
                    } else {
                        return Err(ExecErr::NotFound(name));
                    }
                }
                _ => return Err(Error::new(std::io::ErrorKind::InvalidData, "Expected dir").into()),
            }
        }
        Ok(())
    }
}

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

#[derive(Debug)]
pub struct Sym {
    pub name: String,
    pub ty: Ty,
    pub kind: AstKind,
    pub scope: Scope,
    pub reqs: Req,
    pub val: Ast,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Scope {
    Global,
    Local(String),
}
