use crate::ast::{Dir, Expr, File, Lit, Ref, Ty};

pub trait Dump {
    fn dump(&self) -> String;
}

impl Dump for Dir {
    fn dump(&self) -> String {
        let mut buf = String::from("dir ");
        buf.push_str(&self.alias);
        if self.params.len() > 0 {
            buf.push_str(" (");
            for (ix, param) in self.params.iter().enumerate() {
                if ix > 0 {
                    buf.push_str(", ");
                }
                buf.push_str(&param.dump());
            }
            buf.push_str(")");
        }
        buf.push_str(": ");
        buf.push_str(&self.path.to_str().unwrap());
        buf.push_str(" {");
        buf.push_str(&self.children.dump());
        buf.push_str("}");
        buf
    }
}

impl Dump for File {
    fn dump(&self) -> String {
        let mut buf = String::from("file ");
        buf.push_str(&self.alias);
        if self.params.len() > 0 {
            buf.push_str(" (");
            for (ix, param) in self.params.iter().enumerate() {
                if ix > 0 {
                    buf.push_str(", ");
                }
                buf.push_str(&param.dump());
            }
            buf.push_str(")");
        }
        buf.push_str(": ");
        buf.push_str(&self.path);
        buf.push_str("{\"");
        buf.push_str(&self.content.dump());
        buf.push_str("\"}");
        buf
    }
}

impl Dump for &(String, Ty) {
    fn dump(&self) -> String {
        format!("{}: {}", self.0, self.1.dump())
    }
}

impl Dump for Ty {
    fn dump(&self) -> String {
        match self {
            Ty::String => "str".into(),
            Ty::Int => "int".into(),
            Ty::List => "list".into(),
            Ty::Dir => "dir".into(),
            Ty::File => "file".into(),
            Ty::Unknown => "unknown".into(),
        }
    }
}

impl Dump for Vec<Expr> {
    fn dump(&self) -> String {
        let mut buf = String::new();
        for (ix, expr) in self.iter().enumerate() {
            if ix > 0 {
                buf.push_str(", ");
            }
            buf.push_str(&expr.dump());
        }
        buf
    }
}

impl Dump for Expr {
    fn dump(&self) -> String {
        match self {
            Expr::Lit(l) => l.dump(),
            Expr::Ref(r) => r.dump(),
            Expr::If(_) => todo!(),
        }
    }
}

impl Dump for Lit {
    fn dump(&self) -> String {
        match self {
            Lit::String(s) => format!("\"{}\"", s),
            Lit::File(f) => {
                let mut buf = format!("\"{}\":", f.path);
                buf.push_str(&f.content.dump());
                buf
            }
            Lit::Dir(d) => {
                let mut buf = format!("\"{}\" {{", d.path.to_str().unwrap());
                for child in &d.children {
                    buf.push_str(&child.dump());
                    buf.push_str(", ");
                }
                buf
            }
            Lit::Int(i) => i.to_string(),
            Lit::BinOp(op) => format!("({} {} {})", op.lhs.dump(), op.op, op.rhs.dump()),
        }
    }
}

impl Dump for Ref {
    fn dump(&self) -> String {
        let mut buf = String::from("@");
        buf.push_str(&self.name);
        if self.args.len() > 0 {
            buf.push_str("(");
            for (ix, arg) in self.args.iter().enumerate() {
                if ix > 0 {
                    buf.push_str(", ");
                }
                buf.push_str(&arg.dump());
            }
            buf.push_str(")");
        }
        buf
    }
}

impl Dump for &(String, Expr) {
    fn dump(&self) -> String {
        format!("{}: {}", self.0, self.1.dump())
    }
}
