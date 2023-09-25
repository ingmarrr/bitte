use std::collections::HashMap;

use crate::{ast::Ty, decl::Decl, err::SemanticError};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Scope {
    Global,
    Local(Box<str>),
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Scope::Global => write!(f, "global"),
            Scope::Local(name) => write!(f, "local {}", name),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Req {
    Some(Vec<(Box<str>, Ty)>),
    None,
}

#[derive(Debug)]
pub struct Sym {
    pub name: Box<str>,
    pub ty: Ty,
    pub scope: Scope,
    pub reqs: Req,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Key(pub Box<str>, pub Scope);

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
            let res = match decl {
                Decl::Let(l) => {
                    sym_table.insert(l.name.to_owned(), Scope::Global, l.ty.clone(), Req::None)
                }
                Decl::Struct(s) => {
                    for (name, ty) in &s.params {
                        if let Err(err) = sym_table.insert(
                            name.to_owned(),
                            Scope::Local(s.name.to_owned()),
                            ty.clone(),
                            Req::None,
                        ) {
                            errs.push(err);
                        };
                    }
                    sym_table.insert(
                        s.name.to_owned(),
                        Scope::Global,
                        Ty::Struct,
                        Req::Some(s.params.to_owned()),
                    )
                }
                Decl::Fmt(f) => {
                    for (name, ty) in &f.params {
                        if let Err(err) = sym_table.insert(
                            name.to_owned(),
                            Scope::Local(f.name.to_owned()),
                            ty.clone(),
                            Req::None,
                        ) {
                            errs.push(err);
                        };
                    }
                    sym_table.insert(
                        f.name.to_owned(),
                        Scope::Global,
                        Ty::Str,
                        Req::Some(f.params.to_owned()),
                    )
                }
                Decl::Required(r) => {
                    sym_table.insert(r.name.to_owned(), Scope::Global, r.ty.clone(), Req::None)
                }
                Decl::Optional(o) => {
                    sym_table.insert(o.name.to_owned(), Scope::Global, o.ty.clone(), Req::None)
                }
            };
            if let Err(err) = res {
                errs.push(err);
            }
        }
        (sym_table, errs)
    }

    fn insert(
        &mut self,
        name: Box<str>,
        scope: Scope,
        ty: Ty,
        reqs: Req,
    ) -> Result<(), SemanticError> {
        if let Some(sym) = self.symbols.get(&Key(name.to_owned(), scope.to_owned())) {
            if sym.scope == scope {
                return Err(SemanticError::AlreadyExists(sym.name.to_string()));
            }
        }
        self.symbols.insert(
            Key(name.to_owned(), scope.to_owned()),
            Sym {
                name,
                ty,
                scope,
                reqs,
            },
        );
        Ok(())
    }

    pub fn get(&self, key: &Key) -> Option<&Sym> {
        self.symbols.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sym_table() {
        let input = r#"let x: str = "hello" let y: list = ["hello", "world"]"#;
        let lex = crate::lex::Lexer::new(input);
        let mut par = crate::parse::Parser::new(lex);
        let stmts = par.parse().unwrap();
        let (sym_table, errs) = SymTable::from_decls(&stmts);
        assert_eq!(
            sym_table
                .get(&Key("x".to_owned().into_boxed_str(), Scope::Global))
                .unwrap()
                .ty,
            Ty::Str
        );
        assert_eq!(
            sym_table
                .get(&Key("y".to_owned().into_boxed_str(), Scope::Global))
                .unwrap()
                .ty,
            Ty::List
        );
    }
}
