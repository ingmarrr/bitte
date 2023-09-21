use std::collections::HashMap;

use crate::{
    ast::{Decl, Stmt, Ty},
    err::SemanticError,
};

#[derive(PartialEq)]
pub enum Scope {
    Global,
    Local(Box<str>),
}

#[derive(PartialEq)]
pub enum Req {
    Some(Vec<(Box<str>, Ty)>),
    None,
}

pub struct Sym {
    pub name: Box<str>,
    pub ty: Ty,
    pub scope: Scope,
    pub reqs: Vec<(Box<str>, Ty)>,
}

pub struct SymTable {
    pub symbols: HashMap<Box<str>, Sym>,
}

impl SymTable {
    pub fn new() -> SymTable {
        SymTable {
            symbols: HashMap::new(),
        }
    }

    pub fn from_stmts(stmts: &[Stmt]) -> (SymTable, Vec<SemanticError>) {
        let mut sym_table = SymTable::new();
        let mut errs = Vec::new();
        for stmt in stmts.into_iter() {
            if let Stmt::Decl(decl) = stmt {
                let res = match decl {
                    Decl::Let { name, ty, .. } => {
                        sym_table.insert(name.to_owned(), Scope::Global, ty.clone(), Req::None)
                    }
                    Decl::Struct(s) => {
                        for (name, ty) in &s.params {
                            if let Err(err) = sym_table.insert(
                                name.to_owned(),
                                Scope::Local(s.name.clone()),
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
                            Req::Some(s.params),
                        )
                    }
                    Decl::Fmt(f) => {
                        for (name, ty) in &f.params {
                            if let Err(err) = sym_table.insert(
                                name.to_owned(),
                                Scope::Local(f.name.clone()),
                                ty.clone(),
                                Req::None,
                            ) {
                                errs.push(err);
                            };
                        }
                        sym_table.insert(
                            f.name.clone(),
                            Scope::Global,
                            Ty::Str,
                            Req::Some(f.params),
                        )
                    }
                    Decl::Required { name, ty } => {
                        sym_table.insert(name.to_owned(), Scope::Global, ty.clone(), Req::None)
                    }
                };
                if let Err(err) = res {
                    errs.push(err);
                }
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
        if let Some(sym) = self.symbols.get(&name) {
            if sym.scope == scope {
                return Err(SemanticError::AlreadyExists(sym.name.to_string()));
            }
        }
        self.symbols.insert(
            name.clone(),
            Sym {
                name,
                ty,
                scope,
                reqs,
            },
        );
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Sym> {
        self.symbols.get(name)
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
        let (sym_table, errs) = SymTable::from_stmts(&stmts);
        assert_eq!(sym_table.get("x").unwrap().ty, Ty::Str);
        assert_eq!(sym_table.get("y").unwrap().ty, Ty::List);
    }
}
