use crate::generators::mermaid::MerimaidMapper;
use crate::prelude::*;
use crate::generators::*;

use rustpython_parser::{ast, Parse};
use std::{collections::HashSet, path::PathBuf};

struct PyField(String, AccessLevel);

#[derive(PartialEq, Eq, Hash)]
struct PyMethod {
    name: String,
    access: AccessLevel,
    args: Vec<String>,
    returns: Option<String>,
}

pub struct PyClass {
    name: String,
    access: AccessLevel,
    parents: HashSet<String>,
    methods: HashSet<PyMethod>,
    fields: HashSet<PyField>,
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
            let name = cls.name.to_string();
            let mut parents = HashSet::new();
            let mut methods = HashSet::new();
            // let mut fields = HashSet::new();
            for parent in cls.bases.iter() {
                match parent {
                    ast::Expr::Name(name) => {
                        parents.insert(name.id.to_string());
                    }
                    ast::Expr::Attribute(attr) => {
                        if let ast::Expr::Name(name) = attr.value.as_ref() {
                            parents.insert(name.id.to_string());
                        } else {
                            // TODO: return error here instead of printing
                            eprintln!("Failed to extract base class name from attribute.");
                        }
                    }
                    _ => todo!(),
                }
            }
            for attr in cls.body.iter() {
                match attr {
                    ast::Stmt::FunctionDef(func) => {
                        let name = func.name.to_string();
                        let mut args = Vec::new();
                        args.extend(func.args.posonlyargs.clone());
                        args.extend(func.args.kwonlyargs.clone());
                        args.extend(func.args.args.clone());
                        let args = args.iter().map(|a| a.def.arg.to_string()).collect();
                        let returns = if func.returns.is_some() {
                            match *func.returns.clone().unwrap() {
                                ast::Expr::Constant(c) => {
                                    Some(String::new())
                                    // todo!("Match constant as ast::Expr::Constant")
                                    // constant.value
                                }
                                ast::Expr::Attribute(a) => Some(a.attr.to_string()),
                                ast::Expr::Name(n) => Some(n.id.to_string()),
                                _ => todo!(),
                            }
                        } else {
                            None
                        };

                        let access = if name.starts_with("_") {
                            AccessLevel::Private
                        } else {
                            AccessLevel::Public
                        };

                        methods.insert(PyMethod {
                            name,
                            access,
                            args,
                            returns,
                        });
                    }
                    _ => todo!(),
                }
            }
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
