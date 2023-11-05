use std::collections::HashMap;

use crate::{ast::Ty, decl::Decl, err::SemanticError};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Scope {
    Global,
    Local(String),
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Scope::Global => write!(f, "global"),
            Scope::Local(name) => write!(f, "local {}", name),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Requires {
    Some(Vec<(String, Ty)>),
    None,
}

#[derive(Debug)]
pub struct Sym {
    pub name: String,
    pub ty: Ty,
    pub kind: Kind,
    pub scope: Scope,
    pub reqs: Requires,
}

#[derive(Debug)]
pub enum Kind {
    Let,
    Struct,
    Fmt,
    Required,
    Optional,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Key(pub String, pub Scope);

#[derive(Debug)]
pub struct SymTable {
    pub symbols: HashMap<Key, Sym>,
}

impl SymTable {
    pub fn new() -> SymTable {
        SymTable {
            symbols: HashMap::new(),
        }
    }

    pub fn from_decls(decls: &[Decl]) -> (SymTable, Vec<SemanticError>) {
        let mut sym_table = SymTable::new();
        let mut errs = Vec::new();
        for decl in decls.into_iter() {
            let (syms, mut err) = sym_table.sym(decl);
            if let Err(e) = sym_table.insert_all(syms) {
                err.extend(e);
            }
            errs.append(&mut err);
        }
        (sym_table, errs)
    }

    pub fn sym(&self, decl: &Decl) -> (Vec<Sym>, Vec<SemanticError>) {
        let mut syms = Vec::new();
        let mut errs = Vec::new();
        match decl {
            Decl::Let(l) => syms.push(Sym {
                name: l.name.to_owned(),
                scope: Scope::Global,
                ty: l.ty.clone(),
                kind: Kind::Let,
                reqs: Requires::None,
            }),
            Decl::Struct(s) => {
                for (name, ty) in &s.params {
                    if let Some(sym) =
                        self.get(&Key(name.to_owned(), Scope::Local(s.name.to_owned())))
                    {
                        errs.push(SemanticError::AlreadyExists(sym.name.to_string()));
                    };
                    syms.push(Sym {
                        name: name.to_owned(),
                        scope: Scope::Local(s.name.to_owned()),
                        ty: ty.clone(),
                        kind: Kind::Struct,
                        reqs: Requires::None,
                    })
                }
                syms.push(Sym {
                    name: s.name.to_owned(),
                    scope: Scope::Global,
                    ty: Ty::Dir,
                    kind: Kind::Struct,
                    reqs: Requires::Some(s.params.to_owned()),
                })
            }
            Decl::Fmt(f) => {
                for (name, ty) in &f.params {
                    if let Some(sym) =
                        self.get(&Key(name.to_owned(), Scope::Local(f.name.to_owned())))
                    {
                        errs.push(SemanticError::AlreadyExists(sym.name.to_string()));
                    };
                    syms.push(Sym {
                        name: name.to_owned(),
                        scope: Scope::Local(f.name.to_owned()),
                        ty: ty.clone(),
                        kind: Kind::Fmt,
                        reqs: Requires::None,
                    })
                }
                syms.push(Sym {
                    name: f.name.to_owned(),
                    scope: Scope::Global,
                    ty: Ty::Str,
                    kind: Kind::Fmt,
                    reqs: Requires::Some(f.params.to_owned()),
                })
            }
            Decl::Required(r) => syms.push(Sym {
                name: r.name.to_owned(),
                scope: Scope::Global,
                ty: r.ty.clone(),
                kind: Kind::Required,
                reqs: Requires::None,
            }),
            Decl::Optional(o) => syms.push(Sym {
                name: o.name.to_owned(),
                scope: Scope::Global,
                ty: o.ty.clone(),
                kind: Kind::Optional,
                reqs: Requires::None,
            }),
        }
        (syms, errs)
    }

    fn insert(&mut self, symbol: Sym) -> Result<(), SemanticError> {
        if let Some(sym) = self
            .symbols
            .get(&Key(symbol.name.to_owned(), symbol.scope.to_owned()))
        {
            if sym.scope == symbol.scope {
                return Err(SemanticError::AlreadyExists(sym.name.to_string()));
            }
        }
        self.symbols
            .insert(Key(symbol.name.to_owned(), symbol.scope.to_owned()), symbol);
        Ok(())
    }

    pub fn insert_all(&mut self, syms: Vec<Sym>) -> Result<(), Vec<SemanticError>> {
        let mut errs = Vec::new();
        for sym in syms.into_iter() {
            if let Err(e) = self.insert(sym) {
                errs.push(e);
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }

    pub fn get(&self, key: &Key) -> Option<&Sym> {
        self.symbols.get(key)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_sym_table() {
//         let input = r#"let x: str = "hello" let y: list = ["hello", "world"]"#;
//         let lex = crate::lexer::Lexer::new(input.as_bytes());
//         let mut par = crate::parse::Parser::new(lex);
//         let stmts = par.parse().unwrap();
//         let (sym_table, _) = SymTable::from_decls(&stmts);
//         assert_eq!(
//             sym_table
//                 .get(&Key("x".to_owned(), Scope::Global))
//                 .unwrap()
//                 .ty,
//             Ty::Str
//         );
//         assert_eq!(
//             sym_table
//                 .get(&Key("y".to_owned(), Scope::Global))
//                 .unwrap()
//                 .ty,
//             Ty::List
//         );
//     }
// }
