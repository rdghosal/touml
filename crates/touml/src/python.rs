use crate::_ast::PyExpr;
use crate::errors;
use crate::prelude::*;

use rustpython_parser::{ast, Parse};
use std::collections::BTreeSet;

pub type ParseResult<T> = core::result::Result<T, errors::ParseError>;
type Result<T> = ParseResult<T>;

pub struct PyClassInfo {
    pub name: String,
    pub fields: BTreeSet<Field>,
    pub methods: BTreeSet<Method>,

    /// NOTE: With BTreeSet and the ordering inherent therein, we cannot
    /// preserve MRO when parsing parents of a Python class.
    pub parents: BTreeSet<String>,
}

impl PyClassInfo {
    pub fn from_source(src: &str) -> Result<impl Iterator<Item = Result<Self>>> {
        let parsed = ast::Suite::parse(src, "path").map_err(|_| errors::ParseError::AstParse)?;

        let mapped = parsed.into_iter().filter_map(|node| match node {
            ast::Stmt::ClassDef(stmt) => Some(PyClassInfo::try_from(stmt)),
            _ => None,
        });

        Ok(mapped)
    }

    fn get_class_name(cls: &ast::StmtClassDef) -> String {
        cls.name.to_string()
    }

    fn get_parent_class_names(cls: &ast::StmtClassDef) -> Result<BTreeSet<String>> {
        cls.bases
            .iter()
            .map(|base| match base {
                ast::Expr::Name(name) => Ok(name.id.to_string()),
                ast::Expr::Attribute(attr) => match attr.value.as_ref() {
                    ast::Expr::Name(ast::ExprName { id, .. }) => Ok(format!("{id}.{}", attr.attr)),
                    _ => Ok(attr.attr.to_string()),
                },
                _ => Err(errors::ParseError::ExprParse(base.clone())),
            })
            .collect::<Result<_>>()
    }

    fn get_fields_from_init(func: &ast::StmtFunctionDef) -> Vec<Field> {
        func.body
            .iter()
            .flat_map(|node| {
                // Collect all assignment nodes.
                let fields = match node {
                    // e.g., self.attr1 = self.attr2 = ... = 'some value'
                    ast::Stmt::Assign(assignment) => assignment
                        .targets
                        .iter()
                        .map(|t| (t, None))
                        .collect::<Vec<_>>(),

                    // e.g., self.attr1: str = 'some value'
                    ast::Stmt::AnnAssign(assignment) => {
                        vec![(
                            assignment.target.as_ref(),
                            Some(assignment.annotation.clone()),
                        )]
                    }

                    _ => vec![],
                };

                // Map only those that are attributes on `self` as Fields.
                // e.g., `var = 1` is skipped, whereas `self.var = 1` is not.
                fields.into_iter().filter_map(|(f, a)| {
                    if let ast::Expr::Attribute(ast::ExprAttribute { value, attr, .. }) = f {
                        if let ast::Expr::Name(ast::ExprName { id, .. }) = value.as_ref() {
                            if id.as_str() == "self" {
                                // TODO: Change method return and surface errors?
                                let annotation = a.and_then(|a| a.print_annotation().ok());
                                return Some(Field {
                                    name: attr.to_string(),
                                    dtype: annotation,
                                    default: None,
                                });
                            }
                        }
                    }
                    None
                })
            })
            .collect()
    }

    fn get_fields_and_methods(
        cls: &ast::StmtClassDef,
    ) -> Result<(BTreeSet<Field>, BTreeSet<Method>)> {
        let mut fields = BTreeSet::new();
        let mut methods = BTreeSet::new();
        // let mut is_std_cls = false; // TODO: use to discriminate class variables, etc.
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
                        //is_std_cls = true;
                        fields.extend(Self::get_fields_from_init(func));
                    }
                    methods.insert(method);
                }
                _ => continue,
            }
        }
        Ok((fields, methods))
    }
}

impl TryFrom<ast::StmtClassDef> for PyClassInfo {
    type Error = errors::ParseError;

