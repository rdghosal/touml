#![allow(dead_code)]

use crate::generators::mermaid::MerimaidMapper;
use crate::generators::*;
use crate::prelude::*;

use rustpython_parser::{ast, Parse};
use std::{collections::HashSet, path::PathBuf};
use thiserror::Error;

struct PyField(String, AccessLevel);

#[derive(Debug, PartialEq, Eq, Hash)]
struct PyMethod {
    name: String,
    access: AccessLevel,
    args: Vec<String>,
    returns: Option<String>,
}

impl From<&ast::StmtFunctionDef> for PyMethod {
    fn from(value: &ast::StmtFunctionDef) -> Self {
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
impl From<&ast::StmtAsyncFunctionDef> for PyMethod {
    fn from(value: &ast::StmtAsyncFunctionDef) -> Self {
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

pub struct PyClass {
    name: String,
    access: AccessLevel,
    parents: HashSet<String>,
    methods: HashSet<PyMethod>,
    fields: HashSet<PyField>,
}

#[derive(Error, Debug)]
enum PythonParseError {
    #[error("found unexpected statement of type {0:?}")]
    StmtType(ast::Stmt),
    #[error("found unexpected expression of type {0:?}")]
    ExprType(ast::Expr),
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

fn get_fields_and_methods(cls: &ast::StmtClassDef) -> (HashSet<PyField>, HashSet<PyMethod>) {
    let mut fields = HashSet::new();
    let mut methods = HashSet::new();
    for attr in cls.body.iter() {
        match attr {
            ast::Stmt::AnnAssign(a) => todo!(),
            ast::Stmt::Assign(a) => todo!(),
            ast::Stmt::AsyncFunctionDef(func) => {
                methods.insert(PyMethod::from(func));
            }
            ast::Stmt::FunctionDef(func) => {
                methods.insert(PyMethod::from(func));
            }
            _ => todo!(),
        }
    }
    (fields, methods)
}

pub(crate) fn parse_module(contents: String, path: &str) -> Result<Vec<PyClass>> {
    let nodes = ast::Suite::parse(&contents, path);
    let parsed = nodes?
        .iter()
        .filter_map(|n| match n {
            ast::Stmt::ClassDef(cls) => Some(cls),
            _ => None,
        })
        .map(|cls| {
            let name = get_class_name(cls);
            let parents = get_parent_class_names(cls);
            let (fields, methods) = get_fields_and_methods(cls);
            // let mut fields = HashSet::new();
            todo!()
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
                dbg!("HEY!");
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
    fn test_aync_function_parse() {
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
}
