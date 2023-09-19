use std::collections::HashMap;

use crate::ast::{Decl, Stmt, Ty};

pub struct Sym {
    pub ty: Ty,
    pub name: Box<str>,
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

    pub fn from_stmts(stmts: &[Stmt]) -> SymTable {
        let mut sym_table = SymTable::new();
        for stmt in stmts.into_iter() {
            if let Stmt::Decl(decl) = stmt {
                match decl {
                    Decl::Let { name, ty, .. } => sym_table.insert(name.to_owned(), ty.clone()),
                    Decl::Struct(s) => sym_table.insert(s.name.to_owned(), Ty::Struct),
                    Decl::Fmt(f) => sym_table.insert(f.name.clone(), Ty::Str),
                    Decl::Required { name, ty } => sym_table.insert(name.to_owned(), ty.clone()),
                }
            }
        }
        sym_table
    }

    fn insert(&mut self, name: Box<str>, ty: Ty) {
        self.symbols.insert(name.clone(), Sym { ty, name });
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
        let sym_table = SymTable::from_stmts(&stmts);
        assert_eq!(sym_table.get("x").unwrap().ty, Ty::Str);
        assert_eq!(sym_table.get("y").unwrap().ty, Ty::List);
    }
}
