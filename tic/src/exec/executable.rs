use std::io::Error;
use std::io::Write;

use crate::ast::Ast;
use crate::ast::Dir;
use crate::ast::File;
use crate::ast::LitExecutable;
use crate::ast::Ty;
use crate::exec::ExecErr;
use crate::exec::Expr;
use crate::exec::Syms;

use super::resolve::Resolve;

pub trait Executable: Resolve {
    fn execute(
        self,
        syms: &Syms,
        parent: std::path::PathBuf,
        args: Vec<(String, Expr)>,
    ) -> Result<(), ExecErr>;
}

impl Executable for LitExecutable {
    fn execute(
        self,
        syms: &Syms,
        parent: std::path::PathBuf,
        args: Vec<(String, Expr)>,
    ) -> Result<(), ExecErr> {
        use LitExecutable as Le;
        match self {
            Le::File(file) => file.execute(syms, parent, args),
            Le::Dir(dir) => dir.execute(syms, parent, args),
            Le::String(st) => st.execute(syms, parent, args),
        }
    }
}

impl Executable for Dir {
    fn execute(
        self,
        syms: &Syms,
        parent: std::path::PathBuf,
        args: Vec<(String, Expr)>,
    ) -> Result<(), ExecErr> {
        let path = parent.join(&self.path);
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        for child in self.children.into_iter() {
            match child {
                Ast::File(file) => file.execute(syms, path.clone(), args.clone())?,
                Ast::Dir(dir) => dir.execute(syms, path.clone(), args.clone())?,
                Ast::Ref(ref_) => ref_
                    .clone()
                    .resolve::<LitExecutable>(syms, &Ty::Unknown, args.clone())?
                    .execute(syms, path.clone(), args.clone())?,
                _ => return Err(Error::new(std::io::ErrorKind::InvalidData, "Expected dir").into()),
            }
        }

        Ok(())
    }
}

impl Executable for File {
    fn execute(
        self,
        syms: &Syms,
        parent: std::path::PathBuf,
        args: Vec<(String, Expr)>,
    ) -> Result<(), ExecErr> {
        let path = parent.join(&self.path);
        if !parent.exists() {
            std::fs::create_dir_all(&parent)?;
        }

        let mut fi = std::fs::File::create(path)?;
        let body = self.content.resolve::<String>(syms, &Ty::String, args)?;
        let _ = fi.write_all(body.as_bytes());
        Ok(())
    }
}

impl Executable for String {
    fn execute(
        self,
        _: &Syms,
        _: std::path::PathBuf,
        _: Vec<(String, Expr)>,
    ) -> Result<(), ExecErr> {
        println!("{}", self);
        Ok(())
    }
}
