pub fn args(args: Vec<String>) -> Option<Vec<(String, String)>> {
    let mut res = Vec::new();
    for arg in args {
        let mut arg = arg.split('=');
        let name = arg.next().unwrap().to_string();
        let val = arg.next().unwrap().to_string();
        res.push((name, val));
    }
    Some(res)
}
