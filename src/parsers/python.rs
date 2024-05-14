#![allow(dead_code)]

use crate::prelude::*;
use rayon::prelude::*;

use anyhow::bail;
use rustpython_parser::{ast, Parse};
use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::iter::zip;
use thiserror::Error;

pub fn parse_module(contents: String, path: &str) -> Result<Vec<PyClass>> {
    let nodes = ast::Suite::parse(&contents, path);
    let parsed = nodes
        .into_iter()
        .flatten()
        .par_bridge()
        .filter_map(|n| {
            if let ast::Stmt::ClassDef(cls) = n {
                Some(PyClass::try_from(cls))
            } else {
                None
            }
        })
        .collect::<Result<Vec<PyClass>>>()?;
    Ok(parsed)
}

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
    pub name: String,
    pub access: Accessibility,
    // NOTE: With BTreeSet and the ordering inherent therein, we cannot
    // preserve MRO when parsing parents of a Python class.
    pub parents: BTreeSet<String>,
    pub methods: BTreeSet<Method>,
    pub fields: BTreeSet<Field>,
}

impl PyClass {
    fn get_class_name(cls: &ast::StmtClassDef) -> String {
        cls.name.to_string()
    }

    fn get_parent_class_names(cls: &ast::StmtClassDef) -> BTreeSet<String> {
        let mut parents = BTreeSet::new();
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

    fn get_fields_from_init(func: &ast::StmtFunctionDef) -> Result<Vec<Field>> {
        let mut fields = Vec::new();
        for node in func.body.iter() {
            let mut field_names = Vec::new();
            let mut dtype = None;
            match node {
                ast::Stmt::Assign(assignment) => {
                    for target in assignment.targets.iter() {
                        let mut is_self = false;
                        if let ast::Expr::Attribute(attr) = target {
                            if let ast::Expr::Name(assignee) = attr.value.borrow() {
                                if assignee.id.to_string() == "self".to_string() {
                                    is_self = true;
                                }
                            }
                            if is_self {
                                field_names.push(attr.attr.to_string());
                            }
                        }
                    }
                }
                ast::Stmt::AnnAssign(assignment) => {
                    let mut is_self = false;
                    if let ast::Expr::Attribute(attr) = assignment.target.borrow() {
                        if let ast::Expr::Name(assignee) = attr.value.borrow() {
                            if assignee.id.to_string() == "self".to_string() {
                                is_self = true;
                            }
                        }
                        if is_self {
                            field_names.push(attr.attr.to_string());
                        }
                    }
                    dtype = get_pytype(assignment.annotation.borrow())?;
                }
                _ => todo!(),
            }

            for field_name in field_names {
                fields.push(Field {
                    access: get_access_from_name(&field_name),
                    name: field_name,
                    dtype: dtype.clone(),
                    default: None,
                });
            }
        }

        Ok(fields)
    }

    fn get_fields_and_methods(
        cls: &ast::StmtClassDef,
    ) -> Result<(BTreeSet<Field>, BTreeSet<Method>)> {
        let mut fields = BTreeSet::new();
        let mut methods = BTreeSet::new();
        let mut is_std_cls = false; // TODO: use to discriminate class variables, etc.
        for attr in cls.body.iter() {
            match attr {
                ast::Stmt::AnnAssign(a) => {
                    fields.insert(Field::try_from(a)?);
                }
                ast::Stmt::Assign(a) => {
                    fields.insert(Field::try_from(a)?);
                }
                ast::Stmt::AsyncFunctionDef(func) => {
                    methods.insert(Method::try_from(func)?);
                }
                ast::Stmt::FunctionDef(func) => {
                    let method = Method::try_from(func)?;
                    if method.name == "__init__" {
                        is_std_cls = true;
                        fields.extend(Self::get_fields_from_init(func)?);
                    }
                    methods.insert(method);
                }
                _ => continue,
            }
        }
        Ok((fields, methods))
    }
}

impl TryFrom<ast::StmtClassDef> for PyClass {
    type Error = anyhow::Error;
    fn try_from(value: ast::StmtClassDef) -> Result<Self> {
        let name = Self::get_class_name(&value);
        let access = get_access_from_name(&name);
        let parents = Self::get_parent_class_names(&value);
        let (fields, methods) = Self::get_fields_and_methods(&value)?;
        let result = PyClass {
            name,
            access,
            parents,
            fields,
            methods,
        };
        Ok(result)
    }
}

impl TryFrom<&ast::StmtAssign> for Field {
    type Error = anyhow::Error;
    fn try_from(value: &ast::StmtAssign) -> Result<Field> {
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
        let (dtype, default) = match value.value.borrow() {
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
            dtype,
            default,
        })
    }
}

