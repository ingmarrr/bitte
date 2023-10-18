use std::io::{Error, Write};

use crate::ast::{Dir, File};

pub struct Excecuter;

impl Excecuter {
    pub fn file(parent: std::path::PathBuf, file: File) -> Result<(), Error> {
        let path = file.path(parent.clone());
        if !path.exists() {
            let _ = std::fs::create_dir_all(parent.clone());
        }
        let fi = std::fs::File::create(path);
        match fi {
            Ok(mut f) => {
                let _ = f.write_all(file.content.as_bytes());
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn dir(dir: Dir) -> Result<(), Error> {
        let path = std::path::Path::new(&dir.name);
        println!("{:?}", path);
        if !path.exists() {
            let _ = std::fs::create_dir_all(&dir.name);
        }
        if path.is_file() {
            return Err(Error::new(
                std::io::ErrorKind::AlreadyExists,
                "Path is a file",
            ));
        }
        for file in dir.files {
            println!("{:?}", file);
            let _ = Self::file(dir.name.clone(), file);
        }
        for mut child in dir.children {
            child.name = dir.name.join(child.name);
            let _ = Self::dir(child);
        }
        Ok(())
    }
}
