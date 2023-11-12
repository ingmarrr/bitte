use std::path::PathBuf;

use crate::{
    ast::{Ast, Dir, Expr, File, Lit},
    err::ExecErr,
};

#[rustfmt::skip]
pub fn init(name: Option<String>, publish: bool, local: bool, force: bool) {
    let name = match name {
        Some(name) => name,
        None => std::env::current_dir().unwrap()
            .file_name().unwrap()
            .to_str().unwrap()
            .to_string(),
    };

    let res = into_dir(std::env::current_dir().unwrap(), true);
    if let Err(err) = res {
        println!("Error: {}", err);
        return;
    }

    // let dir = res.unwrap();
    // let body = Ast::Dir(dir).dump();
    let body = String::new();
    let fi = match std::fs::File::create(&format!("{}.ti", name)) {
        Ok(fi) => fi,
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    match std::io::Write::write_all(&mut std::io::BufWriter::new(fi), body.as_bytes()) {
        Ok(_) => println!("Done"),
        Err(err) => println!("Error: {}", err),
    };

    if publish {
        let path = name.clone() + ".ti";
        crate::publish(Some(name), path, local, force)
    }
}

fn into_dir(path: PathBuf, main: bool) -> Result<Dir, ExecErr> {
    if !path.exists() {
        return Err(ExecErr::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Path does not exist",
        )));
    }
    let mut children = Vec::new();
    for child in path.read_dir()? {
        let child = child?;
        let path = child.path();
        if path.is_dir() {
            children.push(Ast::Dir(into_dir(path, false)?));
        } else {
            children.push(Ast::File(into_file(path)?));
        }
    }
    let name = path.file_name().unwrap().to_str().unwrap().to_string();
    Ok(Dir {
        main,
        params: Vec::new(),
        path,
        alias: name.clone(),
        children,
    })
}

fn into_file(path: PathBuf) -> Result<File, ExecErr> {
    if !path.exists() {
        return Err(ExecErr::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Path does not exist",
        )));
    }
    let content = std::fs::read_to_string(path.clone())?;
    let name = path.file_name().unwrap().to_str().unwrap().to_string();

    Ok(File {
        main: false,
        params: Vec::new(),
        alias: name.clone(),
        path: name,
        content: vec![Expr::Lit(Lit::String(content))],
    })
}
