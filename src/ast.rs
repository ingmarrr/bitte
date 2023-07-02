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
        val: Value,
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
pub enum Value {
    String {
        body: String,
        insertions: Vec<Insertion>,
    },
    List {
        content: Vec<String>,
    },
    ListStr {
        ident: ListIdent,
        body: String,
        insertions: Vec<Insertion>,
    },
    Element {
        ident: String,
        inputs: Vec<String>,
    },
}

#[derive(Debug)]
pub enum Insertion {
    Ident(String),
    Value(Value),
    Empty,
}

#[derive(Debug)]
pub enum ListIdent {
    Anonymos(Vec<String>),
    Name(String),
}

impl From<&str> for Insertion {
    fn from(value: &str) -> Self {
        match value {
            "" => Self::Empty,
            _ => Self::Ident(value.to_owned()),
        }
    }
}

impl From<String> for Insertion {
    fn from(value: String) -> Self {
        match value.as_str() {
            "" => Self::Empty,
            _ => Self::Ident(value),
        }
    }
}

#[derive(Debug)]
pub struct Format;

#[derive(Debug)]
pub struct Param {
    _ty: Type,
    _ident: String,
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
