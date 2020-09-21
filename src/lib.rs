mod ast;
pub mod interactive;
mod lexer;
mod parser;
mod reader;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AmpError {
    #[error("invalid token, expected '{1:?}' got '{0:?}'")]
    InvalidToken(ast::Token, ast::Token),
    #[error("missing expression '{0}'")]
    MissingExpression(String),
}
