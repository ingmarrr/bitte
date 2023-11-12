use crate::syntax;

pub fn check(path: String) {
    let path = std::path::Path::new(&path);
    let path = path.canonicalize().unwrap();
    let path = path.to_str().unwrap().to_string();

    let body = match std::fs::read_to_string(&path) {
        Ok(body) => body,
        Err(err) => {
            println!("Error: {}", err);
            return;
        }
    };

    let mut syn = syntax::Syntax::new(&body.as_bytes());
    let res = syn.parse_all();

    if let Ok(_) = res {
        println!("No errors found.")
    } else {
        println!("{}", res.unwrap_err().dump_err(body.as_bytes()));
    }
}

pub fn return_check(path: String) -> bool {
    let path = std::path::Path::new(&path);
    let path = path.canonicalize().unwrap();
    let path = path.to_str().unwrap().to_string();

    let body = match std::fs::read_to_string(&path) {
        Ok(body) => body,
        Err(err) => {
            println!("Error: {}", err);
            return false;
        }
    };

    let mut syn = syntax::Syntax::new(&body.as_bytes());
    let res = syn.parse_all();

    if let Ok(_) = res {
        return true;
    } else {
        return false;
    }
}
