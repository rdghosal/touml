#![allow(dead_code)]

use crate::generators::mermaid::MerimaidMapper;
use crate::generators::*;
use crate::prelude::*;

use anyhow::bail;
use rustpython_parser::{ast, Parse};
use std::borrow::Borrow;
use std::{collections::HashSet, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
enum PyParseError {
    #[error("found unexpected statement of type {0:?}")]
    StmtType(ast::Stmt),
    #[error("found unexpected expression of type {0:?}")]
    ExprType(ast::Expr),
    #[error("unable to parse field from assignment {0:?}")]
    StmtAssignParse(ast::StmtAssign),
    #[error("unable to parse field from assignment {0:?}")]
    StmtAnnAssignParse(ast::StmtAnnAssign),
}

pub struct PyClass {
    name: String,
    access: AccessLevel,
    parents: HashSet<String>,
    methods: HashSet<PyMethod>,
    fields: HashSet<PyField>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct PyField {
    name: String,
    pytype: Option<String>,
    default: Option<String>,
    access: AccessLevel,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct PyMethod {
    name: String,
    access: AccessLevel,
    args: Vec<String>,
    returns: Option<String>,
}

macro_rules! pymethod_impl {
    ( $($s: path)+) => {
        $(
            impl From<&$s> for PyMethod {
                fn from(value: &$s) -> Self {
                    let name = value.name.to_string();
                    let access = get_access_from_name(&name);
                    let args = vec![
                        value.args.posonlyargs.clone(),
                        value.args.kwonlyargs.clone(),
                        value.args.args.clone(),
                    ]
                    .iter()
                    .flatten()
                    .map(|a| a.def.arg.to_string())
                    .collect();
                    let returns = if value.returns.is_some() {
                        match *value.returns.clone().unwrap() {
                            ast::Expr::Constant(c) => {
                                Some(c.value.as_str().unwrap_or(&String::new()).to_owned())
                            }
                            ast::Expr::Attribute(a) => Some(a.attr.to_string()),
                            ast::Expr::Name(n) => Some(n.id.to_string()),
                            _ => {
                                dbg!("failed to parse {:?}", &value.returns);
                                None
                            }
                        }
                    } else {
                        None
                    };

                    Self {
                        name,
                        access,
                        args,
                        returns,
                    }
                }
            }
        )?
    };
}

pymethod_impl! {
    ast::StmtFunctionDef
    ast::StmtAsyncFunctionDef
}

impl TryFrom<&ast::StmtAssign> for PyField {
    type Error = anyhow::Error;
    fn try_from(value: &ast::StmtAssign) -> Result<PyField> {
        let ident = value.targets.iter().next();
        let name = match ident {
            Some(expr) => {
                if let ast::Expr::Name(n) = expr {
                    Some(n.id.to_string())
                } else {
                    None
                }
            }
            None => None,
        };
        let Some(name) = name else {
            bail!(PyParseError::StmtAssignParse(value.clone()));
        };
        let access = get_access_from_name(&name);
        let (pytype, default) = match value.value.borrow() {
            // TODO: handle container types
            ast::Expr::Constant(c) => match &c.value {
                ast::Constant::Str(s) => (Some("str".to_string()), Some(s.to_string())),
                ast::Constant::Int(i) => (Some("int".to_string()), Some(i.to_string())),
                ast::Constant::Bool(b) => (Some("bool".to_string()), Some(b.to_string())),
                ast::Constant::None => (Some("None".to_string()), None),
                _ => bail!(PyParseError::StmtAssignParse(value.clone())),
            },
            _ => (None, None),
        };
        Ok(Self {
            name,
            access,
            pytype,
            default,
        })
    }
}

impl TryFrom<&ast::StmtAnnAssign> for PyField {
    type Error = anyhow::Error;
    fn try_from(value: &ast::StmtAnnAssign) -> Result<PyField> {
        let name: String;
        let ident = value.target.clone();
        let pytype = match value.annotation.borrow() {
            ast::Expr::Name(n) => {
                Some(n.id.to_string())
            },
            ast::Expr::Subscript(s) => {
                let ast::Expr::Name(container_t) = s.value.borrow() else {
                    bail!(PyParseError::StmtAnnAssignParse(value.clone()));
                };
                let ast::Expr::Name(contained_t) = s.slice.borrow() else {
                    bail!(PyParseError::StmtAnnAssignParse(value.clone()));
                };
                Some(format!(
                    "{}[{}]",
                    container_t.id.as_str(),
                    contained_t.id.as_str()
                ))
            },
            _ => bail!(PyParseError::StmtAnnAssignParse(value.clone()))
        };
        if let ast::Expr::Name(n) = *ident {
            name = n.id.to_string();
        } else {
            bail!(PyParseError::StmtAnnAssignParse(value.clone()));
        };
        let access = get_access_from_name(&name);
        let default = match &value.value {
            Some(v) => match v.borrow() {
                ast::Expr::Constant(c) => match &c.value {
                    ast::Constant::Str(s) => Some(s.to_string()),
                    ast::Constant::Int(i) => Some(i.to_string()),
                    ast::Constant::Bool(b) => Some(b.to_string()),
                    ast::Constant::None => Some("None".to_string()),
                    _ => None,
                },
                _ => bail!(PyParseError::StmtAnnAssignParse(value.clone())),
            },
            None => None,
        };
        Ok(Self {
            name,
            access,
            pytype,
            default,
        })
    }
}

fn get_access_from_name(name: &str) -> AccessLevel {
    if name.starts_with("_") {
        AccessLevel::Private
    } else {
        AccessLevel::Public
    }
}

fn get_class_name(cls: &ast::StmtClassDef) -> String {
    cls.name.to_string()
}

fn get_parent_class_names(cls: &ast::StmtClassDef) -> HashSet<String> {
    let mut parents = HashSet::new();
    for parent in cls.bases.iter() {
        match parent {
            ast::Expr::Name(name) => {
                parents.insert(name.id.to_string());
            }
            ast::Expr::Attribute(attr) => {
                if let ast::Expr::Name(name) = attr.value.as_ref() {
                    parents.insert(name.id.to_string());
                } else {
                    eprintln!("Failed to extract base class name from attribute.");
                }
            }
            _ => {
                dbg!("failed to parse {:?}", parent);
            }
        }
    }
    parents
}

fn get_fields_and_methods(
    cls: &ast::StmtClassDef,
) -> Result<(HashSet<PyField>, HashSet<PyMethod>)> {
    let mut fields = HashSet::new();
    let mut methods = HashSet::new();
    for attr in cls.body.iter() {
        match attr {
            ast::Stmt::AnnAssign(a) => {
                fields.insert(PyField::try_from(a)?);
            }
            ast::Stmt::Assign(a) => {
                fields.insert(PyField::try_from(a)?);
            }
            ast::Stmt::AsyncFunctionDef(func) => {
                methods.insert(PyMethod::from(func));
            }
            ast::Stmt::FunctionDef(func) => {
                methods.insert(PyMethod::from(func));
            }
            _ => todo!(),
        }
    }
    Ok((fields, methods))
}

pub fn parse_module(contents: String, path: &str) -> Result<Vec<PyClass>> {
    let nodes = ast::Suite::parse(&contents, path);
    let parsed = nodes?
        .iter()
        .filter_map(|n| match n {
            ast::Stmt::ClassDef(cls) => Some(cls),
            _ => None,
        })
        .map(|cls| {
            let name = get_class_name(cls);
            let access = get_access_from_name(&name);
            let parents = get_parent_class_names(cls);
            let (fields, methods) = get_fields_and_methods(cls).unwrap(); // TODO: handle error
            PyClass {
                name,
                access,
                parents,
                fields,
                methods,
            }
        })
        .collect::<Vec<PyClass>>();
    Ok(parsed)
}

impl MerimaidMapper for PyClass {
    fn to_mermaid(self) -> mermaid::MermaidClass {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sync_function_parse() {
        let func = r#"
def my_func(name: str):
    return f"Hello {name}!"
"#;
        match &ast::Suite::parse(func, ".").unwrap()[0] {
            ast::Stmt::FunctionDef(f) => {
                let method = PyMethod::from(f);
                assert_eq!(
                    method,
                    PyMethod {
                        name: "my_func".to_string(),
                        args: vec!["name".to_string()],
                        returns: None,
                        access: AccessLevel::Public
                    }
                );
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                let method = PyMethod::from(f);
                assert_eq!(
                    method,
                    PyMethod {
                        name: "my_other_func".to_string(),
                        args: vec!["name".to_string(), "age".to_string()],
                        returns: Some("str".to_string()),
                        access: AccessLevel::Private
                    }
                );
            }
            _ => panic!("failed to parse function"),
        };
    }

    #[test]
    fn test_async_function_parse() {
        let func = r#"
async def _my_other_func(name: str, age: int = 18) -> str:
    return f"Hello, I'm {name} and I'm {int} years-old!"
"#;
        match &ast::Suite::parse(func, ".").unwrap()[0] {
            ast::Stmt::AsyncFunctionDef(f) => {
                let method = PyMethod::from(f);
                assert_eq!(
                    method,
                    PyMethod {
                        name: "_my_other_func".to_string(),
                        args: vec!["name".to_string(), "age".to_string()],
                        returns: Some("str".to_string()),
                        access: AccessLevel::Private
                    }
                );
            }
            _ => panic!("failed to parse function"),
        }
    }

    #[test]
    fn test_parse_assignment() {
        let stmt = "x = 42";
        match &ast::Suite::parse(stmt, ".").unwrap()[0] {
            ast::Stmt::Assign(a) => {
                let assignment = PyField::try_from(a).unwrap();
                assert_eq!(
                    assignment,
                    PyField {
                        name: "x".to_string(),
                        pytype: Some("int".to_string()),
                        default: Some("42".to_string()),
                        access: AccessLevel::Public
                    }
                );
            }
            _ => panic!("failed to parse assignment"),
        }
    }

    #[test]
    fn test_parse_annotated_assignment() {
        let stmt = "x: list[int]";
        match &ast::Suite::parse(stmt, ".").unwrap()[0] {
            ast::Stmt::AnnAssign(a) => {
                let assignment = PyField::try_from(a).unwrap();
                assert_eq!(
                    assignment,
                    PyField {
                        name: "x".to_string(),
                        pytype: Some("list[int]".to_string()),
                        default: None,
                        access: AccessLevel::Public
                    }
                );
            }
            _ => panic!("failed to parse assignment"),
        }
    }
}
