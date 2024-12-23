use std::fmt;

use rustpython_parser::ast;
use thiserror::Error;

#[derive(Error, Debug)]
pub struct CliError;
impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "oopsie")
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("failed to parse Python AST from source")]
    AstParse,

    #[error("unable to parse expression {0:?}")]
    ExprParse(ast::Expr),

    #[error("unable to parse field from assignment {0:?}")]
    StmtAssignParse(ast::StmtAssign),

    #[error("unable to parse field from annotated assignment {0:?}")]
    StmtAnnAssignParse(ast::StmtAnnAssign),

    #[error("unable to parse class name from class definition {0:?}")]
    ClassNameParse(ast::StmtClassDef),

    #[error("found unexpected statement of type {0:?}")]
    UnexpectedStmtType(ast::Stmt),

    #[error("found unexpected expression of type {0:?}")]
    UnexpectedExprType(ast::Expr),
}
