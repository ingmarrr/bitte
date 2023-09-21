use std::collections::HashMap;

use crate::{
    ast::{Decl, Expr, Fmt, Insert, Lit, Stmt, Struct, Ty},
    err::SemanticError,
    sym::{Sym, SymTable, Scope},
};

pub struct Analyzer {
    stmts: Vec<Stmt>,
    errs: Vec<SemanticError>,
    sym_table: SymTable,
    requireds: Vec<(String, String)>,
    optionals: Vec<(String, String)>,
    main_struct: Option<Struct>,
    lets: Vec<(String, String)>,
    structs: Vec<Struct>,
    fmts: Vec<Fmt>,
}

impl Analyzer {
    pub fn new(stmts: Vec<Stmt>) -> Analyzer {
        let (sym_table, errs) = SymTable::from_stmts(&stmts);
        Analyzer {
            stmts,
            errs,
            sym_table,
            requireds: Vec::new(),
            optionals: Vec::new(),
            main_struct: None,
            lets: Vec::new(),
            structs: Vec::new(),
            fmts: Vec::new(),
        }
    }

    fn analyze_decl(&mut self, decl: Decl) -> Result<(), SemanticError> {
        match decl {
            Decl::Struct(s) => self.analyze_struct(s)?,
            Decl::Fmt(f) => self.analyze_fmt(f)?,
            Decl::Let { name, ty, expr } => self.analyze_let(name, ty, expr)?,
            Decl::Required { name, ty } => self.analyze_required(name, ty)?,
        }
        Ok(())
    }

    fn analyze_struct(&mut self, s: Struct) -> Result<(), SemanticError> {
        for (_, expr) in &s.files {
            match expr {
                Some(expr) => self.analyze_str_expr(expr)?,
                None => {}
            }
        }

        self.structs.push(s);
        Ok(())
    }

    fn analyze_fmt(&mut self, f: Fmt) -> Result<(), SemanticError> {
        for (_, ins) in &f.inserts {
            self.analyze_insert(ins)?;
        }
        self.fmts.push(f);
        Ok(())
    }

    fn analyze_insert(&self, ins: &Insert) -> Result<(), SemanticError> {
        match ins {
            Insert::For { expr, item, iter } => todo!(),
            Insert::Ident(i) => match self.sym_table.get(&i) {
                Some(sym) => match sym.ty {
                    crate::ast::Ty::Str => Ok(()),
                    ty => Err(SemanticError::InvalidType(ty.to_string())),
                },
                None => Err(SemanticError::UnknownType(i.as_ref().to_owned())),
            },
            Insert::Fmt { name, args } => match self.sym_table.get(&name, args) {
                Some(sym) => match sym.ty {
                    crate::ast::Ty::Str => {
                        for arg in args {
                            match arg {
                                Expr::Ident(ident) => self.analyze_ident(ident, Ty::Str)?,
                                Expr::Fmt { name, args } => 
                                Expr::Inserted { inserts, .. } => todo!(),
                            }
                        }
                        Ok(())
                    }
                    ty => Err(SemanticError::InvalidType(ty.to_string())),
                },
                None => Err(SemanticError::UnknownFmt(name.as_ref().to_owned())),
            },
        }
    }

    fn analyze_fmt(&self, ident: &Box<str>, args: &Vec<Expr>) -> Result<(), SemanticError> {
        match self.sym_table.get(ident) {
            Some(sym) => match sym.ty {
                crate::ast::Ty::Str => {
                    for arg in args {
                        match arg {
                            Expr::Ident(ident) => self.analyze_ident(ident, Ty::Str, sym.scope)?,
                            Expr::Fmt { name, args: fmt_args } => self.analyze_fmt(name, fmt_args)
                            Expr::Inserted { inserts, .. } => todo!(),
                        }
                    }
                },
                ty => Err(SemanticError::InvalidType(ty.to_string())),
            },
            None => Err(SemanticError::UnknownFmt(ident.as_ref().to_owned())),
        }
    }

    fn analyze_ident(&self, ident: &Box<str>, ty: Ty, scope: Scope) -> Result<(), SemanticError> {
        match self.sym_table.get(ident) {
            Some(sym) => match sym {
                Sym { ty: sym_ty, scope: sym_scope, .. } => {
                    if sym_ty == &ty && (sym_scope == &scope || sym_scope == &Scope::Global) {
                        match sym_scope {
                            Scope::Global => Ok(()),
                            Scope::Local(local) => match local == scope {
                                true => Ok(()),
                                false => Err(SemanticError::OutOfScope(
                                    ident.as_ref().to_owned(),
                                    scope.to_string(),
                                    local.to_string(),
                                )),
                            }
                        }
                    } else {
                        Err(SemanticError::InvalidType(ty.to_string()))
                    }
                }
            },
            None => Err(SemanticError::UnknownType(ident.as_ref().to_owned())),
        }
    }

    fn analyze_str_expr(&self, expr: Expr) -> Result<(), SemanticError> {
        match expr {
            Expr::Ident(i) => match self.sym_table.get(&i) {
                Some(sym) => match sym.ty {
                    crate::ast::Ty::Str => Ok(()),
                    ty => Err(SemanticError::InvalidType(ty.to_string())),
                },
                None => Err(SemanticError::UnknownType(i.as_ref().to_owned())),
            },
            Expr::Lit(lit) => match lit {
                Lit::Str(_) | Lit::Char(_) | Lit::Num(_) => Ok(()),
                Lit::List(_) => Err(SemanticError::InvalidType("list".to_owned())),
            },
            Expr::Fmt { name, args } => match self.sym_table.get(&name) {
                Some(sym) => match sym.ty {
                    crate::ast::Ty::Str => {
                        for arg in args {
                            self.analyze_expr(arg)?;
                        }
                        Ok(())
                    }
                    ty => Err(SemanticError::InvalidType(ty.to_string())),
                },
                None => Err(SemanticError::UnknownFmt(name.as_ref().to_owned())),
            },
            Expr::Inserted { inserts, .. } => {
                for (_, ins) in inserts {
                    self.analyze_insert(&ins)?;
                }
                Ok(())
            }
        }
    }
}