impl TryFrom<&ast::StmtAnnAssign> for Field {
    type Error = anyhow::Error;
    fn try_from(value: &ast::StmtAnnAssign) -> Result<Field> {
        let name: String;
        let ident = value.target.clone();
        let dtype = get_pytype(value.annotation.borrow())?;
        if let ast::Expr::Name(n) = *ident {
            name = n.id.to_string();
        } else {
            bail!(PyParseError::StmtAnnAssignParse(value.clone()));
        };
        let access = get_access_from_name(&name);
        let default = if value.value.is_some() {
            get_pyvalue(value.value.as_ref().unwrap())
        } else {
            None
        };
        Ok(Self {
            name,
            access,
            dtype,
            default,
        })
    }
}

impl TryFrom<&ast::ArgWithDefault> for Field {
    type Error = anyhow::Error;
    fn try_from(value: &ast::ArgWithDefault) -> Result<Field> {
        let name = value.def.arg.to_string();
        let dtype = get_pytype(&*value.def.annotation.as_ref().unwrap())?;
        let access = get_access_from_name(&name);
        let default = if value.default.is_some() {
            get_pyvalue(&*value.default.as_ref().unwrap())
        } else {
            None
        };
        Ok(Self {
            name,
            access,
            dtype,
            default,
        })
    }
}

