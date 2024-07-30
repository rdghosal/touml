use rustpython_parser::ast;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("found unexpected statement of type {0:?}")]
    UnexpectedStmtType(ast::Stmt),

    #[error("found unexpected expression of type {0:?}")]
    UnexpectedExprType(ast::Expr),

    #[error("unable to parse expression {0:?}")]
    ExprParse(ast::Expr),

    #[error("unable to parse field from assignment {0:?}")]
    StmtAssignParse(ast::StmtAssign),

    #[error("unable to parse field from annotated assignment {0:?}")]
    StmtAnnAssignParse(ast::StmtAnnAssign),

    #[error("unable to parse class name from class definition {0:?}")]
    ClassNameParse(ast::StmtClassDef),

}