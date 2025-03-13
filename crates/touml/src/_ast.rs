use crate::errors::ParseError;
use crate::prelude::*;
use crate::python::ParseResult;

use core::iter::zip;
use rustpython_parser::ast;

fn handle_unexpected<T>(expr: ast::Expr) -> ParseResult<T> {
    Err(ParseError::UnexpectedExprType(expr))
}

pub trait PyExpr {
    fn print_value(&self) -> Option<String>;
    fn print_annotation(&self) -> ParseResult<String>;
}

impl PyExpr for ast::Expr {
    fn print_value(&self) -> Option<String> {
        match self {
            ast::Expr::Constant(c) => match &c.value {
                ast::Constant::Str(s) => Some(format!("'{}'", s)),
                ast::Constant::Int(i) => Some(i.to_string()),
                ast::Constant::Bool(b) => Some(b.to_string()),
                ast::Constant::None => Some("None".into()),
                _ => None,
            },
            ast::Expr::Set(set) => {
                let parts = set
                    .elts
                    .iter()
                    .filter_map(|elt| elt.print_value())
                    .collect::<Vec<_>>();
                Some(format!("{{{}}}", parts.join(", ")))
            }
            ast::Expr::Tuple(tuple) => {
                let parts = tuple
                    .elts
                    .iter()
                    .filter_map(|elt| elt.print_value())
                    .collect::<Vec<_>>();
                Some(format!("({},)", parts.join(", ")))
            }
            ast::Expr::List(li) => {
                let tokens = li
                    .elts
                    .iter()
                    .filter_map(|elt| elt.print_value())
                    .collect::<Vec<_>>();
                Some(format!("[{}]", tokens.join(", ")))
            }
            ast::Expr::Dict(d) => {
                let kv = zip(&d.keys, &d.values)
                    .filter_map(|(k, v)| {
                        k.as_ref().and_then(|k| {
                            k.print_value().map(|o| {
                                format!("{o}: {}", v.print_value().unwrap_or_else(|| "None".into()))
                            })
                        })
                    })
                    .collect::<Vec<_>>();
                Some(format!("{{{}}}", kv.join(", ")))
            }
            _ => None,
        }
    }

    fn print_annotation(&self) -> ParseResult<String> {
        match self {
            ast::Expr::Name(n) => Ok(n.id.to_string()),
            ast::Expr::Constant(c) => match c.value {
                ast::Constant::Ellipsis => Ok("...".into()),
                ast::Constant::None => Ok("None".into()),
                _ => handle_unexpected(self.clone())?,
            },
            ast::Expr::Subscript(s) => {
                let t_outer = match s.value.as_ref() {
                    ast::Expr::Name(t_outer) => t_outer.id.as_str().to_string(),
                    _ => handle_unexpected(self.clone())?,
                };
                let t_inner = match s.slice.as_ref() {
                    ast::Expr::Name(n) => n.id.as_str().to_string(),
                    ast::Expr::Tuple(t) => {
                        let to_concat = t
                            .elts
                            .iter()
                            .map(|elt| elt.print_annotation())
                            .collect::<ParseResult<Vec<_>>>()?;
                        to_concat.join(", ")
                    }
                    _ => handle_unexpected(self.clone())?,
                };
                Ok(format!("{}[{}]", t_outer, t_inner))
            }
            ast::Expr::BinOp(b) => {
                let left = b.left.print_annotation()?;
                let right = b.right.print_annotation()?;
                Ok(format!("{} | {}", left, right))
            }
            ast::Expr::Attribute(a) => {
                let attr = a.attr.to_string();
                let value = a.value.print_annotation()?;
                Ok(format!("{}.{}", value, attr))
            }
            _ => handle_unexpected(self.clone())?,
        }
    }
}

impl TryFrom<&ast::StmtAssign> for Field {
    type Error = ParseError;

    fn try_from(value: &ast::StmtAssign) -> ParseResult<Field> {
        let ident = value.targets.first();
        let name = match ident {
            Some(ast::Expr::Name(n)) => Ok(n.id.to_string()),
            _ => Err(ParseError::StmtAssignParse(value.clone())),
        }?;
        let (dtype, default) = match value.value.as_ref() {
            // TODO: handle container types
            ast::Expr::Constant(c) => match &c.value {
                ast::Constant::Str(s) => (
                    Some(value.value.as_ref().print_annotation()?),
                    Some(s.to_string()),
                ),
                ast::Constant::Int(i) => (Some("int".to_string()), Some(i.to_string())),
                ast::Constant::Bool(b) => (Some("bool".to_string()), Some(b.to_string())),
                ast::Constant::None => (Some("None".to_string()), None),
                _ => return Err(ParseError::StmtAssignParse(value.clone())),
            },
            _ => (None, None),
        };

        Ok(Self {
            name,
            dtype,
            default,
        })
    }
}

impl TryFrom<&ast::StmtAnnAssign> for Field {
    type Error = ParseError;

    fn try_from(value: &ast::StmtAnnAssign) -> ParseResult<Field> {
        let ident = value.target.clone();
        let dtype = value.annotation.print_annotation()?;
        let name = match *ident {
            ast::Expr::Name(ast::ExprName { id, .. }) => id.to_string(),
            _ => return Err(ParseError::ExprParse(*ident)),
        };
        let default = if value.value.is_some() {
            value.value.as_ref().and_then(|v| v.print_value())
        } else {
            None
        };

        Ok(Self {
            name,
            default,
            dtype: Some(dtype),
        })
    }
}

impl TryFrom<&ast::ArgWithDefault> for Field {
    type Error = ParseError;
    fn try_from(value: &ast::ArgWithDefault) -> ParseResult<Field> {
        let name = value.def.arg.to_string();
        let dtype = value
            .def
            .annotation
            .as_ref()
            .and_then(|a| a.print_annotation().ok());
        let default = value.default.as_ref().and_then(|d| d.print_value());

        Ok(Self {
            name,
            dtype,
            default,
        })
    }
}

macro_rules! pymethod_impl {
    ( $($s: path)+) => {
        $(
            impl TryFrom<&$s> for Method {
                type Error = ParseError;

                fn try_from(value: &$s) ->ParseResult<Self> {
                    let name = value.name.to_string();
                    let args = vec![
                        value.args.posonlyargs.clone(),
                        value.args.kwonlyargs.clone(),
                        value.args.args.clone(),
                    ]
                    .iter()
                    .flatten()
                    .map(Field::try_from)
                    .collect::<ParseResult<Vec<Field>>>()?;

                    let returns = if value.returns.is_some() {
                       value.returns.as_ref().and_then(|v| v.print_annotation().ok())
                    } else {
                        None
                    };

                    Ok(Self {
                        name,
                        args,
                        returns,
                    })
                }
            }
        )?
    };
}

pymethod_impl! {
    ast::StmtFunctionDef
    ast::StmtAsyncFunctionDef
}
