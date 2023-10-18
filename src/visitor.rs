use std::error::Error;

use crate::err::Trace;

pub trait Visitor<'a, E>
where
    E: Error + 'static,
{
    fn visit_file(&mut self) -> Result<(), Trace<'a, E>>;
    fn visit_dir(&mut self) -> Result<(), Trace<'a, E>>;
    // fn visit_expr(&mut self) -> Result<
}

pub struct Interpreter;

impl<'a, E> Visitor<'a, E> for Interpreter
where
    E: Error + 'static,
{
    fn visit_file(&mut self) -> Result<(), Trace<'a, E>> {
        todo!()
    }

    fn visit_dir(&mut self) -> Result<(), Trace<'a, E>> {
        todo!()
    }
}
