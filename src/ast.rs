
pub enum Kw {
    Req,
    Opt,
    Let,
    Struct,
    Elem,
}

pub enum Value {
    String(String),
    List(Vec<String>),
    Struct,
    None,
}

pub struct Def {
    kw: Kw,
    ident: String,
    val: Value
}


