use crate::{
    ast::Ty,
    expr::{Expr, Insert},
};

#[derive(Debug, PartialEq)]
pub enum Decl {
    Struct(Struct),
    Let(Let),
    Fmt(Fmt),
    Required(Required),
    Optional(Optional),
}

impl Decl {
    pub fn name(&self) -> &str {
        match self {
            Decl::Struct(s) => &s.name,
            Decl::Let(l) => &l.name,
            Decl::Fmt(f) => &f.name,
            Decl::Required(r) => &r.name,
            Decl::Optional(o) => &o.name,
        }
    }

    pub fn ty(&self) -> Ty {
        match self {
            Decl::Struct(_) => Ty::Struct,
            Decl::Let(l) => l.ty,
            Decl::Fmt(_) => Ty::Str,
            Decl::Required(r) => r.ty,
            Decl::Optional(o) => o.ty,
        }
    }

    pub fn is_arg(&self) -> bool {
        match self {
            Decl::Required(_) | Decl::Optional(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Struct {
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub children: Vec<Struct>,
    pub files: Vec<(String, Option<Expr>)>,
}

#[derive(Debug, PartialEq)]
pub struct Let {
    pub name: String,
    pub ty: Ty,
    pub expr: Expr,
}

#[derive(Debug, PartialEq)]
pub struct Fmt {
    pub name: String,
    pub params: Vec<(String, Ty)>,
    pub string_parts: Vec<String>,
    pub inserts: Vec<(usize, Insert)>,
}

#[derive(Debug, PartialEq)]
pub struct Required {
    pub name: String,
    pub ty: Ty,
}

#[derive(Debug, PartialEq)]
pub struct Optional {
    pub name: String,
    pub ty: Ty,
    pub default: Expr,
}
