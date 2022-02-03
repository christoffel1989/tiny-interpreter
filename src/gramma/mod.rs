#[macro_use]
mod util;
pub mod token;
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod environment;
mod usrfun;
pub mod primitive;
pub mod evaluator;

mod test;