use crate::ast::Ast;
use crate::ast::Dir;
use crate::ast::Expr;
use crate::ast::File;
use crate::ast::Lit;
use crate::ast::LitExecutable;
use crate::ast::Ref;
use crate::exec::ExecErr;
use crate::exec::Syms;
use crate::exec::Ty;

use super::Key;
use super::Scope;

#[rustfmt::skip]
pub trait Resolve {
    fn resolve<O>(self, syms: &Syms, target: &Ty, args: Vec<(String, Expr)>) -> Result<O, ExecErr>
    where
        O: TryFrom<Lit, Error = ExecErr>;
}

impl Resolve for Vec<Expr> {
    fn resolve<O>(self, syms: &Syms, target: &Ty, args: Vec<(String, Expr)>) -> Result<O, ExecErr>
    where
        O: TryFrom<Lit, Error = ExecErr>,
    {
        let mut b = String::new();
        for expr in self.into_iter() {
            b.push_str(&expr.resolve::<String>(syms, target, args.clone())?);
        }
        O::try_from(Lit::String(b))
    }
}

impl Resolve for Expr {
    fn resolve<O>(self, syms: &Syms, target: &Ty, args: Vec<(String, Expr)>) -> Result<O, ExecErr>
    where
        O: TryFrom<Lit, Error = ExecErr>,
    {
        match self {
            Expr::Ref(r) => r.resolve(syms, target, args),
            Expr::Lit(l) => O::try_from(l),
        }
    }
}

impl Resolve for Ref {
    fn resolve<O>(self, syms: &Syms, target: &Ty, args: Vec<(String, Expr)>) -> Result<O, ExecErr>
    where
        O: TryFrom<Lit, Error = ExecErr>,
    {
        let sym = match syms.get(&Key(self.name.clone(), Scope::Global)) {
            None => match args.iter().find(|(name, _)| name == &self.name).cloned() {
                Some((_, expr)) => return expr.resolve(syms, target, args),
                None => return Err(ExecErr::NotFound(self.name)),
            },
            Some(s) => s,
        };

        let resolve_to = if &sym.ty() == target || target == &Ty::Unknown {
            sym.ty()
        } else {
            return Err(ExecErr::InvalidType(self.name, sym.ty().to_string()));
        };

        match sym.val {
            Ast::Req(ref req) => O::try_from(Lit::String(req.expr.to_owned())),
            Ast::Lit(ref lit) => O::try_from(Lit::String(lit.to_owned())),
            Ast::Let(ref let_) => {
                let mut b = String::new();
                for expr in let_.expr.clone().into_iter() {
                    b.push_str(&expr.resolve::<String>(syms, &resolve_to, args.clone())?);
                }
                O::try_from(Lit::String(b))
            }
            Ast::Dir(ref dir) => O::try_from(Lit::Dir(dir.clone())),
            Ast::File(ref file) => O::try_from(Lit::File(file.clone())),
            Ast::Ref(ref ref_) => ref_.clone().resolve(syms, &resolve_to, args),
        }
    }
}

impl Resolve for Lit {
    fn resolve<O>(self, _: &Syms, _: &Ty, _: Vec<(String, Expr)>) -> Result<O, ExecErr>
    where
        O: TryFrom<Lit, Error = ExecErr>,
    {
        return O::try_from(self);
    }
}

impl Resolve for String {
    fn resolve<O>(self, _: &Syms, _: &Ty, _: Vec<(String, Expr)>) -> Result<O, ExecErr>
    where
        O: TryFrom<Lit, Error = ExecErr>,
    {
        return O::try_from(Lit::String(self));
    }
}

impl Resolve for File {
    fn resolve<O>(self, _: &Syms, _: &Ty, _: Vec<(String, Expr)>) -> Result<O, ExecErr>
    where
        O: TryFrom<Lit, Error = ExecErr>,
    {
        return O::try_from(Lit::File(self));
    }
}

impl Resolve for Dir {
    fn resolve<O>(self, _: &Syms, _: &Ty, _: Vec<(String, Expr)>) -> Result<O, ExecErr>
    where
        O: TryFrom<Lit, Error = ExecErr>,
    {
        return O::try_from(Lit::Dir(self));
    }
}

impl Resolve for LitExecutable {
    fn resolve<O>(self, syms: &Syms, target: &Ty, args: Vec<(String, Expr)>) -> Result<O, ExecErr>
    where
        O: TryFrom<Lit, Error = ExecErr>,
    {
        use LitExecutable as Le;
        match self {
            Le::Dir(dir) => dir.resolve(syms, target, args),
            Le::File(file) => file.resolve(syms, target, args),
            Le::String(st) => st.resolve(syms, target, args),
        }
    }
}
