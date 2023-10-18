use crate::{decl::Decl, expr::Expr};

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Decl(Decl),
    Expr(Expr),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Ty {
    Str,
    Char,
    Int,
    List,
    Struct,
    Unknown,
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Str => f.write_str("str"),
            Ty::Char => f.write_str("char"),
            Ty::Int => f.write_str("int"),
            Ty::List => f.write_str("list"),
            Ty::Struct => f.write_str("struct"),
            Ty::Unknown => f.write_str("unknown"),
        }
    }
}
