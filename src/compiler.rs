use crate::parser::*;
use crate::prelude::*;

use std::collections::BTreeSet;

#[cfg(windows)]
static EOL: &str = "\r\n";

#[cfg(not(windows))]
static EOL: &str = "\n";

static INDENT: &str = "    ";

pub trait MermaidMappable {
    fn to_mermaid(self) -> MermaidClass;
}

pub struct MermaidClass {
    name: String,
    parents: BTreeSet<String>,
    methods: BTreeSet<Method>,
    fields: BTreeSet<Field>,
}

impl MermaidClass {
    fn print(&self) -> String {
        let mut result = vec![];

        // Define class as well as the fields and methods therein.
        let class_name = format!("{INDENT}class {} {{", self.name);
        result.push(class_name);
        result.extend(self.make_class_fields());
        result.extend(self.make_class_methods());

        for parent in self.parents.iter() {
            result.push(format!("{INDENT}`{parent}` <|-- {}", self.name));
        }

        result.join(EOL).to_string()
    }

    fn make_class_methods(&self) -> Vec<String> {
        let mut result = self
            .methods
            .iter()
            .map(|method| {
                let access_modifier = match method.is_public() {
                    true => '+',
                    false => '-',
                };
                let mut method_str = format!("{INDENT}{INDENT}{access_modifier}{}(", method.name);
                let mut args: Vec<String> = vec![];
                for arg in method.args.iter() {
                    if let Some(t) = &arg.dtype {
                        args.push(format!("{t} {}", arg.name));
                    } else {
                        args.push(arg.name.to_owned());
                    }
                }
                if args.is_empty() {
                    let args_str = args.join(", ");
                    method_str.push_str(args_str.as_str());
                }
                method_str.push(')');
                if let Some(return_type) = &method.returns {
                    method_str.push_str(&format!(" {return_type}"));
                }
                method_str
            })
            .collect::<Vec<_>>();
        result.push(format!("{}}}", INDENT));
        result
    }

    fn make_class_fields(&self) -> Vec<String> {
        let mut result = Vec::with_capacity(self.fields.len());
        for field in self.fields.iter() {
            let line = match (&field.dtype, &field.default) {
                (Some(t), _) => {
                    format!("{INDENT}{INDENT}+{} {t}", field.name)
                }
                (_, Some(d)) => {
                    format!("{INDENT}{INDENT}+{} = {d}", field.name)
                }
                _ => format!("{INDENT}{INDENT}+{}", field.name),
            };
            result.push(line);
        }
        result
    }
}

impl MermaidMappable for PyClassInfo {
    // TODO: make opinionation here configurable
    fn to_mermaid(self) -> MermaidClass {
        let methods = self
            .methods
            .into_iter()
            // Remove dunders.
            .filter(|m| !(m.name.starts_with("__") & m.name.ends_with("__")))
            .collect::<BTreeSet<Method>>();
        let fields = self
            .fields
            .into_iter()
            .filter(|f| !(f.name.starts_with("__") & f.name.ends_with("__")))
            .collect::<BTreeSet<Field>>();

        MermaidClass {
            name: self.name,
            parents: self.parents,
            methods,
            fields,
        }
    }
}

pub fn make_class_diagram<T>(nodes: Vec<T>) -> String
where
    T: MermaidMappable,
{
    let mut result = vec![String::from("classDiagram")];
    for node in nodes.into_iter() {
        result.push(node.to_mermaid().print());
    }
    result.join(EOL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mermaid_display() {
        let cls = PyClassInfo {
            name: "TestClass".to_string(),
            parents: BTreeSet::from([
                "ParentTestClass".to_string(),
                "AnotherTestClass".to_string(),
            ]),
            fields: BTreeSet::from([Field {
                name: "id".to_string(),
                dtype: Some("int".to_string()),
                default: None,
            }]),
            methods: BTreeSet::new(),
        };
        assert_eq!(
            format!("{}", cls.to_mermaid().print()),
            [
                "    class TestClass {",
                "        +id int",
                "    }",
                "    `AnotherTestClass` <|-- TestClass",
                "    `ParentTestClass` <|-- TestClass",
            ]
            .join(EOL)
        )
    }
}
