mod ast;
pub mod interactive;
mod lexer;
mod parser;
mod reader;
pub use parser::{parses_if_else, parses_let_statement, parses_prefix_expression};

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AmpError {
    #[error("invalid token, expected '{1:?}' got '{0:?}'")]
    InvalidToken(ast::Token, ast::Token),
    #[error("missing expression '{0}'")]
    MissingExpression(String),
}
