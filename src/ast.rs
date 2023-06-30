#[derive(Debug)]
pub struct Program {
    pub defs: Vec<Def>,
}

#[derive(Debug)]
pub enum Def {
    Init {
        kind: InitKind,
        ident: String,
    },
    Let {
        ty: Type,
        ident: String,
        val: String,
    },
    Elem {
        ident: String,
        params: Vec<Param>,
        val: String,
    },
    Partial {
        ident: String,
        params: Vec<Param>,
        val: Format,
    },
    Struct {
        ident: String,
        val: Format,
    },
}

#[derive(Debug)]
pub struct Format {
    body: String,
    insertions: Vec<String>,
}

#[derive(Debug)]
pub struct Param {
    ty: Type,
    ident: String,
}

#[derive(Debug)]
pub enum Type {
    String,
    List,
}

#[derive(Debug)]
pub enum InitKind {
    Required,
    Optional,
}
