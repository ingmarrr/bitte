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
}

impl Decl {
    pub fn name(&self) -> &str {
        match self {
            Decl::Struct(s) => &s.name,
            Decl::Let(l) => &l.name,
            Decl::Fmt(f) => &f.name,
            Decl::Required(r) => &r.name,
        }
    }

    pub fn ty(&self) -> Ty {
        match self {
            Decl::Struct(s) => Ty::Struct,
            Decl::Let(l) => l.ty.clone(),
            Decl::Fmt(f) => Ty::Str,
            Decl::Required(r) => r.ty.clone(),
        }
    }

    pub fn is_req(&self) -> bool {
        match self {
            Decl::Required(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Fmt {
    pub name: Box<str>,
    pub params: Vec<(Box<str>, Ty)>,
    pub string_parts: Vec<Box<str>>,
    pub inserts: Vec<(usize, Insert)>,
}

#[derive(Debug, PartialEq)]
pub struct Let {
    pub name: Box<str>,
    pub ty: Ty,
    pub expr: Expr,
}

#[derive(Debug, PartialEq)]
pub struct Required {
    pub name: Box<str>,
    pub ty: Ty,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Struct {
    pub name: Box<str>,
    pub params: Vec<(Box<str>, Ty)>,
    pub children: Vec<Struct>,
    pub files: Vec<(Box<str>, Option<Expr>)>,
}
