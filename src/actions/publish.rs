use std::path::Path;

use crate::Template;

use super::local::Local;

pub fn publish(name: Option<String>, path: String, local: bool, force: bool) {
    if local {
        publish_local(name, path, force);
    } else {
        publish_remote(name, path, force);
    }
}

fn publish_local(name: Option<String>, path: String, force: bool) {
    let name = match name {
        Some(name) => name,
        None => {
            let path = Path::new(&path);
            let name = path.file_stem().unwrap().to_str().unwrap();
            name.to_string()
        }
    };

    let path = Path::new(&path);
    let path = path.canonicalize().unwrap();
    let path = path.to_str().unwrap().to_string();

    let repo = match Local::new() {
        Ok(local) => local,
        Err(err) => {
            println!("Sqlite Error: {}", err);
            return;
        }
    };

    let body = match std::fs::read_to_string(&path) {
        Ok(body) => body,
        Err(err) => {
            println!("Io Error: {}", err);
            return;
        }
    };

    let res = if force {
        repo.upsert(Template { name, body })
    } else {
        repo.add(Template { name, body })
    };
    if let Err(err) = res {
        println!("Insertion Error: {}", err);
    }
}

fn publish_remote(_name: Option<String>, _path: String, _force: bool) {
    unimplemented!()
}