    fn try_from(value: ast::StmtClassDef) -> Result<Self> {
        let name = Self::get_class_name(&value);
        let parents = Self::get_parent_class_names(&value)?;
        let (fields, methods) = Self::get_fields_and_methods(&value)?;

        Ok(PyClassInfo {
            name,
            parents,
            fields,
            methods,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rustpython_parser::Parse;

    fn get_stmt(py: &str) -> ast::Stmt {
        ast::Suite::parse(py, ".").unwrap().swap_remove(0)
    }

    #[test]
    fn test_sync_function_parse() {
        #[rustfmt::skip]
        let py = [
            "def my_func(name: str):",
            r#"    return f'Hello {name}!'"#,
        ].join("\n");

        if let ast::Stmt::FunctionDef(ref f) = get_stmt(&py) {
            let method = Method::try_from(f).unwrap();
            assert_eq!(
                method,
                Method {
                    name: "my_func".to_string(),
                    args: vec![Field {
                        name: "name".to_string(),
                        default: None,
                        dtype: Some("str".to_string()),
                    }],
                    returns: None,
                }
            );
        } else {
            panic!("failed to parse function");
        }
    }

    #[test]
    fn test_async_function_parse() {
        #[rustfmt::skip]
        let py = [
            "async def _my_other_func(name: str, age: int = 18) -> str:",
            r#"    return f"Hello, I'm {name} and I'm {int} years-old!""#,
        ].join("\n");

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
                            dtype: Some("str".to_string()),
                        },
                        Field {
                            name: "age".to_string(),
                            default: Some("18".to_string()),
                            dtype: Some("int".to_string()),
                        }
                    ],
                    returns: Some("str".to_string()),
                }
            )
        } else {
            panic!("failed to parse function");
        }
    }

    #[test]
    fn test_parse_assignment() {
        let py = "x = 42";
        if let ast::Stmt::Assign(ref a) = get_stmt(py) {
            let assignment = Field::try_from(a).unwrap();
            assert_eq!(
                assignment,
                Field {
                    name: "x".to_string(),
                    dtype: Some("int".to_string()),
                    default: Some("42".to_string()),
                }
            );
        } else {
            panic!("failed to parse assignment");
        }
    }

    #[test]
    fn test_parse_annotated_list_assignment() {
        let py = "x: list[int] = [1, 2, 3]";
        if let ast::Stmt::AnnAssign(ref a) = get_stmt(py) {
            let assignment = Field::try_from(a).unwrap();
            assert_eq!(
                assignment,
                Field {
                    name: "x".to_string(),
                    dtype: Some("list[int]".to_string()),
                    default: Some("[1, 2, 3]".to_string()),
                }
            );
        } else {
            panic!("failed to parse assignment");
        }
    }

    #[test]
    fn test_parse_annotated_dict_assignment() {
        let py = "x: dict[str, tuple[int, ...]] = {'a': (1, 2), 'b': (2,), 'c': (3, 3, 3,)}";
        if let ast::Stmt::AnnAssign(ref a) = get_stmt(py) {
            let assignment = Field::try_from(a).unwrap();
            assert_eq!(
                assignment,
                Field {
                    name: "x".to_string(),
                    dtype: Some("dict[str, tuple[int, ...]]".to_string()),
                    default: Some("{'a': (1, 2,), 'b': (2,), 'c': (3, 3, 3,)}".to_string()),
                }
            );
        } else {
            panic!("failed to parse assignment");
        }
    }

    #[test]
    fn test_parse_annotation_union() {
        let py = "x: dict | int | None = {'a': (1, 2), 'b': (2,), 'c': (3, 3, 3,)}";
        if let ast::Stmt::AnnAssign(ref a) = get_stmt(py) {
            let assignment = Field::try_from(a).unwrap();
            assert_eq!(
                assignment,
                Field {
                    name: "x".to_string(),
                    dtype: Some("dict | int | None".to_string()),
                    default: Some("{'a': (1, 2,), 'b': (2,), 'c': (3, 3, 3,)}".to_string()),
                }
            );
        } else {
            panic!("failed to parse assignment");
        }
    }

    #[test]
    fn test_parse_annotation_typing_union() {
        let py = "x: Union[dict, int, None] = {'a': (1, 2), 'b': (2,), 'c': (3, 3, 3,)}";
        if let ast::Stmt::AnnAssign(ref a) = get_stmt(py) {
            let assignment = Field::try_from(a).unwrap();
            assert_eq!(
                assignment,
                Field {
                    name: "x".to_string(),
                    dtype: Some("Union[dict, int, None]".to_string()),
                    default: Some("{'a': (1, 2,), 'b': (2,), 'c': (3, 3, 3,)}".to_string()),
                }
            );
        } else {
            panic!("failed to parse assignment");
        }
    }

    #[test]
    fn test_parse_annotation_attrs() {
        let py = "x: t.Any = None";
        if let ast::Stmt::AnnAssign(ref a) = get_stmt(py) {
            let assignment = Field::try_from(a).unwrap();
            assert_eq!(
                assignment,
                Field {
                    name: "x".to_string(),
                    dtype: Some("t.Any".to_string()),
                    default: Some("None".to_string()),
                }
            );
        } else {
            panic!("failed to parse assignment");
        }
    }

    #[test]
    fn test_parse_fields_from_init() {
        #[rustfmt::skip]
        let py = [
            "class MyClass:",
            "   def __init__(self, name, id) -> None:",
            "       self.id = id",
            "       self.name = name",
        ]
        .join("\n");

        if let ast::Stmt::ClassDef(ref c) = get_stmt(&py) {
            if let ast::Stmt::FunctionDef(ref f) = c.body[0] {
                let result = PyClassInfo::get_fields_from_init(f);
                return assert_eq!(
                    result,
                    vec![
                        Field {
                            name: "id".to_string(),
                            dtype: None,
                            default: None,
                        },
                        Field {
                            name: "name".to_string(),
                            dtype: None,
                            default: None,
                        },
                    ]
                );
            }
        }
        panic!("failed to parse class");
    }
}
