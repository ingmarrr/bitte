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
    pub sym_table: SymTable,
    pub requireds: Vec<(String, Ty)>,
    optionals: Vec<(String, Ty, Expr)>,
    pub main_struct: Option<Struct>,
    pub lets: Vec<Let>,
    pub structs: Vec<Struct>,
    allow_args: bool,
    pub fmts: Vec<Fmt>,
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
            allow_args: true,
        }
    }

    pub fn has_req(&self, name: String, ty: Ty) -> bool {
        self.requireds.contains(&(name, ty))
    }

    pub fn analyze_all(&mut self, decls: Vec<Decl>) -> Option<()> {
        let (sym_table, errs) = SymTable::from_decls(&decls);
        self.sym_table = sym_table;
        self.errs = errs;
        for decl in decls {
            if !decl.is_arg() {
                self.allow_args = false;
            }
            if let Err(err) = self.analyze_decl(decl) {
                self.errs.push(err);
            }
        }
        if self.main_struct.is_none() {
            self.errs.push(SemanticError::NoMainStruct);
        }
        if self.errs.is_empty() {
            Some(())
        } else {
            None
        }
    }

    pub fn analyze(&mut self, decl: Decl) -> Option<()> {
        let (syms, mut errs) = self.sym_table.sym(&decl);
        self.errs.append(&mut errs);
        if let Err(e) = self.sym_table.insert_all(syms) {
            self.errs.extend(e);
        }
        let err = self.analyze_decl(decl);
        if let Err(err) = err {
            self.errs.push(err);
        }
        if self.errs.is_empty() {
            Some(())
        } else {
            None
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
        self.analyze_expr(let_decl.expr.clone(), let_decl.ty, Scope::Global)?;
        self.lets.push(let_decl);
        Ok(())
    }

    fn analyze_struct(&mut self, struc: Struct) -> Result<(), SemanticError> {
        for (_, expr) in struc.files.clone() {
            match expr {
                Some(expr) => self.analyze_str_expr(expr, Scope::Local(struc.name.to_owned()))?,
                None => {}
            }
        }

        if *struc.name == *"main" {
            if self.main_struct.is_some() {
                return Err(SemanticError::AlreadyExists(struc.name.to_string()));
            }
            self.main_struct = Some(struc);
            return Ok(());
        }
        self.structs.push(struc);
        Ok(())
    }

    fn analyze_required(&mut self, name: String, ty: Ty) -> Result<(), SemanticError> {
        info!("Required: {} :: {}", name, ty);
        if self.allow_args {
            self.requireds.push((name, ty));
            Ok(())
        } else {
            Err(SemanticError::RequiredsOnlyAtTop(name))
        }
    }

    fn analyze_optional(
        &mut self,
        name: String,
        ty: Ty,
        default: Expr,
    ) -> Result<(), SemanticError> {
        info!("Optional: {} :: {}", name, ty);
        if self.allow_args {
            self.analyze_expr(default.clone(), ty, Scope::Global)?;
            self.optionals.push((name, ty, default));
            Ok(())
        } else {
            Err(SemanticError::RequiredsOnlyAtTop(name))
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
                None => Err(SemanticError::UnknownType(i.to_owned())),
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
                    None => Err(SemanticError::UnknownFmt(name)),
                }
            }
        }
    }

    fn analyze_fmt_expr(
        &self,
        ident: &String,
        fields: &Vec<(String, Expr)>,
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
            None => Err(SemanticError::UnknownFmt(ident.to_owned())),
        }
    }

    fn analyze_ident_expr(&self, ident: &str, ty: Ty, scope: Scope) -> Result<(), SemanticError> {
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
                                    ident.to_owned(),
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
            None => Err(SemanticError::UnknownType(ident.to_owned())),
        }
    }

    fn analyze_str_expr(&self, expr: Expr, scope: Scope) -> Result<(), SemanticError> {
        match expr {
            Expr::Ident(i) => match self.sym_table.get(&Key(i.clone(), Scope::Global)) {
                Some(sym) => match sym.ty {
                    crate::ast::Ty::Str => Ok(()),
                    ty => Err(SemanticError::InvalidType(ty.to_string())),
                },
                None => Err(SemanticError::UnknownType(i)),
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
                    None => Err(SemanticError::UnknownFmt(name.to_owned())),
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
