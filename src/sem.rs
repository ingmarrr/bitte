use std::collections::HashMap;

use crate::{
    ast::{Fmt, Stmt, Struct},
    err::SemanticError,
    sym::SymTable,
};

pub struct Analyzer {
    stmts: Vec<Stmt>,
    sym_table: SymTable,
    requireds: Vec<(String, String)>,
    optionals: Vec<(String, String)>,
    main_struct: Option<Struct>,
    lets: Vec<(String, String)>,
    structs: Vec<Struct>,
    fmts: Vec<Fmt>,
}

// impl Analyzer {
//     pub fn analyze(stmts: Vec<Stmt>) -> Result<Analyzer, SemanticError> {
//         let mut analyzer = Analyzer {
//             stmts,
//             requireds: Vec::new(),
//             optionals: Vec::new(),
//             main_struct: None,
//             lets: Vec::new(),
//             structs: Vec::new(),
//             fmts: Vec::new(),
//         };
//         analyzer.analyze_stmts()?;
//         Ok(analyzer)
//     }

//     fn analyze_struct(&mut self, s: &Struct) -> Result<(), SemanticError> {
//         if self.decl_table.contains_key(s.name.as_ref()) {
//             return Err(SemanticError::StructAlreadyExists(s.name.clone()));
//         }
//         self.decl_table
//             .insert(s.name.clone(), Decl::Struct(s.clone()));
//         self.structs.push(s.clone());
//         Ok(())
//     }

//     fn analyze_stmts(&mut self) -> Resutl<(), SemanticError> {
//         for stmt in &self.stmts {
//             match stmt {
//                 Stmt::Struct(s) => self.analyze_struct(s)?,
//                 Stmt::Fmt(f) => self.analyze_fmt(f)?,
//                 Stmt::Let(l) => self.analyze_let(l)?,
//                 Stmt::MainStruct(s) => self.analyze_main_struct(s)?,
//             }
//         }
//         Ok(())
//     }
// }
