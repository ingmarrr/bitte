use crate::{
    ast::Ty,
    decl::{Decl, Fmt, Let, Optional, Required, Struct},
    err::SemanticError,
    expr::{Expr, FmtCall, Insert, Inserted, Lit},
    info,
    sym::{Key, Scope, Sym, SymTable},
};

#[derive(Debug)]
pub struct Analyzer {
    errs: Vec<SemanticError>,
    sym_table: SymTable,
    requireds: Vec<(Box<str>, Ty)>,
    optionals: Vec<(Box<str>, Ty, Expr)>,
    main_struct: Option<Struct>,
    lets: Vec<(String, String)>,
    structs: Vec<Struct>,
    fmts: Vec<Fmt>,
    allow_requireds: bool,
}

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer {
            errs: Vec::new(),
            sym_table: SymTable::new(),
            requireds: Vec::new(),
            optionals: Vec::new(),
            main_struct: None,
            lets: Vec::new(),
            structs: Vec::new(),
            fmts: Vec::new(),
            allow_requireds: true,
        }
    }

    pub fn analyze(&mut self, decls: Vec<Decl>) -> Result<(), Vec<SemanticError>> {
        let (sym_table, errs) = SymTable::from_decls(&decls);
        self.sym_table = sym_table;
        self.errs = errs;
        for decl in decls {
            if !decl.is_req() {
                self.allow_requireds = false;
            }
            if let Err(err) = self.analyze_decl(decl) {
                self.errs.push(err);
            }
        }
        if self.errs.is_empty() {
            Ok(())
        } else {
            Err(self.errs.clone())
        }
    }

    fn analyze_decl(&mut self, decl: Decl) -> Result<(), SemanticError> {
        match decl {
            Decl::Struct(s) => self.analyze_struct(s)?,
            Decl::Fmt(f) => self.analyze_fmt(f)?,
            Decl::Let(l) => self.analyze_let(l)?,
            Decl::Required(Required { name, ty }) => self.analyze_required(name, ty)?,
            Decl::Optional(Optional { name, ty, default }) => {
                self.analyze_optional(name, ty, default)?
            }
        }
        Ok(())
    }

    fn analyze_fmt(&mut self, fmt: Fmt) -> Result<(), SemanticError> {
        for (_, expr) in fmt.inserts.clone() {
            self.analyze_insert_expr(expr, Scope::Global)?;
        }
        self.fmts.push(fmt);
        Ok(())
    }

    fn analyze_let(&mut self, let_decl: Let) -> Result<(), SemanticError> {
        self.analyze_expr(let_decl.expr, let_decl.ty, Scope::Global)?;
        self.lets
            .push((let_decl.name.to_string(), let_decl.ty.to_string()));
        Ok(())
    }

    fn analyze_struct(&mut self, s: Struct) -> Result<(), SemanticError> {
        for (_, expr) in s.files.clone() {
            match expr {
                Some(expr) => self.analyze_str_expr(expr, Scope::Local(s.name.to_owned()))?,
                None => {}
            }
        }

        if *s.name == *"main" {
            if self.main_struct.is_some() {
                return Err(SemanticError::AlreadyExists(s.name.to_string()));
            }
            self.main_struct = Some(s);
            return Ok(());
        }
        self.structs.push(s);
        Ok(())
    }

    fn analyze_required(&mut self, name: Box<str>, ty: Ty) -> Result<(), SemanticError> {
        info!("Required: {} :: {}", name, ty);
        if self.allow_requireds {
            self.requireds.push((name, ty));
            Ok(())
        } else {
            Err(SemanticError::RequiredsOnlyAtTop(name.to_string()))
        }
    }

    fn analyze_optional(
        &mut self,
        name: Box<str>,
        ty: Ty,
        default: Expr,
    ) -> Result<(), SemanticError> {
        info!("Optional: {} :: {}", name, ty);
        if self.allow_requireds {
            self.analyze_expr(default.clone(), ty, Scope::Global)?;
            self.optionals.push((name, ty, default));
            Ok(())
        } else {
            return Err(SemanticError::RequiredsOnlyAtTop(name.to_string()));
        }
    }

    fn analyze_expr(&self, expr: Expr, ty: Ty, scope: Scope) -> Result<(), SemanticError> {
        Ok(match expr {
            Expr::Ident(i) => self.analyze_ident_expr(&i, ty, scope)?,
            Expr::Fmt(FmtCall { name, fields }) => self.analyze_fmt_expr(&name, &fields, scope)?,
            Expr::Inserted(Inserted { inserts, .. }) => {
                for (_, insert) in inserts {
                    self.analyze_insert_expr(insert, scope.clone())?;
                }
            }
            Expr::Lit(lit) => {
                if lit.ty() != ty {
                    return Err(SemanticError::InvalidType(lit.ty().to_string()));
                }
            }
            Expr::Struct(_) => todo!("analyze struct"),
        })
    }

    fn analyze_insert_expr(&self, ins: Insert, scope: Scope) -> Result<(), SemanticError> {
        match ins {
            Insert::For { .. } => todo!(),
            Insert::Ident(i) => match self.sym_table.get(&Key(i.to_owned(), scope)) {
                Some(sym) => match sym.ty {
                    Ty::Str => Ok(()),
                    ty => Err(SemanticError::InvalidType(ty.to_string())),
                },
                None => Err(SemanticError::UnknownType(i.as_ref().to_owned())),
            },
            Insert::Fmt { name, args } => {
                match self.sym_table.get(&Key(name.to_owned(), scope.clone())) {
                    Some(sym) => match sym.ty {
                        Ty::Str => {
                            for arg in args {
                                match arg {
                                    Expr::Ident(ident) => {
                                        self.analyze_ident_expr(&ident, Ty::Str, sym.scope.clone())?
                                    }
                                    Expr::Fmt(FmtCall { name, fields }) => {
                                        self.analyze_fmt_expr(&name, &fields, scope.clone())?
                                    }
                                    Expr::Inserted(Inserted { .. }) => todo!(),
                                    _ => (),
                                }
                            }
                            Ok(())
                        }
                        ty => Err(SemanticError::InvalidType(ty.to_string())),
                    },
                    None => Err(SemanticError::UnknownFmt(name.into_string())),
                }
            }
        }
    }

    fn analyze_fmt_expr(
        &self,
        ident: &Box<str>,
        fields: &Vec<(Box<str>, Expr)>,
        scope: Scope,
    ) -> Result<(), SemanticError> {
        match self.sym_table.get(&Key(ident.to_owned(), scope.clone())) {
            Some(sym) => match sym.ty {
                crate::ast::Ty::Str => {
                    for (_, expr) in fields {
                        match expr {
                            Expr::Ident(ident) => {
                                self.analyze_ident_expr(ident, expr.ty(), sym.scope.clone())?
                            }
                            Expr::Fmt(FmtCall {
                                name,
                                fields: fmt_fields,
                            }) => {
                                self.analyze_fmt_expr(name, fmt_fields, scope.clone())?;
                            }
                            Expr::Inserted(Inserted { inserts, .. }) => {
                                for (_, insert) in inserts {
                                    self.analyze_insert_expr(insert.clone(), sym.scope.clone())?;
                                }
                            }
                            _ => (),
                        };
                    }
                    Ok(())
                }
                ty => Err(SemanticError::InvalidType(ty.to_string())),
            },
            None => Err(SemanticError::UnknownFmt(ident.as_ref().to_owned())),
        }
    }

    fn analyze_ident_expr(
        &self,
        ident: &Box<str>,
        ty: Ty,
        scope: Scope,
    ) -> Result<(), SemanticError> {
        match self.sym_table.get(&Key(ident.to_owned(), scope.clone())) {
            Some(sym) => match sym {
                Sym {
                    ty: sym_ty,
                    scope: sym_scope,
                    ..
                } => {
                    if sym_ty == &ty {
                        match sym_scope {
                            Scope::Global => Ok(()),
                            Scope::Local(local) => match Scope::Local(local.to_owned()) == scope {
                                true => Ok(()),
                                false => Err(SemanticError::OutOfScope(
                                    ident.as_ref().to_owned(),
                                    scope.to_string(),
                                    local.to_string(),
                                )),
                            },
                        }
                    } else {
                        Err(SemanticError::InvalidType(ty.to_string()))
                    }
                }
            },
            None => Err(SemanticError::UnknownType(ident.as_ref().to_owned())),
        }
    }

    fn analyze_str_expr(&self, expr: Expr, scope: Scope) -> Result<(), SemanticError> {
        match expr {
            Expr::Ident(i) => match self.sym_table.get(&Key(i.to_owned(), Scope::Global)) {
                Some(sym) => match sym.ty {
                    crate::ast::Ty::Str => Ok(()),
                    ty => Err(SemanticError::InvalidType(ty.to_string())),
                },
                None => Err(SemanticError::UnknownType(i.into_string())),
            },
            Expr::Lit(lit) => match lit {
                Lit::Str(_) | Lit::Char(_) => Ok(()),
                Lit::List(_) => Err(SemanticError::InvalidType("list".to_owned())),
            },
            Expr::Fmt(FmtCall { name, fields }) => {
                match self.sym_table.get(&Key(name.to_owned(), scope)) {
                    Some(sym) => match sym.ty {
                        crate::ast::Ty::Str => {
                            for (name, expr) in fields {
                                self.analyze_expr(expr, sym.ty, Scope::Local(name))?;
                            }
                            Ok(())
                        }
                        ty => Err(SemanticError::InvalidType(ty.to_string())),
                    },
                    None => Err(SemanticError::UnknownFmt(name.as_ref().to_owned())),
                }
            }
            Expr::Inserted(Inserted { inserts, .. }) => {
                for (_, ins) in inserts {
                    self.analyze_insert_expr(ins, Scope::Global)?;
                }
                Ok(())
            }
            Expr::Struct(_) => todo!("struct expression"),
        }
    }
}
