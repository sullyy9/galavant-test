use super::{
    error::Error,
    evaluate::{evaluate, FrontendRequest},
    expression::Expr,
    parse::parse_from_str,
};

////////////////////////////////////////////////////////////////
// types
////////////////////////////////////////////////////////////////

/// Interpreter for test scripts.
///
#[derive(PartialEq, Clone, Debug)]
pub struct Interpreter {
    ast: Vec<Expr>,
    index: usize,
}

////////////////////////////////////////////////////////////////
// construction / conversion
////////////////////////////////////////////////////////////////

impl Interpreter {
    pub fn try_from_str(script: &str) -> Result<Self, Vec<Error>> {
        Ok(Self {
            ast: parse_from_str(script)?,
            index: 0,
        })
    }
}

////////////////////////////////////////////////////////////////
// iteration
////////////////////////////////////////////////////////////////

impl Iterator for Interpreter {
    type Item = Result<FrontendRequest, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(expr) = self.ast.get(self.index) {
            self.index += 1;
            Some(evaluate(expr))
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////
