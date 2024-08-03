use crate::prelude::*;
use crate::python::*;

use std::collections::BTreeSet;

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
    pub fn print(&self) -> String {
        let mut result = vec![];

        // Define class as well as the fields and methods therein.
        let class_name = format!("{INDENT}class {} {{{EOL}", self.name);
        result.push(class_name);
        result.extend(self.make_class_fields());
        result.extend(self.make_class_methods());
        result.push(format!("{INDENT}}}{EOL}{EOL}"));

        for (i, parent) in self.parents.iter().enumerate() {
            let parent_str = if i == self.parents.len() - 1 {
                format!("{INDENT}`{parent}` <|-- {}", self.name)
            } else {
                format!("{INDENT}`{parent}` <|-- {}{EOL}", self.name)
            };
            result.push(parent_str)
        }

        result.push(EOL.to_string());
        result.join("")
    }

    fn get_access_modifier(is_public: bool) -> String {
        match is_public {
            true => "+".to_string(),
            false => "-".to_string(),
        }
    }

    fn make_class_methods(&self) -> Vec<String> {
        self.methods
            .iter()
            .map(|method| {
                let access_modifier = Self::get_access_modifier(method.is_public());
                let mut method_str = format!("{INDENT}{INDENT}{access_modifier} {}(", method.name);

                let args = method
                    .args
                    .iter()
                    .map(|a| {
                        a.dtype
                            .clone()
                            .map_or_else(|| a.name.to_string(), |t| format!("{t} {}", a.name))
                    })
                    .collect::<Vec<_>>();

                if !args.is_empty() {
                    let args_str = args.join(", ");
                    method_str.push_str(args_str.as_str());
                }
                method_str.push(')');

                if let Some(return_type) = &method.returns {
                    method_str.push_str(&format!(" {return_type}"));
                }

                method_str.push_str(EOL);
                method_str
            })
            .collect::<Vec<_>>()
    }

    fn make_class_fields(&self) -> Vec<String> {
        let mut result = Vec::with_capacity(self.fields.len());
        for field in self.fields.iter() {
            let access_modifier = Self::get_access_modifier(field.is_public());
            let line = match (&field.dtype, &field.default) {
                (Some(t), _) => {
                    format!("{INDENT}{INDENT}{access_modifier} {} {t}{EOL}", field.name)
                }
                (_, Some(d)) => {
                    format!(
                        "{INDENT}{INDENT}{access_modifier} {} = {d}{EOL}",
                        field.name
                    )
                }
                _ => format!("{INDENT}{INDENT}{access_modifier} {}{EOL}", field.name),
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

//pub fn make_class_diagram<T>(nodes: impl Iterator<Item = T>) -> String
//where
//    T: MermaidMappable,
//{
//    std::iter::once(String::from("classDiagram"))
//        .chain(nodes.map(|node| node.to_mermaid().print()))
//        .chain(std::iter::once(EOL.to_string()))
//        .collect::<Vec<_>>()
//        .join("\n")
//}
//
//
//
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
