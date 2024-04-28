use crate::parsers::*;
use crate::prelude::*;

use std::collections::HashSet;
use std::fmt::Display;

static INDENT: &str = "    ";

pub struct MermaidClass {
    name: String,
    parents: HashSet<String>,
    methods: HashSet<Method>,
    fields: HashSet<Field>,
}

pub trait MermaidMappable {
    fn as_mermaid(self) -> MermaidClass;
}

// fn make_class_methods(cls: &MermaidClass) -> Vec<String> {
//     let mut result = Vec::with_capacity(cls.methods.len());
//     for method in &cls.methods {
//         let access_modifier = match method.access {
//             Accessibility::Public => '+',
//             Accessibility::Private => '-',
//         };

//         let mut method_str = format!("{INDENT}{INDENT}{access_modifier}{}(", method.name);
//         let mut args: Vec<String> = vec![];
//         for arg in method.args.iter() {
//             if let Some(t) = &arg.dtype {
//                 args.push(format!("{t} {}", arg.name));
//             } else {
//                 args.push(arg.name.to_owned());
//             }
//         }
//         if args.len() > 0 {
//             let args_str = args.join(", ");
//             method_str.push_str(args_str.as_str());
//         }
//         method_str.push_str(")");
//         if let Some(return_type) = &method.returns {
//             method_str.push_str(&format!(" {return_type}"));
//         }
//         result.push(method_str);
//     }
//     result.push(format!("{}}}", INDENT));
//     result
// }

// fn make_class_fields(model: &PyClass) -> Vec<String> {
//     let mut result = Vec::with_capacity(model.fields.len());
//     for field in model.fields.iter() {
//         let line = match (&field.dtype, &field.default) {
//             (Some(t), _) => {
//                 format!("{INDENT}{INDENT}+{} {t}", field.name)
//             }
//             (_, Some(d)) => {
//                 format!("{INDENT}{INDENT}+{} = {d}", field.name)
//             }
//             _ => format!("{INDENT}{INDENT}+{}", field.name),
//         };
//         result.push(line);
//     }
//     result
// }

// impl Display for MermaidClass {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//                 let mut result = vec![];
//         let inherits = " <|-- ";
//         for model in models.iter() {
//             // Define class as well as the fields and methods therein.
//             result.push(String::from("classDiagram"));

//             // Define class as well as the fields and methods therein.
//             let class_name = format!("{INDENT}class {}{{", model.name);
//             result.push(class_name);
//             result.extend(Self::make_class_fields(model));
//             result.extend(Self::make_class_methods(model));
//             for parent in model.parents.iter() {
//                 result.push(format!("{INDENT}`{parent}`{inherits}{}", model.name));
//             }
//         }
//         Ok(result.join("\r\n"))

//         return Some(

//         )

//     }
// }

impl MermaidMappable for PyClass {
    fn as_mermaid(self) -> MermaidClass {
        // Remove dunders.
        // TODO: make opinionation here configurable
        let methods = self
            .methods
            .into_iter()
            .filter(|m| !(m.name.starts_with("__") & m.name.ends_with("__")))
            .collect::<HashSet<Method>>();
        let fields = self
            .fields
            .into_iter()
            .filter(|f| !(f.name.starts_with("__") & f.name.ends_with("__")))
            .collect::<HashSet<Field>>();
        return MermaidClass {
            name: self.name,
            parents: self.parents,
            methods,
            fields,
        };
    }
}

fn make_class_diagram(nodes: Vec<&dyn MermaidMappable>) -> String {
    todo!()
}