macro_rules! pymethod_impl {
    ( $($s: path)+) => {
        $(
            impl TryFrom<&$s> for Method {
                type Error = anyhow::Error;
                fn try_from(value: &$s) -> Result<Self> {
                    let name = value.name.to_string();
                    let access = get_access_from_name(&name);
                    let args = vec![
                        value.args.posonlyargs.clone(),
                        value.args.kwonlyargs.clone(),
                        value.args.args.clone(),
                    ]
                    .iter()
                    .flatten()
                    .map(Field::try_from)
                    .collect::<Result<Vec<Field>>>()?;
                    let returns = if value.returns.is_some() {
                        get_pytype(&*value.returns.as_ref().unwrap())?
                    } else {
                        None
                    };
                    Ok(Self {
                        name,
                        access,
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

fn get_access_from_name(name: &str) -> Accessibility {
    if name.starts_with("_") {
        Accessibility::Private
    } else {
        Accessibility::Public
    }
}

fn get_pyvalue(expr: &ast::Expr) -> Option<String> {
    match expr {
        ast::Expr::Constant(c) => match &c.value {
            ast::Constant::Str(s) => Some(format!("'{}'", s.to_string())),
            ast::Constant::Int(i) => Some(i.to_string()),
            ast::Constant::Bool(b) => Some(b.to_string()),
            ast::Constant::None => Some("None".to_string()),
            _ => None,
        },
        ast::Expr::Set(set) => {
            let tokens = set
                .elts
                .iter()
                .filter_map(|e| get_pyvalue(e))
                .collect::<Vec<_>>();
            Some(format!("{{{}}}", tokens.join(", ")))
        }
        ast::Expr::Tuple(tuple) => {
            let tokens = tuple
                .elts
                .iter()
                .filter_map(|e| get_pyvalue(e))
                .collect::<Vec<_>>();
            Some(format!("({},)", tokens.join(", ")))
        }
        ast::Expr::List(li) => {
            let tokens = li
                .elts
                .iter()
                .filter_map(|e| get_pyvalue(e))
                .collect::<Vec<_>>();
            Some(format!("[{}]", tokens.join(", ")))
        }
        ast::Expr::Dict(d) => {
            let kv = zip(&d.keys, &d.values)
                .filter_map(|(k, v)| {
                    if k.is_some() {
                        if let Some(k) = get_pyvalue(k.as_ref().unwrap()) {
                            let v = get_pyvalue(&v);
                            return Some((k, v));
                        }
                    }
                    return None;
                })
                .map(|(k, v)| format!("{}: {}", k, v.unwrap_or_else(|| String::from("None"))))
                .collect::<Vec<_>>();
            Some(format!("{{{}}}", kv.join(", ")))
        }
        _ => None,
    }
}

fn get_pytype(annotation: &ast::Expr) -> Result<Option<String>> {
    let result = match annotation {
        ast::Expr::Name(n) => Some(n.id.to_string()),
        ast::Expr::Constant(c) => match c.value {
            ast::Constant::Ellipsis => Some("...".to_string()),
            _ => bail!(PyParseError::ExprType(annotation.clone())),
        },
        ast::Expr::Subscript(s) => {
            let t_outer = match s.value.borrow() {
                ast::Expr::Name(t_outer) => t_outer.id.as_str(),
                _ => bail!(PyParseError::ExprType(annotation.clone())),
            };
            let t_inner = match s.slice.borrow() {
                ast::Expr::Name(n) => n.id.as_str().to_string(),
                ast::Expr::Tuple(t) => {
                    let mapped = t.elts.iter().map(get_pytype).collect::<Result<Vec<_>>>();
                    let mut to_concat = Vec::<String>::new();
                    match mapped {
                        Ok(types) => {
                            for t in types {
                                if t.is_none() {
                                    bail!(PyParseError::ExprType(annotation.clone()))
                                }
                                to_concat.push(t.unwrap());
                            }
                        }
                        _ => bail!(PyParseError::ExprType(annotation.clone())),
                    }
                    to_concat.join(", ")
                }
                _ => bail!(PyParseError::ExprType(annotation.clone())),
            };
            Some(format!("{}[{}]", t_outer, t_inner))
        }
        _ => bail!(PyParseError::ExprType(annotation.clone())),
    };
    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_stmt(py: &str) -> ast::Stmt {
        ast::Suite::parse(py, ".").unwrap().swap_remove(0)
    }

    #[test]
    fn test_sync_function_parse() {
        #[rustfmt::skip]
        let py = vec![
            "def my_func(name: str):",
            r#"    return f'Hello {name}!'"#,
        ].join(&"\n");

        if let ast::Stmt::FunctionDef(ref f) = get_stmt(&py) {
            let method = Method::try_from(f).unwrap();
            assert_eq!(
                method,
                Method {
                    name: "my_func".to_string(),
                    args: vec![Field {
                        name: "name".to_string(),
                        default: None,
                        access: Accessibility::Public,
                        dtype: Some("str".to_string()),
                    }],
                    returns: None,
                    access: Accessibility::Public
                }
            );
        } else {
            panic!("failed to parse function");
        }
    }

    #[test]
    fn test_async_function_parse() {
        #[rustfmt::skip]
        let py = vec![
            "async def _my_other_func(name: str, age: int = 18) -> str:",
            r#"    return f"Hello, I'm {name} and I'm {int} years-old!""#,
        ].join(&"\n");

        if let ast::Stmt::AsyncFunctionDef(ref f) = get_stmt(&py) {
            let method = Method::try_from(f).unwrap();
            assert_eq!(
                method,
                Method {
                    name: "_my_other_func".to_string(),
                    args: vec![
                        Field {
                            name: "name".to_string(),
                            default: None,
                            access: Accessibility::Public,
                            dtype: Some("str".to_string()),
                        },
                        Field {
                            name: "age".to_string(),
                            default: Some("18".to_string()),
                            access: Accessibility::Public,
                            dtype: Some("int".to_string()),
                        }
                    ],
                    returns: Some("str".to_string()),
                    access: Accessibility::Private
                }
            )
        } else {
            panic!("failed to parse function");
        }
    }

    #[test]
    fn test_parse_assignment() {
        let py = "x = 42";
        if let ast::Stmt::Assign(ref a) = get_stmt(&py) {
            let assignment = Field::try_from(a).unwrap();
            assert_eq!(
                assignment,
                Field {
                    name: "x".to_string(),
                    dtype: Some("int".to_string()),
                    default: Some("42".to_string()),
                    access: Accessibility::Public
                }
            );
        } else {
            panic!("failed to parse assignment");
        }
    }

    #[test]
    fn test_parse_annotated_list_assignment() {
        let py = "x: list[int] = [1, 2, 3]";
        if let ast::Stmt::AnnAssign(ref a) = get_stmt(&py) {
            let assignment = Field::try_from(a).unwrap();
            assert_eq!(
                assignment,
                Field {
                    name: "x".to_string(),
                    dtype: Some("list[int]".to_string()),
                    default: Some("[1, 2, 3]".to_string()),
                    access: Accessibility::Public
                }
            );
        } else {
            panic!("failed to parse assignment");
        }
    }

    #[test]
    fn test_parse_annotated_dict_assignment() {
        let py = "x: dict[str, tuple[int, ...]] = {'a': (1, 2), 'b': (2,), 'c': (3, 3, 3,)}";
        if let ast::Stmt::AnnAssign(ref a) = get_stmt(&py) {
            let assignment = Field::try_from(a).unwrap();
            assert_eq!(
                assignment,
                Field {
                    name: "x".to_string(),
                    dtype: Some("dict[str, tuple[int, ...]]".to_string()),
                    default: Some("{'a': (1, 2,), 'b': (2,), 'c': (3, 3, 3,)}".to_string()),
                    access: Accessibility::Public
                }
            );
        } else {
            panic!("failed to parse assignment");
        }
    }

    #[test]
    fn test_parse_fields_from_init() {
        #[rustfmt::skip]
        let py = vec![
            "class MyClass:",
            "   def __init__(self, name, id) -> None:",
            "       self.id = id",
            "       self.name = name",
        ]
        .join(&"\n");

        if let ast::Stmt::ClassDef(ref c) = get_stmt(&py) {
            if let ast::Stmt::FunctionDef(ref f) = c.body[0] {
                let result = PyClass::get_fields_from_init(f);
                return assert_eq!(
                    result.unwrap(),
                    vec![
                        Field {
                            name: "id".to_string(),
                            dtype: None,
                            default: None,
                            access: Accessibility::Public
                        },
                        Field {
                            name: "name".to_string(),
                            dtype: None,
                            default: None,
                            access: Accessibility::Public
                        },
                    ]
                );
            }
        }
        panic!("failed to parse class");
    }
}
